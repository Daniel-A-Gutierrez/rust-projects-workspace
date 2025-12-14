mod huffman;
mod tree;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fs;
use std::env;
use std::cmp;
use anyhow::ensure;
use anyhow::{anyhow,bail,Result};
//step 1 : create a huffman tree for some input data. 
#[derive(PartialOrd,Ord,PartialEq,Eq,Debug,Clone)]
pub struct Token (Vec<u8>);

///recursive huffman tree
#[derive(PartialEq, Eq, Debug)]
pub enum RHTree {
    Leaf(Token, u64),
    Node(Box<RHTree>, Box<RHTree>, u64),
}

impl Ord for RHTree {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let own_prob = self.frequency();
        let other_prob = other.frequency();
        return own_prob.cmp(&other_prob);
    }
}

impl PartialOrd for RHTree {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let own_prob = self.frequency();
        let other_prob = other.frequency();
        return Some(own_prob.cmp(&other_prob));
    }
}

/*
Ok I need some terminology
Huffman Tree : uses characters
RH Tree : similar to original huffman tree, only encodes characters present in corpus 
Complete RH Tree : encodes all possible characters, as well as tokens in corpus

Algorithm for creation: 
create a huffman tree of a corpus
according to a heuristic, choose 2 tokens A and B to combine
    new_freq = count the number of times AB occurs
    A.freq -= new_freq, B.freq -= new_freq
    re-tokenize the corpus, replacing A,B with AB
    tree = new_tree(new_frequencies)

I think the only good way to keep track of the tokenized corpus is to 
do the replacements 1 by 1, as we merge tokens. 

This thing can be made much faster if each token stores a list of its occurences in the corpus.

So heuristic idea 1 : 
    goal : minimize the size of the combined huffman tree and encoded corpus.

    decent approximation for change in tree size : ab.len() -a.len() if freq==0 - b.len() if freq==0
    change in corpus : a.len()*a.delta_freq() + b.len()*b.delta_freq() -ab.len()*ab.freq() 

    actually what if I made the tree encode itself? nah the client has no way of decoding it.
    but maybe just for the purposes of judging its length? subtract the old tokens frequencies by 1...

    exact change in tree size
    trees can be serialized like (a,(b,c)).
    Each node becomes 3 characters - (,) , each leaf becomes token.len() characters.
    all tokens are leaves, removing one turns a node into a leaf, so we save 3 + token.len()
    i think position based coding should be possible, and each token could be pessimistically assumed 
    to be unique and thus requiring a full log2(unique) to represent. thus, 2 bits per (,), so 
    6  + log2(unique_tokens)*token.len()  
    im not factoring escape characters, and removing the last leaf is a special case. 

    I guess if I have the complete tree I can use that to compute the delta exactly
    without re-encoding everything. I think ill do that. 

So what I need now is to create huffman trees for some other files, save the encoded trees, 
then train a tree on that file.
I'll just add 1 as the frequency for everything and itll be a complete tree.  
*/

impl RHTree {
    pub fn from_complete_frequency_map(data: &BTreeMap<Vec<u8>,u64>) -> Result<Self> {
        data.keys().enumerate().try_for_each(|t| {
            ensure!(t.1.len() == 1 && t.1.get(0) == Some(&(t.0 as u8)), 
                "data not comprehensive over single bytes");    
            Ok(())
        })?;
        return Self::from_frequency_map(data);
    }

    pub fn from_frequency_map(data: &BTreeMap<Vec<u8>,u64>) -> Result<Self> {
        let mut heap: BinaryHeap<Reverse<RHTree>> = data
            .iter()
            .map(|x| Reverse(RHTree::Leaf(Token(x.0.clone()), *x.1 )))
            .collect();

        while heap.len() > 2 {
            let a = heap.pop().unwrap();
            let b = heap.pop().unwrap();
            let f = a.0.frequency() + b.0.frequency();
            let insert = if a < b {
                RHTree::Node(Box::new(a.0), Box::new(b.0), f)
            } else {
                RHTree::Node(Box::new(b.0), Box::new(a.0), f)
            };
            heap.push(Reverse(insert));
        }
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        let f = a.0.frequency() + b.0.frequency();
        Ok(RHTree::Node(Box::new(a.0), Box::new(b.0), f))
    }

    fn frequency(&self) -> u64 {
        match &self {
            &RHTree::Leaf(Token (_), f) => *f as u64,
            &RHTree::Node(_, _, f) => *f
        }
    }

    pub fn get_frequency_map(&self) -> BTreeMap::<Vec<u8>,u64> {
        let mut map = BTreeMap::<Vec<u8>,u64>::new();
        
        fn walker(map : &mut BTreeMap::<Vec<u8>,u64>, node : &RHTree) {
            match node {
                RHTree::Leaf(Token (symbol), f) => 
                    {map.insert(symbol.clone(),*f);},
                RHTree::Node(zero, one, _) => {
                    walker(map, zero);
                    walker(map, one);
                }
            }
        }
        walker(&mut map, self);
        map
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bit_vec::BitVec;

    //58% on notes, 69.6% on bikes, 18% on smol.txt
    #[test]
    fn huffman_encode_1()
    {
        //let args : Vec<String> = env::args().collect();
        let args = vec!["./data/smol.txt"];
        assert!(args.len() == 1, "Program expects the sole argument to be a path to a file to comrpess");
        let src =  fs::read(&args[0]).expect("failed to read file as utf8 string");
        let tree = huffman::HuffmanTree::from_data(&src);
        let coder = tree.to_coder();
        let bits = coder.encode(&src).unwrap();
        let compression_ratio = (bits.len() as f64 /8.0) / src.len() as f64 ;
        dbg!(compression_ratio);
        //let tree = huffman::Tree::new();
    }
}



