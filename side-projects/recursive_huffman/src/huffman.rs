//! Adapted from huffman_coding, modified to remove normalization and to support tokenized encoding.
//! **huffman_coding** is a small library for reading and writing huffman encoded data
//!
//! There are only 3 things you need to know:
//! * Use HuffmanTree to change the coding and decoding based on your data
//! * Use HuffmanWriter to encode data
//! * use HuffmanReader to decode data
// courtesy of 'huffman-coding' 

use bit_vec;
//use bitstream_io::{BigEndian, BitReader, BitWriter, BitWrite, BitRead, ByteRead, ByteWrite};
use anyhow::{anyhow, bail};
use std::cmp::{self, Reverse};
use std::collections::{HashMap, BinaryHeap};
use bit_vec::BitVec;

use crate::tree::BinaryTree;

#[derive(Eq, Debug)]
pub enum HuffmanTree {
    Leaf(u8, u64),
    Node(Box<HuffmanTree>, Box<HuffmanTree>, u64),
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let own_prob = self.get_frequency();
        let other_prob = other.get_frequency();

        // We want to use the std heap, which is a max heap. However, we want to have
        // the minimum probability on top
        if own_prob < other_prob {
            cmp::Ordering::Less
        } else if own_prob == other_prob {
            cmp::Ordering::Equal
        } else {
            cmp::Ordering::Greater
        }
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HuffmanTree {
    fn eq(&self, other: &HuffmanTree) -> bool {
        match (self, other) {
            (&HuffmanTree::Leaf(ref x1, ref prob1), &HuffmanTree::Leaf(ref x2, ref prob2)) => {
                x1 == x2 && prob1 == prob2
            },
            (&HuffmanTree::Node(ref zero1, ref one1, ref f1), &HuffmanTree::Node(ref zero2, ref one2, ref f2)) => {
                zero1 == zero2 && one1 == one2 && f1==f2
            },
            _ => false
        }
    }
}

impl HuffmanTree {

    pub fn from_frequency_table(data: &[u64;256]) -> Self {
        let mut heap: BinaryHeap<Reverse<HuffmanTree>> = data
            .iter()
            .enumerate()
            .filter(|x| *x.1 > 0)
            .map(|x| Reverse(HuffmanTree::Leaf(x.0 as u8, *x.1 as u64)))
            .collect();

        while heap.len() > 2 {
            let a = heap.pop().unwrap();
            let b = heap.pop().unwrap();
            let f = a.0.get_frequency() + b.0.get_frequency();
            let insert = if a < b {
                HuffmanTree::Node(Box::new(a.0), Box::new(b.0),f)
            } else {
                HuffmanTree::Node(Box::new(b.0), Box::new(a.0),f)
            };
            heap.push(Reverse(insert));
        }
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        let f = a.0.get_frequency() + b.0.get_frequency();
        HuffmanTree::Node(Box::new(a.0), Box::new(b.0), f)
    }

    pub fn from_data(data: &[u8]) -> Self {
        let mut probability: [u64; 256] = [0; 256];
        for item in data {
            probability[*item as usize] += 1;
        }

        HuffmanTree::from_frequency_table(&probability)
    }

    pub fn get_frequency_table(&self) -> [u64; 256] {
        let mut table: [u64; 256] = [0; 256];
        
        fn walker(table : &mut [u64;256], node : &HuffmanTree) {
            match node {
                HuffmanTree::Leaf(chr, freq) => table[*chr as usize]=*freq,
                HuffmanTree::Node(zero, one, _) => {
                    walker(table, zero);
                    walker(table, one);
                }
            }
        }
        walker(&mut table, self);
        table
    }

    fn get_frequency(&self) -> u64 {
        match self {
            &HuffmanTree::Leaf(_, freq) => freq as u64,
            &HuffmanTree::Node(_, _, freq) => freq as u64
        }
    }

    fn to_lookup_table(&self) -> HashMap<u8, BitVec> {
        let mut table = HashMap::new();
        self.to_lookup_table_inner(&mut table, BitVec::new());
        table
    }

    pub fn to_coder(self) -> HuffmanCoder {
        let mut table = vec![BitVec::new();256];
        //self.to_lookup_table_inner(&mut table, BitVec::new());
        fn walker(tree: &HuffmanTree, table : &mut Vec<BitVec>, prev : BitVec) {
            match tree {
                &HuffmanTree::Leaf(ref elem, _) => {
                    table[*elem as usize] = prev;
                },
                &HuffmanTree::Node(ref zero, ref one, _) => {
                    let mut zero_bits = prev.clone();
                    zero_bits.push(false);
                    walker(zero, table, zero_bits);
                    let mut one_bits = prev;
                    one_bits.push(true);
                    walker(one, table, one_bits);
                }
            }
        }
        walker(&self, &mut table, BitVec::new());
        return HuffmanCoder{ encoder: table, decoder : self.to_binary_tree()}; 
    }

    ///discard frequency information
    fn to_binary_tree(self) -> BinaryTree<u8>{
        fn walker(tree : HuffmanTree) -> BinaryTree<u8>{
            match tree {
                HuffmanTree::Leaf(byte, _) => BinaryTree::Leaf(byte),
                HuffmanTree::Node(l,r,_) => {
                    BinaryTree::Node(Box::new(walker(*l)), Box::new(walker(*r)))
                }
            }
        }
        return walker(self);
    }
    

    ///map of characters to their bit encoding.
    pub fn to_lookup_table_inner(&self, data: &mut HashMap<u8, BitVec>, prev: BitVec) {
        match self {
            &HuffmanTree::Leaf(ref elem, _) => {
                data.insert(*elem, prev);
            },
            &HuffmanTree::Node(ref zero, ref one,_) => {
                let mut zero_branch = prev.clone();
                zero_branch.push(false);
                zero.to_lookup_table_inner(data, zero_branch);
                let mut one_branch = prev;
                one_branch.push(true);
                one.to_lookup_table_inner(data, one_branch);
            }
        }
    }
}

//right now single character only.
#[derive(Debug)]
pub struct HuffmanCoder 
{
    //i'll need a proper map eventually
    encoder: Vec<BitVec>,

    decoder: BinaryTree<u8>
}

impl HuffmanCoder 
{
    pub fn encode(&self, src : &[u8]) -> anyhow::Result<BitVec>
    {
        let mut bv = BitVec::new();
        if  self.encoder.len() < 256  {
            bail!("Lookup table is not complete, len is {}", self.encoder.len());
        }
        for byte in src 
        {
            let bits = &self.encoder[*byte as usize];
            if bits.len() == 0 { 
                bail!("Invalid character not found in lookup table") 
            };
            bv.extend(bits);
        }
        return Ok(bv);
    }

    pub fn decode(&self, src : &BitVec) -> anyhow::Result<Vec<u8>>
    {
        let mut state = &self.decoder;
        let mut output = vec![];
        for bit in src {
            match state {
                BinaryTree::Leaf(byte) => {
                    output.push(*byte);
                    state = &self.decoder;
                }
                BinaryTree::Node(zero,one) => {
                    state = if bit {one} else {zero};
                    match state {
                        BinaryTree::Leaf(byte) => { 
                            output.push(*byte); 
                            state = &self.decoder;
                        }
                        BinaryTree::Node(_,_) => {}
                    }
                }
            }
        }
        if state != &self.decoder {bail!("Invalid state")}
        return Ok(output);
    }

    pub fn to_string(&self) -> String {
        return self.decoder.to_string();
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        fn walker(tree : &BinaryTree<u8>) -> Vec<u8> {
            return match tree {
                BinaryTree::Leaf(byte) => {
                    match byte{
                        40|41|44|92 => {vec![92,*byte]} //escape (),\ with a \
                        _ => {vec![*byte]} 
                    }
                },
                BinaryTree::Node(zero,one) => {
                    let mut v = vec![40]; 
                    v.extend(walker(zero));
                    v.push(44);
                    v.extend(walker(one));
                    v.push(41);
                    return v;
                }
            };
        }
        return walker(&self.decoder);
    }

    pub fn from_string(s : &str) -> Result<(Self, &str), anyhow::Error>
    {
        let binary_tree = BinaryTree::<u8>::from_string(s)?;
        let mut encoder = vec![BitVec::new();256];
        fn walker(tree: &BinaryTree<u8>, table : &mut Vec<BitVec>, prev : BitVec) {
            match tree {
                &BinaryTree::Leaf(ref elem) => {
                    table[*elem as usize] = prev;
                },
                &BinaryTree::Node(ref zero, ref one) => {
                    let mut zero_bits = prev.clone();
                    zero_bits.push(false);
                    walker(zero, table, zero_bits);
                    let mut one_bits = prev;
                    one_bits.push(true);
                    walker(one, table, one_bits);
                }
            }
        }
        walker(&binary_tree.0, &mut encoder, BitVec::new());
        return Ok((Self{encoder, decoder: binary_tree.0}, binary_tree.1));

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bit_vec::BitVec;

    #[test]
    fn test_tree_builder() {
        let vec = vec![1, 2, 3, 1, 1, 2];
        let tree = HuffmanTree::from_data(&vec[..]);
        let table = tree.to_lookup_table();

        use std::iter::FromIterator;
        dbg!(&table);
        assert_eq!(table[&1u8], BitVec::from_iter(vec![false].into_iter()));
        assert_eq!(table[&2u8], BitVec::from_iter(vec![true, false].into_iter()));
        assert_eq!(table[&3u8], BitVec::from_iter(vec![true, true].into_iter()));
    }

    #[test]
    fn test_encode_decode() {
        let vec = vec![1u8, 2, 3, 1, 1, 2, 4, 12, 3, 3, 0];
        let tree = HuffmanTree::from_data(&vec[..]);
        let coder = tree.to_coder();
        let encoded = coder.encode(&vec).unwrap();
        let mut bytes = coder.to_bytes();
        bytes.push(0);
        let bytes = &String::from_utf8_lossy(&bytes);
        dbg!(bytes);
        dbg!(&encoded);
        let decoded = coder.decode(&encoded).unwrap();
        assert_eq!(decoded, vec);
    }
}

// /// *HuffmanWriter* is a Write implementation that writes encoded words to the
// /// inner writer.
// ///
// /// # Examples
// /// ```
// /// extern crate huffman_coding;
// /// let pseudo_data = vec![0, 0, 1, 2, 2];
// /// let tree = huffman_coding::HuffmanTree::from_data(&pseudo_data[..]);
// ///
// /// let mut output = Vec::new();
// /// {
// ///     use std::io::Write;
// ///     let mut writer = huffman_coding::HuffmanWriter::new(&mut output, &tree);
// ///     assert!(writer.write(&[2, 2, 0, 0, 1]).is_ok());
// /// }
// /// assert_eq!(&output[..], [43, 8]);
// /// ```


// pub struct HuffmanWriter<W> where W: Write {
//     inner: BitWriter<W,BigEndian>,
//     table: HashMap<u8, BitVec>,
// }

// impl<W> HuffmanWriter<W> where W: Write {
//     /// Construct a new HuffmanWriter using the provided HuffmanTree
//     pub fn new(writer: W, tree: &HuffmanTree) -> Self {
//         HuffmanWriter {
//             inner: BitWriter::new(writer),
//             table: tree.to_lookup_table()
//         }
//     }
// }

// impl<W> HuffmanWriter<W> where W: Write {
//     /// maps u8's to their encoding. 
//     /// pads end of stream with 0s to a whole byte.
//     /// returns (bytes written, bits padding) on success
//     fn write(&mut self, buf: &[u8]) -> IOResult<(usize,u8)> {
//         let mut bits_written : u64 = 0;
//         //id like the error handling here to return the amount written 
//         for item in buf {
//             let bits = self.table.get(item).ok_or(IOError::from(IOErrorKind::InvalidData))?;
//             for bit in bits {
//                 self.inner.write_bit(bit)?;
//             }
//             bits_written += bits.len() as u64;
//         }
//         let padding = (bits_written % 8 ) as u8; 
//         self.inner.pad(padding as u32)?;
//          //the final byte written is the amount of padding in the second to last.
//         self.inner.write::<8,u8>(padding)?; 
//         return Ok(( 1+(bits_written + padding as u64) as usize/8, padding as u8));
//     }

//     fn flush(&mut self) -> IOResult<()> {
//         Ok(())
//     }
// }

// /// *HuffmanReader* is a Read implementation that can read encoded words from the inner reader
// ///
// /// # Examples
// /// ```
// /// extern crate huffman_coding;
// /// let pseudo_data = vec![0, 0, 1, 2, 2];
// /// let tree = huffman_coding::HuffmanTree::from_data(&pseudo_data[..]);
// ///
// /// use std::io::{Read, Cursor};
// /// let cursor = Cursor::new([43, 8]);
// /// let mut buffer: [u8; 5] = [0; 5];
// ///
// /// let mut reader = huffman_coding::HuffmanReader::new(cursor, tree);
// /// assert!(reader.read_exact(&mut buffer[..]).is_ok());
// /// assert_eq!(&buffer[..], &[2, 2, 0, 0, 1]);
// /// ```
// pub struct HuffmanReader<R> where R: Read {
//     inner: BitReader<R,BigEndian>,
//     tree: HuffmanTree,
//     uncompressed_length: u64,
//     //leftover: Option<Vec<u8>> //bytes that didn't fit into the buffer on last read 
// }

// impl<R> HuffmanReader<R> where R: Read {
//     /// Construct a new reader, using the provided HuffmanTree for decoding
//     /// intended for a single use.
//     pub fn new(reader: R, tree: HuffmanTree, uncompressed_length: u64) -> Self {
//         HuffmanReader {
//             inner: BitReader::new(reader),
//             tree: tree,
//             uncompressed_length
//         }
//     }
// }

//impl<R> Read for HuffmanReader<R> where R: Read {
    // ///OK THERES A LOT OF COMPLEXITY INVOLVED IN READING A NON BYTE ALIGNED
    // ///STREAM INTO A BUFFER WHEN THE SOURCE LENGTH IS UNKNOWN
    // fn read_leftovers(&mut self, dst : &mut &mut [u8]) -> IOResult<usize>
    // {
    //     if let Some(leftovers) = &self.leftover {
    //         let mut slice = &leftovers[..];
    //         loop {   
    //             let res = dst.write(&slice);
    //             if let Ok(written) = res {
    //                 slice = &slice[written..];
    //                 if slice.len() == 0 {
    //                     let written = leftovers.len();
    //                     self.leftover = None;
    //                     return Ok(written);
    //                 }
    //             }
    //             else {
    //                 self.leftover = if slice.len() > 0 
    //                                     {Some(Vec::from(slice))}
    //                                 else 
    //                                     {None};
    //                 return res;
    //             } 
    //         }
    //     }
    //     else { return Ok(0); }

    // }
    // ///REQUIRES the expected final, uncompressed length, otherwise padding will get deserialized.
    // fn read(&mut self, mut dst: &mut [u8]) -> IOResult<usize> {
    //     const BITE_SIZE : usize = 1024;

    //     //first, attempt to write any leftovers from last read into dst. 
    //     let wrote = self.read_leftovers(&mut dst)?;
    //     if dst.len() == 0 { return Ok(wrote); }

    //     let mut pos = 0;
    //     let mut state = &self.tree;
    //     let mut buf = [0u8;BITE_SIZE + 2];

    //     //unwrap ok bc we havent read any bits so it has to be aligned. 
    //     let mut reader = self.inner.reader().unwrap();
    //     let redd = reader.read(&mut buf[..BITE_SIZE])?;
    //     //a huffman serialized bytestring is at least 2 bytes.
    //     if redd < 2 {return Err(IOError::from(IOErrorKind::InvalidData));}

    //     let padding = match redd {
    //         1024 => None,
    //         x => Some(buf[redd-1])
    //     };

    //     while padding.is_none() {
    //         //at least 1022 bytes are fine to read.
    //     }

    //     //the end was found, do everything up to the last 2 bytes same as usual.
    //     for _ in 0..(redd-2) {
        
    //     }
    //     //decompress the last 8-padding bits of the last byte

    //     fn read_all_as_unpadded_bits(src : &[u8], dst : &mut[u8], state: &mut &HuffmanTree)
    //     {
    //         let bit_reader = BitReader::<_,BigEndian>::new(src);

    //     }
    //     while pos < buf.len() && pos < self.uncompressed_length as usize{
    //         let bit_opt = self.inner.read_bit();
    //         if let Ok(bit) = bit_opt {
    //             match state {
    //                 //TODO : update this for multi character tokens.
    //                 &HuffmanTree::Leaf(x, _) => {
    //                     buf[pos] = x;
    //                     pos += 1;
    //                     state = &self.tree;
    //                 },
    //                 &HuffmanTree::Node(ref zero, ref one) => {
    //                     state = if bit { one } else { zero };
    //                     if let &HuffmanTree::Leaf(x, _) = state {
    //                         buf[pos] = x;
    //                         pos += 1;
    //                         state = &self.tree;
    //                     }
    //                 }
    //             }
    //         } else {
    //             if &self.tree != state {
    //                 return Err(IOError::from(IOErrorKind::InvalidData))
    //             } else {
    //                 break;
    //             }
    //         }
    //     }
    //     Ok(pos)
    // }
//}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use bit_vec::BitVec;

//     #[test]
//     fn test_tree_builder() {
//         let vec = vec![1, 2, 3, 1, 1, 2];
//         let tree = HuffmanTree::from_data(&vec[..]);
//         let table = tree.to_lookup_table();

//         use std::iter::FromIterator;
//         dbg!(&table);
//         assert_eq!(table[&1u8], BitVec::from_iter(vec![false].into_iter()));
//         assert_eq!(table[&2u8], BitVec::from_iter(vec![true, false].into_iter()));
//         assert_eq!(table[&3u8], BitVec::from_iter(vec![true, true].into_iter()));
//     }

//     #[test]
//     fn test_writer() {
//         use std::io::Write;
//         let pseudo_data = vec![0, 0, 1, 2, 2];
//         let tree = HuffmanTree::from_data(&pseudo_data[..]);
//         dbg!(&tree);
//         let mut vec = Vec::new();
//         {
//             let mut writer = HuffmanWriter::new(&mut vec, &tree);
//             assert!(writer.write(&[0, 2, 1, 1, 2, 2, 0, 2]).is_ok())
//         }
//         assert_eq!(&vec, &[158,64]);
//         //idk how he got that, it doesnt seem correct.
//         //assert_eq!(&vec[..], &[175, 0 , 4]);
//     }

//     #[test]
//     fn test_reader() {
//         let mut table: [u64; 256] = [0; 256];
//         table[0] = 255;
//         table[1] = 128;
//         table[2] = 255;
//         let tree = HuffmanTree::from_table(&table);

//         let mut input: [u8; 2] = [0; 2];
//         input[0] = 158;
//         input[1] = 64;
//         use std::io::Cursor;
//         let mut buf = vec![0; 8];
//         let mut read = HuffmanReader::new(Cursor::new(input), 
//                                           tree, 
//                                           input.len() as u64);
//         use std::io::Read;
//         //assert!(read.read_exact(&mut buf[..]).is_ok());

//         assert_eq!(&buf[..], &[0, 2, 1, 1, 2, 2, 0, 2]);
//         // let read_end = read.read(&mut buf[..]);
//         // assert!(read_end.is_ok());
//         // assert_eq!(read_end.unwrap(), 0);
//     }
// }
