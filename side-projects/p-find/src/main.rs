#![allow(unused_parens)]

/// the goal here was to make a search algo that takes a 
/// best guess at each hop, instead of a binary choice.
/// it does find solutions even at very large data sizes in 3-4 hops, but its still
/// twice as slow as binary search.

use std::{ops::Range, time::{Duration, Instant}, u32};

use rand::{Rng, distributions::Uniform, thread_rng};
use rayon::prelude::*;
fn main()
{
    let data = random_numbers(1_000_000);
    let rqueries = random_numbers(10_000);
    let yqueries: Vec<_> = data.iter().step_by(100).collect();
    println!("Binary Search Time {:?}",
             benchit(|| {
                 for q in &rqueries
                 {
                     //let res = data.binary_search(q);
                 }
                 for q in &yqueries
                 {
                     let res = data.binary_search(q);
                 }
             }).1);
    println!("P Find Time {:?}",
             benchit(|| {
                 for q in &rqueries
                 {
                     //let res = p_find(*q, &data);
                 }
                 for q in &yqueries
                 {
                     let res = lerp_search3(**q, &data);
                 }
             }).1);
    let mut test = true;
    for q in &yqueries
    {
        let my_res = lerp_search3(**q, &data);
        let b_res = data.binary_search(q);
        test &= my_res == b_res.ok();
    }
}
#[inline]
fn clamp(x: f64, min: f64, max: f64) -> f64
{
    x.max(min).min(max)
}

//remap x from range p1-p2 to q1-q2
#[inline]
fn remap(x: f64, p1: f64, p2: f64, q1: f64, q2: f64) -> f64
{
    return x * (q2-q1)/(p2-p1)
}

#[rustfmt::skip]
fn lerp_search3(q: u32, v: &Vec<u32>) -> Option<usize>
{
    let qf = q as f64;
    let p1 = 0f64;
    let p2 = v.len() as f64 -1f64;
    let v1 = v[0] as f64;
    let v2 = v[v.len() - 1] as f64;
    let inv_slope = (p2-p1)/(v2-v1);
    let pos = clamp( (qf - v1) * inv_slope ,p1, p2);
    return exp_find(q, v, pos as usize);
}
#[inline]
fn right_direction(target : i64, value : i64, direction : i64) -> bool
{
    return (target-value)*direction > 0
}

fn ascending_range(a : i64, b : i64) -> Range<usize>
{
    return if b > a { a as usize..b as usize} else { b as usize .. a as usize };
}

fn exp_find(q : u32 , v : &Vec<u32>, guess : usize) -> Option<usize>
{
    let mut pos = guess as i64;
    let bound = (v.len() -1) as i64;
    let mut e = 16;
    //if(v[pos as usize] == q) { return Some(pos as usize); }
    let mut direction = if(v[pos as usize] > q){-1} else {1};

    if let Some(solution) = v[ascending_range(pos, pos+direction*16)].iter().position(|e| *e == q){return Some(solution);}

    //skip more as we move towards it, stopping when we pass it or exhaust the array.
    while let Some(val) = v.get(pos as usize)
    {
        if(*val == q){ return Some(pos as usize); }
        if !right_direction(q as i64, *val as i64, direction) {direction *= -1; break;}
        if (pos == 0 || pos == bound) {return None; }
        pos = (pos + e* direction).max(0).min(bound);
        e <<= 1;
    }


    e >>= 1;


    //skip less as we move towards it, turning around when we pass it, until we've decremented to 0 or found it.
    while e > 16
    {
        let val = v[pos as usize];
        if(val == q){ return Some(pos as usize); }
        if( !right_direction(q as i64, val as i64, direction)) // if we've passed it, turn around.
        {
            direction *= -1;
        }
        pos += e * direction;
        e >>= 1;
    }

    if( !right_direction(q as i64, v[pos as usize] as i64, direction)) // if we've passed it, turn around.
    {
        direction *= -1;
    }
    return v[ ascending_range(pos, pos+e*direction)].iter().position(|e| *e==q);

}

///about the same
#[rustfmt::skip]
fn lerp_search2(q: u32, v: &Vec<u32>) -> Option<usize>
{
    let qf = q as f64;
    let mut p1 = 0f64;
    let mut p2 = v.len() as f64 -1f64;
    let mut v1 = v[0] as f64;
    let mut v2 = v[v.len() - 1] as f64;

    let mut inv_slope = (p2-p1)/(v2-v1);
    let mut pos = clamp( (qf - v1) * inv_slope ,p1, p2);
    let mut guess = v[pos as usize];
    if ( q == guess ) {return Some(pos as usize);}
    if (q < guess) { p2 = pos - 1.0 ; v2 = v[p2 as usize] as f64; }
    else { p1 = pos + 1.0; v1 = v[p1 as usize] as f64; }

    let mut c = 0;
    while c < 16 && p2 - p1 > 32.0
    {
        inv_slope =  (p2-p1) / (v2-v1);
        pos = clamp(pos + (q as i64 - guess as i64) as f64 * inv_slope, p1, p2);
        guess = v[pos as usize];
        if ( q == guess ) {return Some(pos as usize);}
        if (q < guess) { p2 = pos - 1.0 ; v2 = v[p2 as usize] as f64; }
        else { p1 = pos + 1.0; v1 = v[p1 as usize] as f64; }
        //if lower_b == upper && lower != q as f64 {return None;}
        c +=1 
    }
    return v[p1 as usize..p2.max(p1) as usize].iter().position(|&e| e==q);
}

fn lerp_search(q: u32, v: &Vec<u32>) -> Option<usize>
{
    if (v.len() == 0)
    {
        return None;
    }
    //assume elements are uniformly distriuted in a sorted array.
    let qf = q as f64;
    let mut len = v.len() as f64;
    let mut min = 0f64;
    let mut max = v.len() as f64 - 1f64;
    let mut minv = v[min as usize] as f64;
    let mut maxv = v[max as usize] as f64;
    let mut pos = 0f64;
    let mut dy = (maxv - minv) as f64;
    let mut dx = (len / dy) * (qf - minv);
    pos += dx;
    let mut here = *v.get(pos as usize)? as f64;
    let mut c = 0;
    while len > 16f64 && c < 16
    {
        if (f64::abs(qf - here) < 1f64)
        {
            return Some(pos as usize);
        }
        else if (qf < here)
        {
            max = pos - 1f64;
            maxv = v[max as usize] as f64;
        }
        else
        {
            min = pos + 1f64;
            minv = v[min as usize] as f64;
        }
        //if max < min { return None; }
        len = (max - min);
        dy = (maxv - minv) as f64;
        dx = (len / dy) * (qf - minv);
        pos = min + dx;
        here = *v.get(pos as usize)? as f64;
        c += 1;
    }
    if (max > min || max - min > 1000f64)
    {
        return None;
    }
    return v[min as usize..max as usize].iter().position(|&e| e == q);
}

fn random_numbers(n: u32) -> Vec<u32>
{
    let mut rng = rand::thread_rng();
    let mut v = Vec::with_capacity(n as usize);
    for _ in 0..n as usize
    {
        v.push(rng.sample(Uniform::new(0, u32::MAX)) as u32);
    }
    v.par_sort();
    return v;
}

fn benchit<T, F: FnOnce() -> T>(f: F) -> (T, Duration)
{
    // GLOBAL.reset();
    // let mem0 = GLOBAL.get();
    let start = std::time::Instant::now();
    let r = f();
    // let mem1 = GLOBAL.get();
    let end = Instant::now();
    return (r, end - start);
}
