/*
Eytzinger orderings are a alternative sorting of an ordered array that improves cache locality of binary search.
The functions in this module generate a generalized Eytzinger ordering for N-ary search.
Technically its not an Eytzinger ordering anymore but I dont know what to call it.
I could be vain and say GE-Ordering, for Generalized Eytzinger Ordering that also means Gutierrez-Eyztinger...
*/

use anyhow::{bail, Result};
//#![feature(test)]
#[cfg(test)]
mod test
{
    extern crate test;
    use super::*;
    use test::Bencher;

    #[test]
    fn gen_e_orderings()
    {
        for i in 0..64
        {
            let o = eytzinger_ordering(i);
            let correct = if test_ordering(&o) { "✓" } else { "❌" };
            println!("{i} {correct}: {:?}", o);
        }
    }

    #[test]
    fn gen_N_orderings()
    {
        let N = 4;
        let mut len = N;
        for i in 0..4
        {
            let r = GE_ordering9::<4>(len - 1).unwrap();
            let correct = if test_ordering(&r) { "✓" } else { "❌" };
            println!("{} {correct}: {:?}", len - 1, r);
            len *= N;
        }
    }

    #[test]
    fn gen_GE_orderings()
    {
        let pow = 3;
        for i in 3..16usize
        {
            let len = i.pow(pow) - 1;
            let r = fastest_GE_ordering(i, len).unwrap();
            let correct = if test_ordering(&r) { "✓" } else { "❌" };
            println!("Degree : {i}, Length : {len}, {correct}");
        }
    }

    #[test]
    fn gen_rev_orderings()
    {
        let pow = 3;
        for i in 3..16usize
        {
            let len = i.pow(pow) - 1;
            let r = reverse_GE_ordering(i, len).unwrap();
            let correct = if test_ordering(&r) { "✓" } else { "❌" };
            println!("Degree : {i}, Length : {len}, {correct}");
        }
    }

    #[bench]
    fn ordering_bench(b: &mut Bencher)
    {
        b.iter(|| GE_ordering9::<16>(16usize.pow(4) - 1));
    }

    #[bench]
    fn control_bench(b: &mut Bencher)
    {
        b.iter(|| (0..(16usize.pow(4))).into_iter().collect::<Vec<usize>>());
    }
}

//ensure uniqueness and completeness of set.
fn test_ordering(ordering: &Vec<usize>) -> bool
{
    let mut copy = ordering.clone();
    copy.sort();
    return copy.iter().enumerate().all(|(a, b)| a == *b);
}

fn eytzinger_ordering(len: usize) -> Vec<usize>
{
    let mut numer = 1;
    let mut denom = 2;
    let mut ordering = vec![];
    for _ in 0..len
    {
        ordering.push(numer * len / denom);
        numer += 2;
        if numer > denom
        {
            numer = 1;
            denom *= 2;
        }
    }
    return ordering;
}

fn power_of_N_minus_one(q: u64, n: u64) -> (bool, u32)
{
    let log = (q + 1).ilog(n); // (26+1).ilog(3) = 3
    return (n.pow(log) == (q + 1), log); // 3^3 == 26 + 1.
}

// dividing with each step could be skipped if i scaled the array by denom.
fn GE_ordering(degree: usize, len: usize) -> Result<Vec<usize>>
{
    if degree < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, degree as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              degree,
              len
        );
    }

    let mut chunks = 1;
    let chunk_len = degree - 1;
    let mut denom = degree;
    let mut ordering = Vec::with_capacity(len);
    for _ in 0..depth
    {
        let mut i = 1;
        for _ in 0..chunks
        {
            for index in i..i + chunk_len
            {
                ordering.push(index * len / denom);
            }
            i += chunk_len + 1;
        }
        chunks = denom;
        denom *= degree;
    }
    return Ok(ordering);
}

//4x slower ouch
fn reverse_GE_ordering(degree: usize, len: usize) -> Result<Vec<usize>>
{
    if degree < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, _) = power_of_N_minus_one(len as u64, degree as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              degree,
              len
        );
    }

    let mut rev_ordering = Vec::with_capacity(len);
    let mut next_tierA = Vec::from_iter((0..len).into_iter());
    let mut next_tierB = Vec::with_capacity(len / degree / degree);
    let mut src = &mut next_tierA;
    let mut dst = &mut next_tierB;
    while rev_ordering.len() < len
    {
        for _ in 0..(src.len() / degree).max(1) - 1
        {
            for _ in 0..degree - 1
            {
                rev_ordering.push(src.pop().unwrap());
            }
            dst.push(src.pop().unwrap());
        }
        for _ in 0..degree - 1
        {
            rev_ordering.push(src.pop().unwrap());
        }
        let tmp = src;
        src = dst;
        dst = tmp;
    }
    rev_ordering.reverse();
    return Ok(rev_ordering);
}

// almost 2x as fast! 78.5us -> 48us
fn faster_GE_ordering(degree: usize, len: usize) -> Result<Vec<usize>>
{
    if degree < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, degree as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              degree,
              len
        );
    }

    let mut rev_ordering = Vec::with_capacity(len);
    let degree = degree as i64;
    let mut skip_every = degree;
    let mut step_by = 1i64;
    for _ in 0..depth
    {
        //let mut i = len as i64;
        for i in (0..len as i64 + 1).rev().step_by(skip_every as usize)
        //0..(i+1)/skip_every
        {
            for d in (step_by..skip_every).step_by(step_by as usize)
            {
                rev_ordering.push((i - d) as usize);
            }
            //i -= skip_every;
        }
        step_by = skip_every;
        skip_every *= degree;
    }

    rev_ordering.reverse();
    return Ok(rev_ordering);
}

//actually not any faster...
fn fasterist_GE_ordering<const DEGREE: i64>(len: usize) -> Result<Vec<usize>>
{
    if DEGREE < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, DEGREE as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              DEGREE,
              len
        );
    }

    let mut ordering = vec![0; len];
    let mut skip_every = DEGREE;
    let mut step_by = 1i64;
    for _ in 0..depth
    {
        //let mut i = len as i64;
        //len+1 / skip_every * (degree - 1) items per loop
        //so we start each tier at len/skip_every ? 0 also seems to always be in the bottom tier.
        //each spot is (idx-start) + (idx-start)/(degree or skip_every-1)
        let start = len * 1 / skip_every as usize;
        let tier_iter = (0..len).rev()
                                .enumerate()
                                .step_by(step_by as usize)
                                .map(|(i, e)| e + i / skip_every as usize);
        ordering[start..].iter_mut()
                         .rev()
                         .zip(tier_iter)
                         .for_each(|(dst, src)| *dst = src);

        step_by = skip_every;
        skip_every *= DEGREE;
    }

    return Ok(ordering);
}

//WELL NOW IT DOES SIMD I GUESS SMFH like 3x faster than base. 25us.
fn fastest_GE_ordering(degree: usize, len: usize) -> Result<Vec<usize>>
{
    if degree < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, degree as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              degree,
              len
        );
    }

    let mut ordering = vec![0; len];
    let mut iter = ordering.iter_mut();
    let degree = degree as i64;
    let mut skip_every = degree;
    let mut step_by = 1i64;
    for _ in 0..depth
    {
        //let mut i = len as i64;
        for i in (0..len as i64 + 1).rev().step_by(skip_every as usize)
        //0..(i+1)/skip_every
        {
            for d in (step_by..skip_every).step_by(step_by as usize)
            {
                *(iter.next()).unwrap() = (i - d) as usize;
            }
            //i -= skip_every;
        }
        step_by = skip_every;
        skip_every *= degree;
    }

    return Ok(ordering);
}

//wait that actually worked, its down to 18.7 us from the original's 78.5us ...
//also at sizes around 16M (16^6) it breaks even with the control.
fn faSTERMASDF_GE_ordering<const DEGREE: i64>(len: usize) -> Result<Vec<usize>>
{
    if DEGREE < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, DEGREE as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              DEGREE,
              len
        );
    }

    let mut ordering = vec![0; len];
    let mut iter = ordering.iter_mut();
    let mut skip_every = DEGREE;
    let mut step_by = 1i64;
    for _ in 0..depth
    {
        //let mut i = len as i64;
        for i in (0..len as i64 + 1).rev().step_by(skip_every as usize)
        //0..(i+1)/skip_every
        {
            for d in (0..DEGREE - 1)
            {
                *(iter.next()).unwrap() = (i - d * step_by) as usize;
            }
            //i -= skip_every;
        }
        step_by = skip_every;
        skip_every *= DEGREE;
    }

    return Ok(ordering);
}

//down to 17.1 us. jk it wasnt correct, its 18us now. also checked godbolt, its not simd yet.
fn GE_ordering7<const DEGREE: i64>(len: usize) -> Result<Vec<usize>>
{
    if DEGREE < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, DEGREE as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              DEGREE,
              len
        );
    }

    let mut ordering = vec![0; len];
    let mut iter = ordering.iter_mut().rev();
    let mut skip_every = DEGREE;
    let mut step_by = 1i64;
    for _ in 0..depth
    {
        for i in (1..(len as i64 + 1) / skip_every + 1).rev()
        {
            for d in (1..DEGREE)
            {
                *(iter.next()).unwrap() = (i * skip_every - d * step_by - 1) as usize;
            }
        }
        step_by = skip_every;
        skip_every *= DEGREE;
    }

    return Ok(ordering);
}

//way slower, 48us.
#[rustfmt::skip = "ugly"]
fn GE_ordering8<const DEGREE: i64>(len: usize) -> Result<Vec<usize>>
{
    if DEGREE < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, DEGREE as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              DEGREE,
              len
        );
    }

    let mut ordering = vec![0; len];
    let mut iter = ordering.iter_mut();
    let mut skip_every = DEGREE;
    let mut step_by = 1i64;
    for _ in 0..depth
    {
        (0..(len as i64 + 1) / skip_every).rev()
        .for_each(|i| 
        {
            (&mut iter).zip((0..DEGREE - 1).into_iter())
                       .for_each(|(dst, d)| 
                        {
                            *dst = (i * skip_every - d * step_by) as usize;   
                        })
        });
        step_by = skip_every;
        skip_every *= DEGREE;
    }

    return Ok(ordering);
}

//gonna try chunking.
//WOOO 11us
//still not SIMD so i give up.
//its faster on larger DEGREES but slower with DEGREE=2
fn GE_ordering9<const DEGREE: i64>(len: usize) -> Result<Vec<usize>>
{
    if DEGREE < 2
    {
        bail!(" N must be at least 2. ");
    }
    let (valid_len, depth) = power_of_N_minus_one(len as u64, DEGREE as u64);
    if !valid_len
    {
        bail!(
              "N Ordering only valid for arrays with lengths\
         that are powers of N, minus 1. N : {}, length : {}.",
              DEGREE,
              len
        );
    }

    let mut ordering = vec![0; len];
    //let mut iter = ordering.iter_mut().rev();
    let mut skip_every = DEGREE;
    let chunk_len = DEGREE as usize - 1;
    let mut chunk_start = len;
    let mut step_by = 1i64;
    for _ in 0..depth
    {
        for i in (1..(len as i64 + 1) / skip_every + 1).rev()
        {
            chunk_start -= chunk_len;
            let chunk = &mut ordering[chunk_start..chunk_start + chunk_len];
            for d in (1..DEGREE)
            {
                chunk[(DEGREE - d - 1) as usize] = (i * skip_every - d * step_by - 1) as usize;
                //*(iter.next()).unwrap() = (i*skip_every-d*step_by - 1) as usize ;
            }
        }
        step_by = skip_every;
        skip_every *= DEGREE;
    }

    return Ok(ordering);
}
