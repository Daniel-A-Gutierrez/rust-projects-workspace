use array_list;
use radix_trie::Trie;
use radix_trie::TrieCommon;
use radix_trie::TrieKey;
use rudy;
use rudy::rudymap::RudyMap;
use sorted_vec::ReverseSortedVec;
use sorted_vec::SortedVec;
use std::cmp::Reverse;
use std::iter::Rev;
use std::ops::Deref;
use std::ops::DerefMut;
use std::{collections::{BTreeMap, HashMap},
          hash::Hash};

/*
I'll make a table of strings of various lengths, 0 - 12 characters.
Each collection stores the sorted order of the strings, S-> index .
the strings are assumed to be unique

For some table Vec<Value>, an index exists Index<Key> such that table[index[value]] = value

a shortcoming of using vec for this is that indexes cant store &Value pointing into it,
unless the vec will never be mutated after its indexed.

if you want to have a mutable vector after indexing (the point of most of these datastructures is
fast insertion afterall), you need to base it on something that can be appended to without
a mutable reference.

also if you store references in the index, you dont need to store the key, because it can be
calculated from the array base address.
*/

pub struct Table<INDEX, VAL>
    where VAL: Ord + PartialEq + Clone
{
    inner: Vec<VAL>,
    index: INDEX,
}

pub type HashMapIndex<T> = HashMap<T, usize>;
pub type BTreeIndex<T> = BTreeMap<T, usize>;
//pub type SortedVecIndex<T> = SortedVec<(T,usize)>;
pub type RadixTreeIndex<T> = Trie<T, usize>;

// pub enum Indexes<T>
// where T : Ord + PartialEq + Hash + Clone
// {
//     no_index(NoIndex),
//     hash_map(HashMap<T,usize>),
//     btree_map(BTreeMap<T,usize>),
//     sorted_vec(SortedVec<(T,usize)>)
// }

// pub enum NumericIndexes<T>
// where T : rudy::Key
// {
//     rudy_map(RudyMap<T,usize>)
// }

pub trait SeqIndex<VALUE>
    where Self: Sized + EqIndex<VALUE>,
          VALUE: Clone
{
    fn find_n_lt<'a, 'b>(&self,
                         v: &'a VALUE,
                         n: usize,
                         data: &'b Vec<VALUE>)
                         -> impl Iterator<Item = &'b VALUE> + use<'a, 'b, '_, VALUE, Self>;
}

pub trait EqIndex<VALUE>
    where Self: Sized + Index<VALUE>,
          VALUE: Clone
{
    fn find_eq<'a, 'b>(&self, v: &'a VALUE, data: &'b Vec<VALUE>) -> Option<&'b VALUE>;
}

pub trait Index<VALUE>
    where Self: Sized + FromIterator<(VALUE, usize)>,
          VALUE: Clone
{
    fn insert(&mut self, v: &VALUE, k: usize);

    fn new(v: &[VALUE]) -> Self
    {
        return Self::from_iter(v.iter().cloned().enumerate().map(|(i, e)| (e, i)));
    }

    fn name(&self) -> &'static str;
}

fn find_n_lt_unindexed<'a, 'b, VALUE>(v: &'a VALUE,
                                      n: usize,
                                      data: &'b Vec<VALUE>)
                                      -> impl Iterator<Item = &'b VALUE> + use<'a, 'b, VALUE>
    where VALUE: Ord + Clone
{
    let mut sv = ReverseSortedVec::with_capacity(n + 1);
    data.iter().for_each(|e| {
                   if (e < v)
                   {
                       sv.insert(Reverse(e));
                   }
                   if (sv.len() > n)
                   {
                       sv.pop();
                   }
               });
    return sv.iter()
             .map(|Reverse(e)| *e)
             .collect::<Vec<&VALUE>>()
             .into_iter();
}

///outdated
fn find_n_gt_unindexed<VALUE>(v: &VALUE, n: usize, data: &Vec<VALUE>) -> impl Iterator<Item = usize>
    where VALUE: Ord + PartialEq + Clone
{
    let mut sv = SortedVec::with_capacity(n + 1);
    data.iter().enumerate().for_each(|(i, e)| {
                               if (e < v)
                               {
                                   sv.insert((e, i));
                               }
                               if (sv.len() > n)
                               {
                                   sv.pop();
                               }
                           });
    return sv.iter()
             .map(|(e, i)| *i)
             .collect::<Vec<usize>>()
             .into_iter();
}

pub struct NoIndex {}

impl<T> FromIterator<T> for NoIndex
{
    fn from_iter<X>(iter: X) -> Self
    {
        return NoIndex {};
    }
}

impl<INDEX, VALUE> Table<INDEX, VALUE>
    where INDEX: SeqIndex<VALUE>,
          VALUE: Ord + PartialEq + Hash + Clone
{
    pub fn n_less_than<'a, 'b>(&'a self,
                               n: usize,
                               v: &'b VALUE)
                               -> impl Iterator<Item = &'a VALUE> + use<'a, 'b, VALUE, INDEX>
    {
        return self.index.find_n_lt(v, n, &self.inner);
    }
}

impl<INDEX, VALUE> Table<INDEX, VALUE>
    where INDEX: EqIndex<VALUE>,
          VALUE: Ord + PartialEq + Hash + Clone
{
    pub fn equal_to(&self, v: &VALUE) -> Option<&VALUE>
    {
        return self.index.find_eq(v, &self.inner);
    }
}

impl<INDEX, VALUE> Table<INDEX, VALUE>
    where INDEX: Index<VALUE>,
          VALUE: Ord + PartialEq + Hash + Clone
{
    pub fn new(v: &[VALUE]) -> Self
    {
        return Self { inner: Vec::from(v),
                      index: INDEX::new(v), };
    }

    pub fn push(&mut self, v: &VALUE) -> usize
    {
        self.index.insert(v, self.inner.len());
        self.inner.push(v.clone());
        return self.inner.len() - 1;
    }

    pub fn name(&self) -> &'static str
    {
        return self.index.name();
    }
}

impl<VALUE> Index<VALUE> for NoIndex where VALUE: Clone
{
    fn insert(&mut self, v: &VALUE, k: usize) {}

    fn new(v: &[VALUE]) -> Self
    {
        return NoIndex {};
    }

    fn name(&self) -> &'static str
    {
        return "No Index";
    }
}

impl<VALUE> EqIndex<VALUE> for NoIndex where VALUE: Clone + PartialEq
{
    fn find_eq<'a, 'b>(&self, v: &'a VALUE, data: &'b Vec<VALUE>) -> Option<&'b VALUE>
    {
        data.iter().find(|d| *d == v)
    }
}

impl<VALUE> SeqIndex<VALUE> for NoIndex where VALUE: Ord + Clone
{
    fn find_n_lt<'a, 'b>(&self,
                         v: &'a VALUE,
                         n: usize,
                         data: &'b Vec<VALUE>)
                         -> impl Iterator<Item = &'b VALUE> + use<'a, 'b, '_, VALUE>
    {
        find_n_lt_unindexed(v, n, data)
    }
}

impl<VALUE> Index<VALUE> for HashMap<VALUE, usize> where VALUE: Ord + Hash + Clone
{
    fn insert(&mut self, v: &VALUE, k: usize)
    {
        self.insert(v.clone(), k);
    }

    fn name(&self) -> &'static str
    {
        return "Hash Map";
    }
}

impl<VALUE> EqIndex<VALUE> for HashMap<VALUE, usize> where VALUE: Ord + Hash + Clone
{
    fn find_eq<'a, 'b>(&self, v: &'a VALUE, data: &'b Vec<VALUE>) -> Option<&'b VALUE>
    {
        return self.get(v).and_then(|idx| Some(&data[*idx]));
    }
}

impl<VALUE> Index<VALUE> for BTreeMap<VALUE, usize> where VALUE: Ord + Clone
{
    fn insert(&mut self, v: &VALUE, k: usize)
    {
        self.insert(v.clone(), k);
    }

    fn name(&self) -> &'static str
    {
        return "B Tree Map";
    }
}

impl<VALUE> EqIndex<VALUE> for BTreeMap<VALUE, usize> where VALUE: Ord + Clone
{
    fn find_eq<'a, 'b>(&self, v: &'a VALUE, data: &'b Vec<VALUE>) -> Option<&'b VALUE>
    {
        return self.get(v).and_then(|idx| Some(&data[*idx]));
    }
}

impl<VALUE> SeqIndex<VALUE> for BTreeMap<VALUE, usize> where VALUE: Ord + Clone
{
    fn find_n_lt<'a, 'b>(&self,
                         v: &'a VALUE,
                         n: usize,
                         data: &'b Vec<VALUE>)
                         -> impl Iterator<Item = &'b VALUE> + use<'a, 'b, '_, VALUE>
    {
        return self.range(..v).rev().take(n).map(|(_, i)| &data[*i]);
    }
}

impl<VALUE> Index<VALUE> for Trie<VALUE, usize> where VALUE: Ord + Clone + TrieKey
{
    fn insert(&mut self, v: &VALUE, k: usize)
    {
        self.insert(v.clone(), k);
    }

    fn name(&self) -> &'static str
    {
        return "Radix Trie Map";
    }
}

impl<VALUE> EqIndex<VALUE> for Trie<VALUE, usize> where VALUE: Ord + Clone + TrieKey
{
    fn find_eq<'a, 'b>(&self, v: &'a VALUE, data: &'b Vec<VALUE>) -> Option<&'b VALUE>
    {
        return self.get(v).and_then(|idx| Some(&data[*idx]));
    }
}

///this radix tree needs work, it also doesnt support reverse iterators.
impl<VALUE> SeqIndex<VALUE> for Trie<VALUE, usize> where VALUE: Ord + Clone + TrieKey
{
    fn find_n_lt<'a, 'b>(&self,
                         v: &'a VALUE,
                         n: usize,
                         data: &'b Vec<VALUE>)
                         -> impl Iterator<Item = &'b VALUE> + use<'a, 'b, '_, VALUE>
    {
        return self.subtrie(&v)
                   .expect("Radix Tree doesnt support positioning to keys not in the tree")
                   .iter()
                   .map(|node| &data[*(node.1)])
                   .take(n);
    }
}

use rayon::prelude::*;
pub struct SortedVecIndex<VALUE: Ord + Send>(SortedVec<(VALUE, usize)>);

impl<VALUE: Ord + Send> Deref for SortedVecIndex<VALUE>
{
    type Target = SortedVec<(VALUE, usize)>;
    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<VALUE: Ord + Send> DerefMut for SortedVecIndex<VALUE>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl<VALUE> FromIterator<(VALUE, usize)> for SortedVecIndex<VALUE> where VALUE: Ord + Clone + Send
{
    fn from_iter<ITER: IntoIterator<Item = (VALUE, usize)>>(iter: ITER) -> Self
    {
        let mut v: Vec<(VALUE, usize)> = iter.into_iter().collect();
        v.par_sort_unstable();
        let sv = unsafe { SortedVec::<(VALUE, usize)>::from_sorted(v) };
        return SortedVecIndex(sv);
    }
}

impl<VALUE> Index<VALUE> for SortedVecIndex<VALUE> where VALUE: Ord + Clone + Send
{
    fn insert(&mut self, v: &VALUE, k: usize)
    {
        self.push((v.clone(), k));
    }

    fn name(&self) -> &'static str
    {
        return "Parallel Vec";
    }
}

impl<VALUE> EqIndex<VALUE> for SortedVecIndex<VALUE> where VALUE: Ord + Clone + Send
{
    fn find_eq<'a, 'b>(&self, v: &'a VALUE, data: &'b Vec<VALUE>) -> Option<&'b VALUE>
    {
        return self.0
                   .binary_search_by_key(&v, |(val, idx)| val)
                   .ok()
                   .and_then(|(index_idx)| Some(&data[self[index_idx].1]));
    }
}

impl<VALUE> SeqIndex<VALUE> for SortedVecIndex<VALUE> where VALUE: Ord + Clone + Send
{
    fn find_n_lt<'a, 'b>(&self,
                         v: &'a VALUE,
                         n: usize,
                         data: &'b Vec<VALUE>)
                         -> impl Iterator<Item = &'b VALUE> + use<'a, 'b, '_, VALUE>
    {
        let index_idx = match self.binary_search_by_key(&v, |(val, idx)| val)
        {
            Ok(x) => x,
            Err(x) => x,
        };
        return self[(index_idx - n).max(0)..index_idx].iter()
                                                      .map(|(_, idx)| &data[*idx]);
    }
}

// Define the newtype wrapper
pub struct RudyMapIndex<KEY: rudy::Key>(RudyMap<KEY, usize>);

// Implement Deref and DerefMut to access the inner RudyMap use std::ops::{Deref, DerefMut};
impl<KEY: rudy::Key> Deref for RudyMapIndex<KEY>
{
    type Target = RudyMap<KEY, usize>;
    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<KEY: rudy::Key> DerefMut for RudyMapIndex<KEY>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

// Implement FromIterWrapper for the newtype
impl<KEY> FromIterator<(KEY, usize)> for RudyMapIndex<KEY> where KEY: rudy::Key
{
    fn from_iter<ITER: IntoIterator<Item = (KEY, usize)>>(iter: ITER) -> Self
    {
        let mut map = RudyMap::new();
        for (key, value) in iter
        {
            map.insert(key, value);
        }
        RudyMapIndex(map)
    }
}

impl<VALUE> Index<VALUE> for RudyMapIndex<VALUE> where VALUE: rudy::Key
{
    fn insert(&mut self, v: &VALUE, k: usize)
    {
        self.0.insert(v.clone(), k);
    }

    fn name(&self) -> &'static str
    {
        return "Rudy Map";
    }
}

impl<VALUE> EqIndex<VALUE> for RudyMapIndex<VALUE> where VALUE: rudy::Key
{
    fn find_eq<'a, 'b>(&self, v: &'a VALUE, data: &'b Vec<VALUE>) -> Option<&'b VALUE>
    {
        return self.get(*v).and_then(|idx| Some(&data[*idx]));
    }
}

// impl<VALUE> Table<SortedVec<(VALUE,usize)>, VALUE>
// where VALUE : Ord + PartialEq + Clone
// {
//     fn new(v : &SortedVec<VALUE>) -> Self
//     {
//         let sorted = v.iter().enumerate().map(|(i,v)| (v.clone(),i)).collect();
//         // safe because the source is another sorted vec.
//         let sv = unsafe { SortedVec::from_sorted(sorted) };
//         return Table{inner : v.to_vec(), index : sv};
//     }

//     fn push(&mut self, v : VALUE)
//     {
//         self.index.insert((v.clone(),self.inner.len()));
//         self.inner.push(v);
//     }

//     fn n_less_than(&self, n : usize, v : &VALUE) -> Vec<&VALUE>
//     {
//         let mut res = Vec::with_capacity(n);
//         let key = match self.index.binary_search_by_key(&v, |(v,_)| v)
//         {
//             Ok(k) | Err(k) => k,
//         };
//         let indeces = &self.index[usize::min(0, key-n).. key];
//         indeces.iter().map(|(_,i)| &self.inner[*i]).collect_into(&mut res);
//         return res;
//     }

//     fn equal_to(&self, v : &VALUE) -> Option<&VALUE>
//     {
//         return self.index.binary_search_by_key(&v, |(v,_)| v)
//                          .ok()
//                          .map(|i| &self.inner[i]);
//     }

//     fn name()-> &'static str {return "Sorted Vec"}
// }
