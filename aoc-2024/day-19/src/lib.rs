#![allow(unused_parens)]
mod inputs;
use anyhow::Result;
use inputs::*;

pub fn part1() -> Result<()>
{
    let (available, desired) = load_input();
    let mut solved = 0;
    println!("Pieces : {:?}", available);
    for pattern in desired.iter()
    {
        let plausible: Vec<&'static str> = available.clone()
                                                    .into_iter()
                                                    .filter(|&a| pattern.contains(a))
                                                    .collect();
        match try_make(&plausible, &pattern)
        {
            None =>
            {
                println!("Pattern '{}' not possible.", pattern)
            }
            Some(solution) =>
            {
                println!("Pattern {} solved! : {:?}", pattern, solution);
                solved += 1;
            }
        }
    }
    println!("Total solved : {}", solved);

    return Ok(());
}

pub fn part2() -> Result<()>
{
    let (available, desired) = load_input();
    let mut solved = 0;
    println!("Pieces : {:?}", available);
    for pattern in desired.iter()
    {
        let plausible: Vec<&'static str> = available.clone()
                                                    .into_iter()
                                                    .filter(|&a| pattern.contains(a))
                                                    .collect();
        match try_make_all(&plausible, &pattern, &mut vec![None; pattern.len() + 1])
        {
            0 =>
            {
                println!("Pattern '{}' not possible.", pattern)
            }
            solutions =>
            {
                println!("Pattern '{}' solved {} ways", pattern, solutions);
                solved += solutions;
            }
        }
    }
    println!("Total solved : {}", solved);

    return Ok(());
}

fn try_make(pieces: &[&'static str], desired: &'static str) -> Option<Vec<&'static str>>
{
    if (desired.len() == 0)
    {
        return Some(vec![]);
    }

    let prefixes = pieces.iter().filter(|&p| desired.starts_with(p));

    for &prefix in prefixes
    {
        match try_make(pieces, &desired[prefix.len()..])
        {
            None =>
            {}
            Some(mut solution) =>
            {
                solution.push(prefix);
                return Some(solution);
            }
        }
    }

    return None;
}

// memo[desired.len()] = number of solutions
// None = unknown
fn try_make_all(pieces: &[&'static str], desired: &'static str, memos: &mut Vec<Option<u64>>)
                -> u64
{
    if (desired.len() == 0)
    {
        memos[0] = Some(1);
        return 1;
    }

    match memos[desired.len()]
    {
        None =>
        {}
        Some(memo) => return memo,
    }

    let prefixes = pieces.iter().filter(|&p| desired.starts_with(p));
    let mut solutions = 0;

    for &prefix in prefixes
    {
        match try_make_all(pieces, &desired[prefix.len()..], memos)
        {
            0 =>
            {}
            x =>
            {
                //solution.iter_mut().for_each(|e| e.push(prefix));
                solutions += x;
            }
        }
    }

    memos[desired.len()] = Some(solutions);
    return solutions;
}
