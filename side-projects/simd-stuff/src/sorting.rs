#![allow(unused)]

use std::{array::from_fn, cmp::Ordering, ops::{RemAssign, Shl}, simd::{prelude::*, LaneCount, SupportedLaneCount}};
use rand::Rng;
use rayon::slice::ParallelSliceMut;
use rand::distr::uniform::Uniform;
use typenum::{Const, Double, UInt, Unsigned};
use generic_array::{ArrayLength, IntoArrayLength, ConstArrayLength,  GenericArray as Array};
use random_data::random_numbers;

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
            let mut sample : [i32 ; 32] = random_numbers(32,0,1000).try_into().unwrap();
            bubble_sort_vec(&mut sample);
            assert!(sample.is_sorted(), "bubble sort vec doesn't sort!");
        }

        // about 700ns : ( . Need shuffle_dyn for this to be decent i bet. 
        // 300ns after optimizing. not even close still. 
        // 86ns after using min max !
        #[bench]
        fn bubble_sort_vec_b(bencher : &mut Bencher)
        {
            let mut sample = random_numbers(32,0,1000);
            //sample.sort();
            bencher.iter(|| 
            {
                let mut src = sample.clone();
                bubble_sort_vec(&mut src.try_into().unwrap());
            });
        }
        
        //about 200ns
        // 150-200 ns per iter.
        #[bench]
        fn bubble_sort_b(bencher : &mut Bencher)
        {
            let mut sample = random_numbers(32,0,1000);
            //sample.sort();
            bencher.iter(|| 
            {
                let mut src = sample.clone();
                bubble_sort(&mut src);
            });
        }

        #[bench]
        fn control(bencher : &mut Bencher)
        {
            let sample = random_numbers(32,0,1000);
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

const fn alternating<const LEN : usize>() -> [bool ; LEN]
where LaneCount<LEN> : SupportedLaneCount, 
{
    let mut r = [false; LEN];
    let mut i = 0;
    loop 
    {
        if i > LEN - 2 { break }
        r[i] = true;
        i+=2;
    }
    return r;
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
const fn swapped_evens<const LEN : usize>() -> [i32 ; LEN]
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
const fn swapped_odds<const LEN : usize>() -> [i32 ; LEN]
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
const fn idxs<const LEN : usize>() -> [i32; LEN]
where LaneCount<LEN> : SupportedLaneCount, 

{
    let mut r = [0i32;LEN];
    let mut i = 0;
    loop
    {
        r[i as usize] = i;
        i += 1;
        if i > LEN as i32 - 1 {break}
    }
    return r;
}

//let gt_m = simd_swizzle!(src.simd_gt(sl).to_int() , evens()); 
// ^^ doesnt work because const generics are still too limited.
//so i have to use a fixed size of 32 to test swizzles effect on perf. 
#[inline]
fn bubble_sort_vec(arr : &mut[i32; 32]) 
{
    // let swapped_evens = swapped_evens::<32>();
    // let swapped_odds = Simd::from_array(swapped_odds::<32>());    //let idxs = Simd::from_array(idxs());
    // let vidxs = Simd::from_array(idxs::<32>());
    // const EVENS32 : [usize ; 32] = evens::<32>();
    // const ODDS32 : [usize ; 32] = odds::<32>();
    
    //let alt = Mask::from_array(alternating());//1 0 1 0 1 0 1 0
    // let nalt = alt.shift_elements_right::<1>(false);
    // let mut filter = &alt;
    //let mut scatter_mask = [0i32; 32];
    let mut b = true;
    let mut src = Simd::from_array(*arr);
    const even_swap : [usize ; 32] = [0,33,2,35,4,37,6,39,8,41,10,43,12,45,14,47,16,49,18,51,20,53,22,55,24,57,26,59,28,61,30,63];
    //const odd_swap : [usize; 32] =
    loop 
    {
        let sl = src.shift_elements_left::<1>(i32::MAX);
        let sr = src.shift_elements_right::<1>(i32::MIN);
        let swap_mask = src.simd_gt(sl);
        if (!swap_mask.any()) {break;}
        let left = src.simd_min(sl);
        let right = src.simd_max(sr);
        
        if(b) {
            src = simd_swizzle!(left,right,even_swap);}
        else {
            src = simd_swizzle!(right,left,even_swap);
        }
    //     let gt_mask = filter.select_mask(swap_mask, *filter); // basically an AND
    //     unsafe {sr.store_select_unchecked(arr, gt_mask.shift_elements_right::<1>(!b));}
    //     unsafe {sl.store_select_unchecked(arr, gt_mask);}
        b = !b;
    //     filter = if(b) {&alt} else {&nalt};
    // }
    }
    src.copy_to_slice(arr);
    
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