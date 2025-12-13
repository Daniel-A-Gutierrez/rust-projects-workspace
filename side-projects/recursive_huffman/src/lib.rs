mod huffman;
use std::fs;
use std::env;
//step 1 : create a huffman tree for some input data. 

fn huffman_encode_1()
{
    let args : Vec<String> = env::args().collect();
    assert!(args.len() == 1, "Program expects the sole argument to be a path to a file to comrpess");
    let src =  fs::read_to_string(&args[0]).expect("failed to read file as utf8 string");
    //let tree = huffman::Tree::new();

}

