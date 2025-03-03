#![allow(unused_parens, non_snake_case, dead_code)]

use rand::Rng;
use rand::distr::uniform::Uniform;

pub fn random_numbers(n: u32, min : i32, max : i32) -> Vec<i32>
{
    let mut rng = rand::rng();
    let mut v = Vec::with_capacity(n as usize);
    let d = Uniform::new(min,max).unwrap();
    for _ in 0..n as usize
    {
        v.push(rng.sample(d) as i32);
    }
    //v.par_sort();
    return v;
}

pub fn random_unique_strings(n: u32, strlen : u32) -> Vec<String>
{
    //the number of potential strings is about 6E16, too big for u32 but less than u64 int max.
    let (tx, rx) = std::sync::mpsc::channel();
    let nworkers = (n/100).max(10).min(1);
    std::thread::scope(|s| {
        for i in 0..nworkers
        {
            let tx = tx.clone();
            s.spawn(move || {
                 let mut set = std::collections::HashSet::<String>::new();
                 let mut rng = rand::rng();
                 let char_rng = Uniform::new(0, 25).unwrap();
                 let mut buf = Vec::with_capacity(strlen as usize+1);
                 while set.len() < 1 + (n / nworkers) as usize
                 {
                     for _ in 0..strlen - 1
                     //rng.sample(strlen)
                     {
                         buf.push((rng.sample(char_rng) + 65u8) as u8);
                     }
                     buf.push(i as u8);
                     let s = String::from(std::string::String::from_utf8_lossy(&buf).to_string());
                     buf.clear();
                     set.insert(s);
                 }
                 //unused? let r = Uniform::new(0, set.len() - 1).unwrap();
                 let desorted_set: Vec<String> = set.into_iter().collect();
                 tx.send(desorted_set).unwrap();
             });
        }
    });
    let mut sets = vec![];
    while let Ok(set) = rx.try_recv()
    {
        sets.push(set);
    }
    let mut too_many = sets.concat();
    too_many.truncate(n as usize);
    return too_many;
}
