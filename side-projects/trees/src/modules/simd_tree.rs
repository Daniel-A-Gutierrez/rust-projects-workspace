use anyhow::{bail, Result};
use rayon::prelude::*;
use std::simd::cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd};
use std::simd::{prelude::*, LaneCount, Mask, SimdElement, SupportedLaneCount};
//#![feature(test)]
#[cfg(test)]
mod test
{
    extern crate test;
    use super::*;
    use test::Bencher;

    mod tests
    {
        use super::*;

        #[test]
        fn s_tree_negative()
        {
            let data = [0, 0, 1, 2, 3, 8, 7, 10, 6, 5, 14, 14, 11, 13, 3, 3];
            let mut evens: Vec<_> = data.map(|e| e * 2).iter().cloned().collect();
            evens.sort_by(|a, b| b.cmp(a));
            let mut odds: Vec<_> = data.map(|e| e * 2 + 1).iter().cloned().collect();
            let evens_tree = RevSTree::<4>::from_unsorted(&evens).unwrap();
            let odds_tree = RevSTree::<4>::from_unsorted(&odds).unwrap();
            odds.sort_by(|a, b| b.cmp(a));
            println!("{:?}", (0..odds.len()).into_iter().collect::<Vec<usize>>());
            println!("{:?}", evens);
            println!("{:?}", odds);
            println!("index\titem\tfirst\tlast\trange");
            for i in 0..data.len()
            {
                let d = &odds[i];
                println!("{}\t{}\t{:?}\t{:?}\t{:?}",
                         i,
                         d,
                         evens_tree.find_first(d),
                         evens_tree.find_last(d),
                         evens_tree.find_range(d));
            }
            println!();
            println!("{:?}", (0..odds.len()).into_iter().collect::<Vec<usize>>());
            println!("{:?}", odds);
            println!("{:?}", evens);
            println!("index\titem\tfirst\tlast\trange");
            for i in 0..data.len()
            {
                let d = &evens[i];
                println!("{}\t{}\t{:?}\t{:?}\t{:?}",
                         i,
                         d,
                         odds_tree.find_first(d),
                         odds_tree.find_last(d),
                         odds_tree.find_range(d));
            }
        }

        #[test]
        fn s_tree_basic()
        {
            let mut data = [0, 1, 4, 2, 3, 8, 7, 10, 6, 5, 14, 14, 11, 13, 3, 3];
            let tree = RevSTree::<4>::from_unsorted2(&data).unwrap();
            data.sort_by(|a, b| b.cmp(a));
            println!("{:?}", (0..data.len()).into_iter().collect::<Vec<usize>>());
            println!("{:?}", data);
            println!("index\titem\tfirst\tlast\trange");
            for i in 0..data.len()
            {
                let d = &data[i];
                println!("{}\t{}\t{:?}\t{:?}\t{:?}",
                         i,
                         d,
                         tree.find_first(d),
                         tree.find_last(d),
                         tree.find_range(d));
            }
        }
    }

    mod benchmarks
    {
        use super::*;
        use sorted_vec::SortedVec;

        // find range perf :
        // 38us - epow 28, degree 16. //24 us with simd.
        // 8.2us - epow 24, degree 16. //7 us with simd.
        // 5.5us - epow 20, degree 16. 5.291 with find last. sped up to 4us with simd.
        // 3.2us - epow 16, degree 4. with b search on find last, its 1.55us . 1.6us with simd.
        // high degrees do better at larger sizes, small powers at lower sizes.
        #[bench]
        fn find_first_1kx64k(b: &mut Bencher)
        {
            let epow = 16;
            let lpow = 7;
            let scale = 2u32.pow(epow - lpow);
            let v = (0..2u32.pow(epow)).rev().collect::<Vec<u32>>();
            let tree = RevSTree::<4>::from_unsorted(&v).unwrap();
            b.iter(|| {
                 for i in 0..2u32.pow(lpow)
                 {
                     tree.find_last(&(i * scale));
                 }
             });
        }

        // 21 us - epow 28
        // 7.9us - epow 24
        // 4.8us - epow 20
        // 3.1us - epow 16
        #[bench]
        fn btree_search_1kx64k(b: &mut Bencher)
        {
            let epow = 16;
            let lpow = 7;
            let v = (0..2u32.pow(epow)).rev().collect::<Vec<u32>>();
            let tree = std::collections::BTreeSet::from_iter(v.iter());
            let scale = 2u32.pow(epow - lpow);
            b.iter(|| {
                 for i in 0..2u32.pow(lpow)
                 {
                     let r = i * scale;
                     let y = tree.range(r..r);
                 }
             });
        }

        //90 us (epow : 28)
        //14.3us - epow 24
        //6.6us - epow 20
        //1.9us - epow 16
        #[bench]
        fn svec_search_1kx64k(b: &mut Bencher)
        {
            let epow = 16;
            let lpow = 7;
            let v = (0..2u32.pow(epow)).collect::<Vec<u32>>();
            let sv = unsafe { SortedVec::from_sorted(v) };
            b.iter(|| {
                 for i in 0..2u32.pow(lpow)
                 {
                     let mut y = sv.binary_search(&(i * 2u32.pow(epow - lpow)));
                 }
             });
        }

        #[bench]
        fn btree_from_iter_64k(b: &mut Bencher)
        {
            let v = (0..2u32.pow(16)).rev().collect::<Vec<u32>>();

            b.iter(|| {
                 let tree = std::collections::BTreeSet::from_iter(v.iter());
             });
        }

        #[bench]
        fn from_sorted_64k(b: &mut Bencher)
        {
            let v = (0..2u32.pow(16)).rev().collect::<Vec<u32>>();
            b.iter(|| {
                 let tree = RevSTree::<16>::from_unsorted2(&v).unwrap();
             });
        }

        #[bench]
        fn from_sorted_64k_control(b: &mut Bencher)
        {
            let mut v = (0..2u32.pow(16)).collect::<Vec<u32>>();
            b.iter(|| {
                 let mut y = v.clone();
                 y.sort_by(|a, b| b.cmp(a));
             });
        }

        #[bench]
        fn control(b: &mut Bencher)
        {
            b.iter(|| (0..(16usize.pow(4))).into_iter().collect::<Vec<usize>>());
        }
    }
}

/// For going from a higher tier in a S+ tree to a lower tier.
/// If going down 1 level, height is 1. If going down 2, height is 2.
/// The function is height recursions of (idx + 1)*Degree, which expands to a polynomial
/// D^H*I + D^H + D^H-1 + D^H-2 ... while H > 0

fn tree_poly_sum(degree: usize, height: usize, idx: usize) -> usize
{
    let mut sum = idx;
    for _ in 0..height
    {
        sum = sum * degree + degree;
    }
    return sum;
}

/// if an element is found in a higher tier of the tree, this calculates its index
/// in the final (bottom) tier of the tree.
fn bottom_index_of(degree: usize, idx: usize, len: usize) -> usize
{
    let mut sum = idx;
    loop
    {
        let nxt = sum * degree + degree;
        if nxt > len
        {
            break;
        }
        else
        {
            sum = nxt;
        }
    }
    return sum;
}

/// calculates the height and length of a sp tree of degree over a sorted array of len.
/// returns err is the len is not a power of degree >= 1.
fn tree_shape(degree: usize, len: usize) -> Result<(usize, usize)>
{
    let mut sum = degree;
    let mut height = 1;
    let mut d = degree;
    while sum < len
    {
        d = d * degree;
        sum += d;
        height += 1;
    }
    if d == len
    {
        return Ok((height, sum));
    }
    else
    {
        bail!("Invalid len for tree, {} is not a power of {}", len, degree);
    }
}

//cant assume items are unique so have to check eq.
// #[inline]
// fn num_lteq<T , const DEGREE : usize>(v : &Simd<T,DEGREE>, element : &T) -> usize
// where LaneCount<DEGREE>: SupportedLaneCount,
// T : SimdElement, //hate that it has to be uint.
// Simd<T,DEGREE> : SimdOrd + SimdUint
// {
//     let comparand = Simd::splat(*element);
//     let ones = Simd::<T,DEGREE>::splat();
//     let r =  v.saturating_sub(comparand).simd_min(ones).reduce_sum();
// }

#[inline]
fn num_lteq<const DEGREE: usize>(v: &Simd<u32, DEGREE>, element: u32) -> usize
    where LaneCount<DEGREE>: SupportedLaneCount
{
    let comparand = Simd::splat(element);
    let ones = Simd::splat(1u32);
    let r = (v).saturating_sub(comparand).simd_min(ones).reduce_sum() as usize;
    return DEGREE - r;
}

#[inline]
fn num_eq<const DEGREE: usize, T: Ord>(slice: &[T; DEGREE], element: &T) -> usize
{
    let mut eq = 0;
    for i in 0..DEGREE
    {
        if slice[i] == *element
        {
            eq += 1;
        }
    }
    return eq;
}

#[inline]
fn num_lt<const DEGREE: usize, T: Ord>(slice: &[T; DEGREE], element: &T) -> usize
{
    let mut lt = 0;
    for i in 0..DEGREE
    {
        if slice[i] < *element
        {
            lt += 1;
        }
    }
    return lt;
}

#[derive(Debug)]
struct RevSTree<const DEGREE: usize>
{
    keys:   Vec<u32>,
    len:    usize,
    height: usize,
}

impl<const DEGREE: usize> RevSTree<DEGREE>
    where LaneCount<DEGREE>: SupportedLaneCount //where u32 : Clone + Eq + Ord + Sized + Sync + Send
{
    fn from_unsorted(src: &[u32]) -> Result<Self>
    {
        let (height, len) = tree_shape(DEGREE, src.len())?;
        let mut tree = Vec::<u32>::with_capacity(len);
        tree.extend_from_slice(src);
        //reverse sort
        if src.len() > 1023
        {
            tree.par_sort_by(|a, b| b.cmp(a));
        }
        else
        {
            tree.sort_by(|a, b| b.cmp(a));
        }
        let mut tier_len = src.len() / DEGREE;
        let mut step_by = DEGREE;
        for _ in 1..height
        //could be improved by just going over previous tier.
        {
            let mut s = step_by - 1;
            for _ in (0..tier_len)
            {
                tree.push(tree[s].clone());
                s += step_by;
            }
            step_by *= DEGREE;
            tier_len /= DEGREE;
        }

        return Ok(RevSTree { keys: tree,
                             len: src.len(),
                             height });
    }

    fn from_unsorted2(src: &[u32]) -> Result<Self>
    {
        let (height, len) = tree_shape(DEGREE, src.len())?;
        let mut tree = Vec::<u32>::with_capacity(len);
        tree.extend_from_slice(src);
        //reverse sort
        if src.len() > 1023
        {
            tree.par_sort_by(|a, b| b.cmp(a));
        }
        else
        {
            tree.sort_by(|a, b| b.cmp(a));
        }
        let mut last_tier_start = 0;
        for _ in 1..height
        {
            let t = tree.len();
            for s in (last_tier_start + DEGREE - 1..tree.len()).step_by(DEGREE)
            {
                tree.push(tree[s].clone());
            }
            last_tier_start = t;
        }

        return Ok(RevSTree { keys: tree,
                             len: src.len(),
                             height });
    }

    //lt gets me the lower bound (inexact), lteq the upper (exact).
    fn find_last(&self, key: &u32) -> Result<usize, usize>
    {
        // if(self.len < 65537)
        // {
        //     let mut i = self.keys[0..self.len].binary_search(&key)?;
        //     while i > 0
        //     {
        //         if self.keys[i-1] != *key {return Ok(i);}
        //         i-=1;
        //     }
        //     return Ok(0);
        // }
        //bounds check so we dont have to do it in the loop
        let min = &self.keys[self.len - 1];
        if (key == min)
        {
            return Ok(self.len - 1);
        }
        if (key < min)
        {
            return Err(self.len - 1);
        }

        let mut ridx = 0; // reversed chunk index. 0 = [last DEGREE elements].
        for _ in 1..self.height
        {
            let idx = self.keys.len() - DEGREE * ridx;
            let node: &[u32; DEGREE] = (&self.keys[idx - DEGREE..idx]).try_into().unwrap();
            let lteq = num_lteq::<DEGREE>(&Simd::from_slice(node), *key);
            ridx = ridx * DEGREE + lteq;
        }

        let idx = self.keys.len() - DEGREE * ridx;
        for i in (idx - DEGREE..idx)
        {
            if (self.keys[i] == *key)
            {
                return Ok(i);
            }
            if (self.keys[i] < *key)
            {
                return Err(i);
            }
        }
        return Err(idx); // should never happen.
    }

    //lt gets me the lower bound (inexact)
    fn find_first(&self, key: &u32) -> Result<usize, usize>
    {
        //bounds check so we dont have to do it in the loop
        let min = &self.keys[self.len - 1];
        if (key == min)
        {
            return Ok(self.len - 1);
        }
        if (key < min)
        {
            return Err(self.len - 1);
        }

        let mut ridx = 0; // reversed chunk index. 0 = [last DEGREE elements].
        for _ in 1..self.height
        {
            let idx = self.keys.len() - DEGREE * ridx;
            let node = &self.keys[idx - DEGREE..idx];
            let lt = num_lt::<DEGREE, _>(node.try_into().unwrap(), key);
            ridx = ridx * DEGREE + lt;
        }

        let idx = self.keys.len() - DEGREE * ridx;
        let scan = (idx - ((DEGREE) * 2).min(idx)); //prevent uint overflow subtraction
        for i in (scan..idx).rev()
        {
            if (self.keys[i] == *key)
            {
                return Ok(i);
            }
            if (self.keys[i] > *key)
            {
                return Err(i + 1);
            }
        }
        return Err(idx - DEGREE);
    }

    //finds the inclusive range of indeces of elements gteq to lower bound and lteq to upper bound.
    /// Panics if lower bound > upper bound .
    #[rustfmt::skip]
    fn find_range(&self, key : &u32) -> (usize, usize)
    {
        //bounds check so we dont have to do it in the loop
        let min = &self.keys[self.len - 1];
        if (key < min) {return (0,0);}
        
        let mut u_ridx = 0; // reversed chunk index. 0 = [last DEGREE elements].
        let mut l_ridx = 0;
        let mut l_idx;
        let mut u_idx;
        
        //special case
        if key == min 
        {
            l_idx = self.len - 1;
            for _ in 1..self.height
            {
                let idx = self.keys.len() - DEGREE * u_ridx;
                let node = (&self.keys[idx-DEGREE..idx]).try_into().unwrap();
                let eq = num_eq::<DEGREE,u32>(node, key);
                u_ridx = u_ridx * DEGREE + eq;
            }
        }

        else 
        {
            for _ in 1..self.height
            {
                if (u_ridx == l_ridx)
                {
                    let idx = self.keys.len() - DEGREE * u_ridx;
                    let node = (&self.keys[idx-DEGREE..idx]).try_into().unwrap();
                    let lt = num_lt::<DEGREE,u32>(node, key);
                    let eq = num_eq::<DEGREE,u32>(node, key);
                    l_ridx = l_ridx * DEGREE + lt;
                    u_ridx = l_ridx + eq;
                }
                else 
                {
                    let l_idx = self.keys.len() - DEGREE * l_ridx;
                    let l_node = (&self.keys[l_idx-DEGREE..l_idx]).try_into().unwrap();
                    let lt = num_lt::<DEGREE,u32>(l_node, key);
                    l_ridx = l_ridx * DEGREE + lt;
                    
                    let u_idx = self.keys.len() - DEGREE * u_ridx;

                    let u_node : &[u32;DEGREE] = (&self.keys[u_idx-DEGREE..u_idx]).try_into().unwrap();
                    let lteq = num_lteq::<DEGREE>(&Simd::from_slice(u_node), *key); 

                    u_ridx = u_ridx * DEGREE + lteq;   
                }
            }
            l_idx = self.keys.len() - DEGREE * l_ridx;
            let scan = (l_idx - ((DEGREE)*2).min(l_idx)); //prevent uint overflow subtraction
            for i in (scan..l_idx).rev()
            {
                l_idx = i; 
                if (self.keys[i] == *key) { break }
                if (self.keys[i] >  *key) { l_idx = i+1; break }
            };
        }

        u_idx = self.keys.len() - DEGREE * u_ridx;
        for i in (u_idx - DEGREE .. u_idx)
        {
            if (self.keys[i] == *key) { u_idx = i;  break}
            if (self.keys[i] <  *key) { u_idx = i;  break}
        }
        return (l_idx, u_idx);
    }
}

// fn gen_ordering<const DEGREE : usize>(len : usize) -> Result<Vec<usize>>
// {
//     let (height, tree_len) = tree_shape(DEGREE, len)?;
//     let mut tree = vec![0;tree_len];
//     let mut step_by = 1;
//     let mut start = tree_len;
//     let mut tier_len = len;
//     for _ in 0..height
//     {
//         start -= tier_len;
//         //let dst_iter = (&mut tree[start..start+tier_len]).iter_mut();
//         //77us, not fast
//         // let src_iter = (0..len).into_iter().step_by(step_by);
//         // dst_iter.zip(src_iter).for_each(|(dst,src)| *dst = src);

//         // 21us, about as 5x the control
//         //dst_iter.enumerate().for_each(|(i,dst)| *dst = i*step_by);

//         // 22 us.
//         // let dst = tree[start..start + tier_len].chunks_exact_mut(DEGREE).enumerate();
//         // for (i,chunk) in dst
//         // {
//         //     for d in 0..DEGREE
//         //     {
//         //         let idx = d + i*DEGREE;
//         //         chunk[d] = idx * step_by;
//         //     }
//         // }

//         // 18.1 us, or 10.7us by the eytzinger module's standards. So ill call this a win.
//         let dst = &mut tree[start..start + tier_len];
//         let mut s = 0;
//         for i in (0..tier_len)
//         {
//             dst[i] = s;
//             s += step_by;
//         }

//         //25 us.
//         //dst_iter.zip(src_iter).array_chunks::<16>().for_each(|(data)| for (dst,src) in data  { *dst = src ;});
//         step_by *= DEGREE;
//         tier_len /= DEGREE;

//     }
//     return Ok(tree);
// }

// BELOW HERE IS OLD SHIT
// enum Item
// {
//     Slf(u32),
//     Edge(u32),
// }

// struct TreeIter
// {
//     tree:  Vec<u32>,  //must assume len is a power of 2 -1. for now.
//     stack: Vec<Item>, //for iterating over the edges in the tree.
// }

// impl TreeIter
// {
//     fn new(tree: Vec<u32>) -> Self
//     {
//         let mut stack = vec![];
//         tree.get(2).and_then(|e| Some(stack.push(Item::Edge(*e))));
//         if tree.len() == 0
//         {
//             stack.clear();
//         }
//         return TreeIter { tree, stack };
//     }

//     ///i think the trick is gonna be to iterate over the edges.
//     /// an edge is defined as (node, (is left))
//     /// we pop a edge off the stack, visit that node, push its kids onto the stack, then loop
//     /// if there are no kids, we return that edge's node...
//     fn next(&mut self)
//     {
//         //let node = self.stack.peek();
//         //while has left
//         //go left
//         //return left

//         //return self

//         //if has right
//         //return right
//     }
// }

/*
I wanna see if

*/

// const fn items_per_cache_line<T : Sized>() -> usize
// {
//     return 64/size_of::<T>();
// }

// fn make_sized_array<T : Sized, const N : usize>(items : &[T]) -> [T;N]
// {
//     todo!();
// }

// fn make_sized_array_helper<T: Sized, const N : usize>(items : &[T]) -> [T;N]
// {
//     let C = items_per_cache_line::<T>;
//     return make_sized_array::<T,C>(items);
// }

// struct CacheLineArray<T : Sized, const N : usize>([T;N]);

//this seems like a problem for macros.each node is a generic array of at least 2 items T.
//the macro does the size_of and chooses an appropriate generic size for the nodes.
