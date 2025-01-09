#![feature(iter_collect_into, test)]
#![feature(precise_capturing_in_traits)]
#![feature(min_specialization)]
#![allow(unused,dead_code)]
/*
for now the database can be modeled as a Vec<Option<A>>
ideally itd be more like a Vec<Box<[Option<A>;1024]>>
where any chunk but the last is chunk_size.

indexes for now can be NON_INDEXED, BTREE<>, HASH<> , or VECTOR<>

when a query is made, the table is traversed using indeces exclusively.
indeces that havent been created are NON_INDEXED, which just yields the next value that
isnt NONE.

this strongly typed version is insufficient for what i intend to do with graphite so i'll have to
build over it.

struct Order
{
    price : u64,
    uid : u64,
    item_id : u64
}

struct Users
{
    name : String,
    uid : Unique(u64)
}

let orders = Table<Order>::load("orders.nanodb");
let users = Table<User>::load("users.nanodb");

// find all orders with a price < v
let query = orders.price.btree_index()?.less_than(v)

// find all orders for all users named 'bob'
let query2 = users.name.hash_index()
                       .eq("bob")
                       .map(|u| orders.uid.hash_index.eq(u.uid))
                       .collect())

// find all orders for all users
let query3 = users.uid.seq_index().map( |u| orders.uid.hash_index.eq(u.uid) ) ;

I'll need a macro to create the fields under users.x, which can be used in load.
*/

mod trallocator;
mod collections;

use collections::{BTreeIndex, EqIndex, HashMapIndex, Index, NoIndex, RadixTreeIndex, RudyMapIndex, SeqIndex, SortedVecIndex, Table};
use radix_trie::TrieKey;
use rand::{distributions::Uniform, prelude::*};
use smartstring::{LazyCompact, SmartString};
use sorted_vec::SortedSet;
use pstr::IStr;
use std::{alloc::System, collections::HashSet, hash::Hash, thread::Scope, time::{Duration, Instant}};
use trallocator::TracedAllocator;
use std::thread::scope;
use std::sync::mpsc;
type String = SmartString<LazyCompact>;
//type String = IStr;
// needed for Trallocator struct (as written, anyway)

#[global_allocator]
static GLOBAL: TracedAllocator<System> = TracedAllocator::new(System);

fn main()
{
    std::env::set_var("RUST_BACKTRACE", "1");
    run_benchmarks();
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
struct BenchResult
{
    subject:   &'static str,
    test:      &'static str,
    size:      usize,
    mem: u64,
    time : Duration
}

fn run_benchmarks()
{
    let sizes : &[usize] = &[10_000, 100_000, 1_000_000, 10_000_000];
    let data = random_unique_strings(*sizes.last().unwrap() as u32);
    let fk_data = random_unique_numbers(*sizes.last().unwrap() as u32);
    let mut log  = vec![];
    bench_control(&data, &sizes[0..2], &mut log);
    bench_eq_indexes(&data, &sizes, &mut log);
    bench_seq_indexes(&data, &sizes, &mut log);
    bench_fk_indexes(&fk_data, &sizes, &mut log);
    log.sort();
    for line in log 
    {
        println!("{:?}", line);
    }
}

fn bench_control<VALUE : Clone + Ord + Hash>(data : &[VALUE], sizes : &[usize], log : &mut Vec<BenchResult>)
{
    log.append(&mut bench_seq_index::<VALUE,NoIndex>(data, &sizes[0..2]));
}

fn bench_eq_indexes<VALUE : Clone + Ord + Hash>(data : &[VALUE], sizes : &[usize], log : &mut Vec<BenchResult>)
{
    log.append(&mut bench_eq_index::<VALUE,HashMapIndex<VALUE>>(data, sizes));
}

fn bench_fk_indexes<VALUE : rudy::Key + Hash>(data : &[VALUE], sizes : &[usize], log : &mut Vec<BenchResult>)
{
    log.append(&mut bench_eq_index::<VALUE,RudyMapIndex<VALUE>>(data, sizes));
}

fn bench_seq_indexes<VALUE : Clone + Ord + Hash + Send>(data : &[VALUE], sizes : &[usize], log : &mut Vec<BenchResult>)
{
    log.append(&mut bench_seq_index::<VALUE,BTreeIndex<VALUE>>(data, sizes));
    //log.append(&mut bench_seq_index::<VALUE,RadixTreeIndex<VALUE>>(data, sizes)); //radix trie doesnt support smart string so im leaving it commented.
    log.append(&mut bench_seq_index::<VALUE,SortedVecIndex<VALUE>>(data, sizes));
}

fn bench_seq_index<VALUE : Clone + Ord + Hash , INDEX: SeqIndex<VALUE>>(data : &[VALUE], sizes : &[usize]) -> Vec<BenchResult>
{
    let mut performance = vec![];

    for size in sizes 
    {
        let data = &data[0..*size];
        let (table, perf) = bench_ins::<VALUE,INDEX>(&data[0..*size]);
        performance.push(perf);
        performance.push(bench_eq(data, &table));
        performance.push(bench_seq(data, &table));
    }
    return performance;
}

fn bench_eq_index<VALUE : Clone + Ord + Hash, INDEX: EqIndex<VALUE>>(data : &[VALUE], sizes : &[usize]) -> Vec<BenchResult>
{
    let mut performance = vec![];
    for size in sizes 
    {
        let data = &data[0..*size];
        let (table, perf) = bench_ins::<VALUE,INDEX>(&data[0..*size]);
        performance.push(perf);
        performance.push(bench_eq(data, &table));
    }
    return performance;
}

fn bench_eq<VALUE : Clone + Ord + Hash, INDEX: EqIndex<VALUE>>(data: &[VALUE], table : &Table<INDEX,VALUE>) -> BenchResult
{
    let (_, mem, time) = benchit(|| {
        data.iter()
            .step_by(1.max(data.len() / 10000))
            .take(10000)
            .for_each(|ref q| {
                let _ = table.equal_to(q);
            })
    });
    let subject = table.name();
    return BenchResult{subject, test: "rand 10k", size : data.len(), mem, time };
}

fn bench_seq<VALUE: Ord + Clone + Hash, INDEX: SeqIndex<VALUE>>(data: &[VALUE], table : &Table<INDEX,VALUE>) -> BenchResult
{
    let (_, mem, time) = benchit(|| {
        data.iter()
            .step_by( 1.max(100 * data.len() / 1000))
            .take(100)
            .for_each(|q| {
                let _ = table.n_less_than(100, q).collect::<Vec<&VALUE>>();
            })
    });
    let subject = table.name();
    return BenchResult{subject, test: "sequential 100x100", size : data.len(), mem, time };
}

fn bench_ins<VALUE: Clone + Ord + Hash, INDEX: Index<VALUE>>(data: &[VALUE]) -> (Table<INDEX,VALUE>, BenchResult)
{
    let (table, mem, time) =  benchit(|| Table::<INDEX, VALUE>::new(&data));
    let subject = table.name();
    return (table, BenchResult{subject, test: "insertion", size : data.len(), mem, time })
}

fn benchit<T, F: FnOnce() -> T>(f: F) -> (T, u64, Duration)
{
    GLOBAL.reset();
    let mem0 = GLOBAL.get();
    let start = std::time::Instant::now();
    let r = f();
    let mem1 = GLOBAL.get();
    let end = Instant::now();
    return (r, mem1 - mem0, end - start);
}

fn random_unique_strings(n: u32) -> Vec<String>
{
    //the number of potential strings is about 6E16, too big for u32 but less than u64 int max.
    GLOBAL.reset();
    let (tx,rx) = mpsc::channel();
    scope(|s|
    {
        for i in 0..10
        {
            let tx = tx.clone();
            s.spawn( move || 
            {
                let mut set = HashSet::<String>::new();
                let mut rng = rand::thread_rng();
                let strlen = 7; //Uniform::new(4, 16);
                let char_rng = Uniform::new(0, 25);
                let mut buf = Vec::with_capacity(16);
                while set.len() < (n/10) as usize
                {
                    for _ in 0..strlen-1//rng.sample(strlen)
                    {
                        buf.push((rng.sample(char_rng) + 65u8) as u8);
                    }
                    buf.push(i as u8);
                    let s = String::from(std::string::String::from_utf8_lossy(&buf).to_string());
                    buf.clear();
                    set.insert(s);
                }
                let r = Uniform::new(0, set.len()-1);
                let mut desorted_set : Vec<String> = set.into_iter().collect();
                for i in 0..desorted_set.len()
                {
                    let swap_to = rng.sample(r) as usize;
                    desorted_set.swap(i,swap_to);
                }
                tx.send(desorted_set);
            });
        }
    });
    let mut sets = vec![];
    while let Ok(set) = rx.try_recv()
    {
        sets.push(set);
    }
    println!("Ram used generating {} strings : {} bytes", n, GLOBAL.get());
    return sets.concat();
}

fn random_unique_numbers(n : u32) -> Vec<usize>
{
    let mut rng = rand::thread_rng();
    let mut v : Vec<usize> = (0..n as usize).into_iter().collect();
    for i in 0..n as usize
    {
        let swap = rng.sample(Uniform::new(0,n)) as usize;
        v.swap(i,swap);
    }
    return v;
}