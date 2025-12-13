#![feature(iter_array_chunks,
           test,
           portable_simd,
           generic_const_exprs,
           select_unpredictable)]
#![allow(unused_parens, non_snake_case, dead_code)]
#![feature(trait_alias)]
mod sorting;
mod horizontal;
use random_data::random_numbers;

use std::{cmp::Ordering,
          simd::{LaneCount, SupportedLaneCount, prelude::*}};

use rayon::slice::ParallelSliceMut;

fn main()
{
    let a = random_numbers(1024, 0, 1000);
    let b = random_numbers(1024, 0, 1000);
    println!("{:?}",
             compare_arrays_control(&a[0..1024].try_into().unwrap(),
                                    &b[0..1024].try_into().unwrap()));
}

//count the number of items in a less than, equal to, and greater than their
//counterpart in b
fn compare_arrays2(a: &[i32; 1024], b: &[i32; 1024]) -> (i32, i32, i32)
{
    let mut lt: Simd<i32, 16> = Simd::from_array([0; 16]);
    let mut gt: Simd<i32, 16> = Simd::from_array([0; 16]);
    for i in (0..1024).step_by(16)
    {
        let va = Simd::from_slice(&a[i..i + 16]);
        let vb = Simd::from_slice(&b[i..i + 16]);
        let gt_mask = va.simd_gt(vb);
        let lt_mask = va.simd_lt(vb);
        gt += gt_mask.to_int();
        lt += lt_mask.to_int();
    }
    let lt = -lt.reduce_sum();
    let gt = -gt.reduce_sum();
    let eq = 1024 - lt - gt;
    return (lt, eq, gt);
}

#[inline]
fn compare_arrays3(a: &[i32; 1024], b: &[i32; 1024]) -> (i32, i32, i32)
{
    let mut lt: Simd<i32, 16> = Simd::from_array([0; 16]);
    let mut gt: Simd<i32, 16> = Simd::from_array([0; 16]);
    for (a, b) in a.iter().map(|&e|e).array_chunks::<16>().zip(b.iter().map(|&e|e).array_chunks::<16>())
    {
        let va = Simd::from_array(a);
        let vb = Simd::from_array(b);
        let gt_mask = va.simd_gt(vb);
        let lt_mask = va.simd_lt(vb);
        gt += gt_mask.to_int();
        lt += lt_mask.to_int();
    }
    let lt = -lt.reduce_sum();
    let gt = -gt.reduce_sum();
    let eq = 1024 - lt - gt;
    return (lt, eq, gt);
}

#[inline]
fn compare_arrays4<const VLEN: usize>(a: &[i32; 1024], b: &[i32; 1024]) -> (i32, i32, i32)
    where LaneCount<VLEN>: SupportedLaneCount
{
    let mut lt: Simd<i32, VLEN> = Simd::from_array([0; VLEN]);
    let mut gt: Simd<i32, VLEN> = Simd::from_array([0; VLEN]);
    for (a, b) in a.iter().map(|&e|e).array_chunks::<VLEN>().zip(b.iter().map(|&e|e).array_chunks::<VLEN>())
    {
        let va = Simd::from_array(a);
        let vb = Simd::from_array(b);
        let gt_mask = va.simd_gt(vb);
        let lt_mask = va.simd_lt(vb);
        gt += gt_mask.to_int();
        lt += lt_mask.to_int();
    }
    let lt = -lt.reduce_sum();
    let gt = -gt.reduce_sum();
    let eq = 1024 - lt - gt;
    return (lt, eq, gt);
}
//count the number of items in a less than, equal to, and greater than their
//counterpart in b
fn compare_arrays_control(a: &[i32; 1024], b: &[i32; 1024]) -> (i32, i32, i32)
{
    let mut lt: i32 = 0;
    let mut gt: i32 = 0;
    for _ in (0..1024)
    {
        lt += if (a < b) { 1 } else { 0 };
        gt += if (a > b) { 1 } else { 0 };
    }
    let eq = 1024 - lt - gt;
    return (lt, eq, gt);
}

fn compare_strings<const VLEN: usize>(s1: &str, s2: &str) -> Ordering
    where LaneCount<VLEN>: SupportedLaneCount
{
    let bytes1 = s1.as_bytes();
    let bytes2 = s2.as_bytes();
    let chunks1 = bytes1.iter().map(|&e|e).array_chunks::<VLEN>();
    let chunks2 = bytes2.iter().map(|&e|e).array_chunks::<VLEN>();
    let r1 = chunks1.clone().into_remainder();
    let r2 = chunks2.clone().into_remainder();
    for (va, vb) in chunks1.zip(chunks2)
    {
        let va = Simd::from_array(va);
        let vb = Simd::from_array(vb);
        let mask = va.simd_ne(vb);
        if (mask.any())
        {
            let idx = mask.first_set().unwrap();
            return va[idx].cmp(&vb[idx]);
        }
    }

    return r1.cmp(r2);
}

struct IStringPool
{
    previews: Vec<u64>,
    strings:  Vec<String>, //could be faster but whatever.
    len:      usize,
}

impl IStringPool
{
    pub fn new() -> Self
    {
        return Self { previews: vec![],
                      strings:  vec![],
                      len:      0, };
    }

    pub fn from_vec(strings: Vec<String>) -> Self
    {
        let mut pool = Self::new();
        pool.strings = strings;
        for s in &pool.strings
        {
            let preview = unsafe { Self::make_preview(s) };
            pool.previews.push(preview);
        }
        return pool;
    }

    pub fn add(&mut self, s: String) -> usize
    {
        let preview = unsafe { Self::make_preview(&s) };
        self.previews.push(preview);
        self.strings.push(s);
        self.len += 1;
        return self.len - 1;
    }

    ///only works properly on little endian systems.
    unsafe fn make_preview(s: &str) -> u64
    {
        let mut preview = [0u8; 8];
        //maybe this can be a simd load_or ?
        for (i, byte) in s.as_bytes().iter().take(8).enumerate()
        //8th is null terminator
        {
            preview[7 - i] = *byte;
        }
        return unsafe { std::mem::transmute(preview) };
    }
}

//8 strings at a time. maybe 16 will be faster?
//count how many strings in the pool are less than needle
fn compare_string_lt<const VLEN: usize>(needle: &str, tests: &IStringPool) -> i64
    where LaneCount<VLEN>: SupportedLaneCount
{
    let needle_preview = unsafe { IStringPool::make_preview(needle) };
    let needles: Simd<u64, VLEN> = Simd::splat(needle_preview);
    let mut counter = Simd::splat(0);
    for (chunk_idx, schunk) in tests.previews.iter().map(|&e|e).array_chunks::<VLEN>().enumerate()
    {
        let hay = Simd::from_slice(&schunk);
        let mut ltmask = hay.simd_lt(needles);
        let mut eqmask = hay.simd_eq(needles);
        while (eqmask.any() && needle.len() > 8)
        //if needle is 8 characters or less and matches, hay cant be less than it.
        {
            let fs = eqmask.first_set().unwrap(); //just checked
            eqmask.set(fs, false);
            if let Ordering::Less =
                compare_strings::<VLEN>(&tests.strings[chunk_idx * VLEN + fs], needle)
            {
                ltmask.set(fs, true);
            }
        }
        counter += ltmask.to_int();
    }
    return -counter.reduce_sum();
}

// #[cfg(test)]
// mod test
// {
//     extern crate test;
//     use super::*;
//     use test::Bencher;

//     mod benchmarks
//     {
//         use super::*;

//         //366ns per iter with strlen 16, 8x par, 1024 strings.
//         //202.25 ns per iter at vlen = 16, dayum .
//         //48.5ns comparing 'aaaa' against 1024 strings. LOL. LMAO even.
//         #[bench]
//         fn compare_string_lt_bench(bencher : &mut Bencher)
//         {
//             let n_strings = 1024;
//             let s_pool = IStringPool::from_vec(random_unique_strings(n_strings,16));
//             let mut accumulator = 0;
//             let mut idx = 0;
//             bencher.iter(||
//             {
//                 let _needle = &s_pool.strings[idx];
//                 accumulator += compare_string_lt::<16>("zzzz", &s_pool);
//                 idx = (idx + 1) % n_strings as usize;
//             });
//         }

//         //2053 ns/iter !
//         //1390 with the SIMD Compare strings
//         #[bench]
//         fn compare_string_lt_cntrl(bencher : &mut Bencher)
//         {
//             let n_strings = 1024;
//             let s_pool = random_unique_strings(n_strings,16);
//             let mut accumulator = 0;
//             let mut idx = 0;
//             bencher.iter(||
//             {
//                 let needle = &s_pool[idx];
//                 accumulator += s_pool.iter().filter(|e| compare_strings::<8>(e, needle).is_lt()).count();
//                 idx = (idx + 1) % n_strings as usize;
//             });
//         }

//         #[test]
//         fn compare_string_lt_test()
//         {
//             let n_strings = 1024;
//             let mut s_pool = IStringPool::from_vec(random_unique_strings(n_strings,16));
//             let mut accumulator = 0;
//             let idx = 0;
//             let needle = &s_pool.strings[idx];
//             accumulator += compare_string_lt::<8>(needle, &s_pool);
//             //not read idx = (idx + 1) % n_strings as usize;
//             println!("NEEDLE : {}", needle);
//             s_pool.strings.sort();
//             println!("{}, {}" , s_pool.strings[accumulator as usize - 1], s_pool.strings[accumulator as usize + 1]);
//         }

//         mod completed
//         {
//             use super::*;
//             //440ns
//             #[bench]
//             fn compare_ints_control(bencher : &mut Bencher)
//             {
//                 let a = random_numbers(1024);
//                 let b = random_numbers(1024);
//                 let mut accumulator = (0,0,0);
//                 bencher.iter(|| {
//                     let x = compare_arrays_control(&a[0..1024].try_into().unwrap(),
//                         &b[0..1024].try_into().unwrap());
//                     accumulator.0 += x.0;
//                 });
//             }

//             //v2 : 92ns, wow.
//             //v3 : 92ns, o well. 91 inlined.
//             //v4 : 37ns, using lane width 32! dayum.
//             #[bench]
//             fn compare_ints_simd(bencher : &mut Bencher)
//             {
//                 let a = random_numbers(1024);
//                 let b = random_numbers(1024);
//                 let mut accumulator = 0;

//                 bencher.iter(|| {
//                     accumulator += compare_arrays4::<64>(&a[0..1024].try_into().unwrap(),
//                         &b[0..1024].try_into().unwrap()).0;
//                 });
//             }

//             //91 ns/ iter, strlen of 16
//             //108ns / iter, strlen of 64
//             #[bench]
//             fn compare_strings_control(bencher : &mut Bencher)
//             {
//                 let strings = random_unique_strings(1024,64);
//                 let mut accumulator = 0;
//                 bencher.iter(||
//                 {
//                     for chunk in strings.chunks_exact(2)
//                     {
//                         if let Ordering::Less = chunk[0].cmp(&chunk[1])
//                         {
//                             accumulator += 1;
//                         }
//                     }
//                 });
//             }

//             // VLEN of 8 , strlen of 16 : 64ns per iter
//             // VLEN of 16, strlen of 16 : 55ns per iter
//             // VLEN of 16, strlen of 64 : 49ns per iter
//             // VLEN of 32, strlen of 64 : 50ns per iter.
//             #[bench]
//             fn compare_strings_simd(bencher : &mut Bencher)
//             {
//                 let strings = random_unique_strings(1024,64);
//                 let mut accumulator = 0;
//                 bencher.iter(||
//                 {
//                     for chunk in strings.chunks_exact(2)
//                     {
//                         if let Ordering::Less = compare_strings::<32>(&chunk[0], &chunk[1])
//                         {
//                             accumulator += 1;
//                         }
//                     }
//                 });
//             }

//         }

//     }
// }
