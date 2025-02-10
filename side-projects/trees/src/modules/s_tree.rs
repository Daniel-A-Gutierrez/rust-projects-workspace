
use anyhow::{bail, Result};
use rayon::prelude::*;
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
        fn s_tree_new()
        {
            let data = [0,1,4,2,3,8,7,10,6,5,14,14,11,13,3,3];
            let tree = RevSTree::<_,4>::from_unsorted(&data).unwrap();
            println!("{:?}", tree);
            let idx_3 = tree.find_first(&12);
            println!("{:?}", idx_3);
            println!("{:?}", tree.find_last(&12));
        }

        #[test]
        fn poly_sum_test()
        {
            for i in 0..6
            {
                println!("{}", tree_poly_sum(4, i, 3))
            }
        }

        #[test]
        fn par_gen_test() -> Result<()>
        {
            println!("{:?}", par_gen_ordering::<4>(64)?);
            return Ok(());
        }

        #[test]
        fn gen_test() -> Result<()>
        {
            println!("{:?}", gen_ordering(64,4)?);
            return Ok(());
        }

        #[test]
        fn gen_rev_test() -> Result<()>
        {
            println!("{:?}", gen_rev_ordering(64,4)?);
            return Ok(());
        }
    }

    mod benchmarks 
    {
        use super::*;
    
        #[bench]
        fn par_gen_tree_bench(b : &mut Bencher)
        {
            b.iter( || { 
                for i in 1..8
                {
                    par_gen_ordering::<4>( 4usize.pow(i) ).unwrap();
                }
            });
        }

        #[bench]
        fn gen_tree_bench(b : &mut Bencher)
        {
            b.iter( || { 
                    gen_ordering( 16usize.pow(4), 16 ).unwrap();
                
            });
        }

        #[bench]
        fn control(b : &mut Bencher)
        {
            b.iter(|| (0..(16usize.pow(4))).into_iter().collect::<Vec<usize>>());
        }
    }
}

/// For going from a higher tier in a S+ tree to a lower tier.
/// If going down 1 level, height is 1. If going down 2, height is 2.
/// The function is height recursions of (idx + 1)*Degree, which expands to a polynomial
/// D^H*I + D^H + D^H-1 + D^H-2 ... while H > 0

fn tree_poly_sum(degree : usize, height : usize, idx : usize) -> usize
{
    let mut sum = idx;
    for _ in 0..height 
    {
        sum = sum*degree + degree;
    }
    return sum;
}

/// if an element is found in a higher tier of the tree, this calculates its index
/// in the final (bottom) tier of the tree.
fn bottom_index_of(degree : usize, idx : usize, len : usize) -> usize 
{
    let mut sum = idx;
    loop  
    {
        let nxt = sum * degree + degree;
        if nxt > len {break} else {sum = nxt;}
    }
    return sum;
}

/// calculates the height and length of a sp tree of degree over a sorted array of len.
/// returns err is the len is not a power of degree >= 1. 
fn tree_shape(degree : usize, len : usize) -> Result<(usize,usize)>
{
    let mut sum = degree;
    let mut height = 1;
    let mut d = degree;
    while sum < len
    {
        d = d*degree;
        sum += d;
        height += 1;
    }
    if d == len { return Ok( (height, sum) )}
    else { bail!("Invalid len for tree, {} is not a power of {}", len, degree); }
}

// waaay slower lol
fn par_gen_ordering<const DEGREE : usize>(len : usize) -> Result<Vec<usize>>
{
    let (height, tree_len) = tree_shape(DEGREE, len)?;
    let mut tree = vec![0;tree_len];
    let mut step_by = 1;
    let mut start = tree_len;
    let mut tier_len = len;
    for _ in 0..height 
    {
        start -= tier_len;
        let dst_iter = (&mut tree[start..start+tier_len]).par_iter_mut();
        let src_iter = (0..len).into_par_iter().step_by(step_by);
        dst_iter.zip_eq(src_iter).for_each(|(dst,src)| *dst = src);
        step_by *= DEGREE;
        tier_len /= DEGREE;
        
    }
    return Ok(tree);
}

fn gen_ordering(len : usize, degree : usize) -> Result<Vec<usize>>
{
    let (height, tree_len) = tree_shape(degree, len)?;
    let mut tree = vec![0;tree_len];
    let mut step_by = 1;
    let mut start = tree_len;
    let mut tier_len = len;
    for _ in 0..height 
    {
        start -= tier_len;
        let dst = &mut tree[start..start + tier_len];
        let mut s = 0;
        for i in (0..tier_len)
        {
            dst[i] = s;
            s += step_by;
        }
        step_by *= degree;
        tier_len /= degree;
        
    }
    return Ok(tree);
}

// orders like ' 1 2 3 4 5 6 7 8, 1 3 5 7, 1 5 ' . Should be easier to build from unsorted data.
fn gen_rev_ordering(len : usize, degree : usize) -> Result<Vec<usize>>
{
    let (height, tree_len) = tree_shape(degree, len)?;
    let mut tree = vec![0;tree_len];
    let mut step_by = 1;
    let mut start = 0;
    let mut tier_len = len;
    for _ in 0..height 
    {
        let dst = &mut tree[start..start + tier_len];
        let mut s = len;
        for i in (0..tier_len)
        {
            s -= step_by;
            dst[i] = s;
        }
        start += tier_len;
        step_by *= degree;
        tier_len /= degree;
    }
    return Ok(tree);
}


//cant assume items are unique so have to check eq.
#[inline]
fn num_lteq<const DEGREE : usize, T : Ord>(slice : &[T;DEGREE], element : &T) -> usize
{
    let mut lteq = 0;
    for i in 0..DEGREE
    {
        if slice[i] <= *element {lteq += 1;}
    }
    return lteq;
}

#[inline]
fn num_eq<const DEGREE : usize, T : Ord>(slice : &[T;DEGREE], element : &T) -> usize
{
    let mut eq = 0;
    for i in 0..DEGREE
    {
        if slice[i] == *element {eq += 1;}
    }
    return eq;
}

#[inline]
fn num_lt<const DEGREE : usize, T : Ord>(slice : &[T;DEGREE], element : &T) -> usize
{
    let mut lt = 0;
    for i in 0..DEGREE
    {
        if slice[i] < *element {lt += 1;}
    }
    return lt;
}


#[derive(Debug)]
struct RevSTree<KEY : Clone + Eq + Ord + Sized , const DEGREE : usize>
{
    keys : Vec<KEY>,
    len : usize,
    height : usize,
}

impl<const DEGREE : usize, KEY > RevSTree<KEY,DEGREE>
where KEY : Clone + Eq + Ord + Sized
{
    fn from_unsorted(src : &[KEY]) -> Result<Self>
    {
        let (height,len) = tree_shape(DEGREE, src.len())?;
        let mut tree = Vec::<KEY>::with_capacity(len);
        tree.extend_from_slice(src);
        //reverse sort
        tree.sort_by(|a,b| b.cmp(a));

        let mut tier_len = src.len()/DEGREE;
        let mut step_by = DEGREE;
        for _ in 1..height 
        {
            let mut s = DEGREE-1;
            for _ in (0..tier_len)
            {
                tree.push(tree[s].clone());
                s += step_by;
            }
            step_by *= DEGREE;
            tier_len /= DEGREE;
        }

        return Ok(RevSTree{keys : tree, len : src.len(), height});
    }

    //lt gets me the lower bound (inexact), lteq the upper (exact). 
    fn find_last(&self, key : &KEY) -> Result<usize, usize>
    {
        //bounds check so we dont have to do it in the loop
        let min = &self.keys[self.len - 1];
        if (key < min) {return Err(self.len - 1);}
        
        let mut ridx = 0; // reversed chunk index. 0 = [last DEGREE elements].
        for _ in 1..self.height
        {
            let idx = self.keys.len() - DEGREE * ridx;
            let node = &self.keys[idx-DEGREE..idx];
            let lteq = num_lteq::<DEGREE,_>(node.try_into().unwrap(), key);            
            ridx = ridx * DEGREE + lteq;
        }

        let idx = self.keys.len() - DEGREE * ridx;
        for i in (idx - DEGREE .. idx)
        {
            if (self.keys[i] == *key) {return Ok(i);}
            if (self.keys[i] < *key) {return Err(i);}
        }
        return Err(idx); // should never happen. 
    }

    //lt gets me the lower bound (inexact) 
    fn find_first(&self, key : &KEY) -> Result<usize, usize>
    {
        //bounds check so we dont have to do it in the loop
        let min = &self.keys[self.len - 1];
        if (key < min) {return Err(self.len - 1);}
        
        let mut ridx = 0; // reversed chunk index. 0 = [last DEGREE elements].
        for _ in 1..self.height
        {
            let idx = self.keys.len() - DEGREE * ridx;
            let node = &self.keys[idx-DEGREE..idx];
            let lt = num_lteq::<DEGREE,_>(node.try_into().unwrap(), key);            
            ridx = ridx * DEGREE + lt;
        }

        let idx = self.keys.len() - DEGREE * ridx;
        let scan = (idx + DEGREE).min(self.len); //prevent uint overflow subtraction
        for i in ((idx-DEGREE)..(scan)).rev()
        {
            if (self.keys[i] == *key) {return Ok(i);}
            if (self.keys[i] > *key) {return Err(i+1);}
        }
        return Err(idx-DEGREE);
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
