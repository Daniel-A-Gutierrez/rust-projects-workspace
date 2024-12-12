use linked_list::{LinkedList, CursorMut};
use std::collections::HashMap;
use std::mem::replace;
use std::{fs, io::Read};
use anyhow::Result;

pub fn part_one() -> Result<()>
{
    test_num_digits();
    let mut stones = load_input()?;
    println!("stones : {:?}", stones);
    for i in 0..25 { step(&mut stones); }
    println!("Count after {} iterations : {}", 25,  stones.len());
    return Ok(());
}

//silent integer overflow errors HAHAHAHAHA
pub fn part_two() -> Result<()>
{
    test_num_digits();
    let stones = load_input()?;
    let sum = just_count_them(&stones, 75);
    println!("{}", sum);
    return Ok(());
}

fn step(stones : &mut Vec<u64>)
{
    let mut new_stones : Vec<u64> = Vec::with_capacity(stones.capacity());
    stones.iter().for_each(|value| 
    {
        match value 
        {
            0 => new_stones.push(1),
            _ if (num_digits(*value) % 2 == 0 ) => 
            {
                let (left,right) = split_number(*value);
                new_stones.push(left);
                new_stones.push(right);
            },
            _ => new_stones.push(2024*value)
        };
    });
    let _ = replace(stones, new_stones);
}

fn step_in_place(stones : &mut LinkedList<u64>)
{
    let mut cursor = stones.cursor_mut(); //just gonna assume it starts at 0
    cursor.move_next();
    while let Some(value) = cursor.current()
    {
        match *value 
        {
            0 => *value = *value + 1,
            _ if (num_digits(*value) % 2 == 0 ) => 
            {
                let (left,right) = split_number(*value);
                *value = left;
                insert_after(&mut cursor, right);
            },
            _ => { *value = 2024*(*value); }
        };
        cursor.move_next();
    };
}

fn just_count_them(stones :&Vec<u64>, iterations : u32) -> u64
{
    let mut sum = 0;
    let mut memos = HashMap::<(u64,u32), u64>::new();
    stones.iter().for_each(|s| 
    {
        sum += recursive_helper(&mut memos, *s, 0, iterations);
    });
    return sum;
}

// memos need to be indexed by (stone, iter)
fn recursive_helper(memos : &mut HashMap<(u64,u32),u64>, stone : u64, iter : u32, limit : u32) -> u64
{
    if ( iter >= limit ) {return 1;}
    if let Some(value) = memos.get(&(stone,iter)) { return *value; }
    let value = match stone 
    {
        0 => recursive_helper(memos, 1, iter + 1, limit),
        _ if (num_digits(stone) % 2 == 0) => 
        {
            let (left,right) = split_number(stone);
            let l_value = recursive_helper(memos, left, iter + 1, limit);
            let r_value = recursive_helper(memos, right, iter + 1, limit);
            l_value + r_value
        },
        _ => { recursive_helper(memos, stone * 2024, iter + 1, limit) } 
    };
    memos.entry((stone,iter)).or_insert(value);
    return value;
}

fn insert_after(cursor: &mut CursorMut<u64>, val: u64) 
{
    let mut temp_list = LinkedList::new();
    temp_list.extend([val]);
    cursor.splice_after(temp_list);
}

fn num_digits(u : u64) -> u32
{
    return match u 
    {   
        0..=1 => 1,
        _ => (u as f64).log10() as u32 + 1
    };
}

fn split_number(u : u64) -> (u64,u64)
{
    let digits = num_digits(u);
    assert!(digits%2 == 0 , "Cannot split a number with an odd number of digits.");
    let right = u % 10u64.pow(digits/2);
    let left = u / 10u64.pow(digits/2);
    return (left,right)
}

fn legacy_split_number(u : u32) -> (u32,u32)
{
    let s = u.to_string();
    return (s[0..s.len()/2].parse().unwrap(), s[s.len()/2..].parse().unwrap());
}

fn load_input() -> Result<Vec<u64>>
{
    let mut file = fs::OpenOptions::new().read(true).open("aoc-2024/day-11/input.txt")?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let stone_inscriptions = text.split(" ");
    let stones : Vec<u64> = stone_inscriptions.map(|i| i.parse()
        .expect(&format!("{} could not be parsed as a number.",i)))
        .collect();
    return Ok(stones);
}

fn test_num_digits()
{
    assert!(num_digits(0) == 1);
    assert!(num_digits(10) == 2);
    assert!(num_digits(100) == 3);
    assert!(num_digits(1000) == 4);
    assert!(num_digits(10000) == 5);
    assert!(num_digits(100000) == 6);
    assert!(num_digits(1000000) == 7);
    assert!(num_digits(10000000) == 8);
    assert!(num_digits(100000000) == 9);
    assert!(num_digits(1000000000) == 10);
}