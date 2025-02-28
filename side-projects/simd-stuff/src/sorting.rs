#![allow(unused)]

use std::{array::from_fn, cmp::Ordering, ops::{RemAssign, Shl}, simd::{prelude::*, LaneCount, SupportedLaneCount}};
use rand::Rng;
use rayon::slice::ParallelSliceMut;
use rand::distr::uniform::Uniform;
use typenum::{Const, Double, UInt, Unsigned};
use generic_array::{ArrayLength, IntoArrayLength, ConstArrayLength,  GenericArray as Array};
use crate::random_numbers;

#[cfg(test)]
mod test
{
    extern crate test;
    use super::*;
    use test::Bencher;

    mod benchmarks
    {
        use super::*;

        #[test]
        fn bubble_sort_vec_t()
        {
            let mut sample : [i32 ; 32] = random_numbers(32).try_into().unwrap();
            bubble_sort_vec::<32>(&mut sample);
            assert!(sample.is_sorted(), "bubble sort vec doesn't sort!");
        }

        #[bench]
        fn bubble_sort_vec_b(bencher : &mut Bencher)
        {
            let sample = random_numbers(32);
            bencher.iter(|| 
            {
                let mut src = sample.clone();
                bubble_sort_vec::<32>(&mut src.try_into().unwrap());
            });
        }
        
        #[bench]
        fn bubble_sort_b(bencher : &mut Bencher)
        {
            let sample = random_numbers(32);
            bencher.iter(|| 
            {
                let mut src = sample.clone();
                bubble_sort(&mut src);
            });
        }

        #[bench]
        fn control(bencher : &mut Bencher)
        {
            let sample = random_numbers(16);
            bencher.iter(|| 
            {
                let mut src = sample.clone();
            });
        }
    }
}

const fn dbl(x : usize)->usize {2*x}
const fn half(x : usize) -> usize {x/2}

// generic_const_exprs is apparently experimental so watching out for bugs here...
//fn merge_sorted<const VLEN : usize> (a : [i32;VLEN]) -> [i32; dbl(VLEN)]

///merge together 2 arrays that are half-vec in size
fn merge_sorted_vecs<const VLEN: usize>(a : &mut [i32; half(VLEN)], b : &mut [i32; half(VLEN)])
where LaneCount<VLEN> : SupportedLaneCount, 
      LaneCount<{half(VLEN)}> : SupportedLaneCount
{
    let mut va : Simd<i32, {half(VLEN)}> = Simd::from_array(*a);
    let mut vb : Simd<i32, {half(VLEN)}> = Simd::from_array(*b);
    let (va, vb) = va.interleave(vb);
    let mut v = [0; VLEN];
    va.copy_to_slice(&mut v[0..VLEN/2]);
    vb.copy_to_slice(&mut v[VLEN/2..]);
    let mut v = Simd::from_array(v);
    let mut vr = v.rotate_elements_left::<1>();
    let mut m = v.simd_gt(vr);
    m.set(VLEN-1,false);
    while m.any()
    {
        //v.scatter(v, idxs);
    }

    todo!();
}

fn network_sort() {}

fn bubble_merge()
{

}

///0, 0, 2, 2, 4, 4, ...
const fn evens<const LEN : usize>() -> [usize ; LEN]
where LaneCount<LEN> : SupportedLaneCount, 
{
    let mut r = [0; LEN];
    let mut i = 0;
    loop 
    {
        if i > LEN - 2 { break }
        r[i+1] = i;
        r[i] = i;
        i+=2;
    }
    return r;
}

///1, 1, 3, 3, 5, 5, ...
const fn odds<const LEN : usize>() -> [usize ; LEN]
where LaneCount<LEN> : SupportedLaneCount, 
{
    let mut r = [0; LEN];
    let mut i = 1;
    loop 
    {
        if i > LEN - 1 { break }
        r[i-1] = i;
        r[i] = i;
        i+=2;
    }
    return r;
}

/// 1, 0, 3, 2, 5, 4 ...
const fn swapped_evens<const LEN : usize>() -> [usize; LEN]
where LaneCount<LEN> : SupportedLaneCount, 
{
    let mut idxs = idxs::<LEN>();
    let mut i = 0;
    loop 
    {
        if i > LEN - 2 {break}
        idxs.swap(i, i+1);
        i+= 2;
    }
    return idxs;
}
///0, 2, 1, 4, 3, ...
const fn swapped_odds<const LEN : usize>() -> [usize; LEN]
where LaneCount<LEN> : SupportedLaneCount, 
{
    let mut idxs = idxs::<LEN>();
    let mut i = 1;
    loop 
    {
        if i > LEN - 2 {break}
        idxs.swap(i, i+1);
        i+= 2;
    }
    return idxs;
}

///0, 1, 2, 3, 4, 5, ...
const fn idxs<const LEN : usize>() -> [usize; LEN]
where LaneCount<LEN> : SupportedLaneCount, 

{
    let mut r = [0;LEN];
    let mut i = 0;
    loop
    {
        r[i] = i;
        i += 1;
        if i > LEN - 1 {break}
    }
    return r;
}

#[inline]
fn bubble_sort_vec<const VLEN : usize>(arr : &mut[i32; VLEN])
where LaneCount<VLEN> : SupportedLaneCount, 
{
    let swapped_evens = Simd::from_array(swapped_evens());
    let swapped_odds = Simd::from_array(swapped_odds());
    //let idxs = Simd::from_array(idxs());
    let evens = Simd::from_array(evens());
    let odds = Simd::from_array(odds());
    let mut scatter_mask = [0i32; VLEN];

    loop 
    {
        let mut src = Simd::from_array(*arr);
        let sl = src.shift_elements_left::<1>(i32::MAX);
        let gt_m = src.simd_gt(sl);
        if (!gt_m.any()) {break;}

        // vperm : "look what they need to mimic a fraction of our power" 
        
        let gtma = gt_m.to_int().to_array();
        let gt_m_e = Mask::from_int(Simd::gather_or(&gtma, evens, src)); //src is unused.
        //let gt_m_e = Mask::from_int(Simd::from_array(scatter_mask));
        
        // this into being required feels bad.
        src.scatter_select(arr, gt_m_e.into(), swapped_evens);
        src = Simd::from_array(*arr);

        let sr = src.shift_elements_left::<1>(i32::MIN);
        let lt_m = src.simd_gt(sr);
        let ltma = lt_m.to_int().to_array();
        //lt_m.to_int().scatter(&mut scatter_mask, odds);
        let lt_m_o = Mask::from_int(Simd::gather_or(&ltma, odds, src));

        src.scatter_select(arr, lt_m_o.into(), swapped_odds);
    }
}

fn bubble_sort(arr : &mut [i32])
{
    let mut unsorted = false;
    if (arr.len() < 2) {return ;}

    for i in 0..(arr.len() - 1)
    {
        if (arr[i] > arr[i+1]){arr.swap(i,i+1); unsorted = true;}
    }

    while unsorted 
    {
        unsorted = false;
        for i in 0..(arr.len() - 1)
        {
            if (arr[i] > arr[i+1]){arr.swap(i,i+1); unsorted = true;}
        }
    }
}

//fn check_sorted_vec<const VLEN : usize>(arr : &)

/*

#[cfg(test)]
mod test
{
    extern crate test;
    use super::*;
    use test::Bencher;

    mod benchmarks
    {
        use super::*;
        #[bench]
        fn bench(bencher : &mut Bencher)
        {
            bencher.iter(|| 
            {
                
            });
        }
    }
}
*/