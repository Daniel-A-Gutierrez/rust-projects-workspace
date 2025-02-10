#![allow(unused_parens)]
use anyhow::Result;
use ndarray::{Array, Array2};
use sorted_vec::SortedSet;
use std::{fs, io::Read};
pub fn part_one() -> Result<()>
{
    let matrix = read_topographic_map()?;
    let trailheads = find_trailheads(&matrix);
    let mut sum = 0;
    for trailhead in trailheads
    {
        let mut explorer = Explorer { untraveled: vec![],
                                      position:   trailhead,
                                      map:        &matrix, };
        let score = explorer.score_trailhead(trailhead);
        println!("trailhead : {:?}, score : {}", trailhead, score);
        sum += score;
    }
    println!("sum of all trailheads : {}", sum);
    return Ok(());
}

pub fn part_two() -> Result<()>
{
    let matrix = read_topographic_map()?;
    let trailheads = find_trailheads(&matrix);
    let mut sum = 0;
    for trailhead in trailheads
    {
        let mut explorer = Explorer { untraveled: vec![],
                                      position:   trailhead,
                                      map:        &matrix, };
        let score = explorer.rate_trailhead(trailhead);
        println!("trailhead : {:?}, rating : {}", trailhead, score);
        sum += score;
    }
    println!("sum of all trailheads : {}", sum);
    return Ok(());
}

/// read text file into ndarray
fn read_topographic_map() -> Result<Array2<u8>>
{
    let mut file = fs::OpenOptions::new().read(true)
                                         .open("aoc-2024/day-10/input.txt")?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let width = text.find("\n").expect("No newlines in input.txt");
    let height = text.len() / (width + 1);
    let arr =
        text.lines().fold(Vec::<u8>::new(), |mut a, b| {
                        a.extend(b.chars()
                                  .map(|c| c.to_digit(10).expect("couldn't parse char as digit"))
                                  .map(|c| c as u8)
                                  .collect::<Vec<u8>>());
                        a
                    });
    let matrix = Array::from_shape_vec((height, width), arr)?;
    return Ok(matrix);
}

#[rustfmt::skip]
fn find_trailheads(matrix: &Array2<u8>) -> Vec<(usize, usize)>
{
    return matrix.indexed_iter()
                 .filter_map(|e| { if (*e.1 == 0){Some(e.0)} else{None}})
                 .collect::<Vec<_>>();
}

/// start at a trailhead.
/// make uphill moves while 1 move is possible
/// if more than 1 move is possible, push a tuple of the untaken moves onto a stack
/// continue this until we reach a terminus. if the height is 9, add the peak to a set of peaks
/// pop the last untaken move + position off the stack, take it.
/// if no moves remain, finish.
struct Explorer<'a>
{
    untraveled: Vec<(usize, usize)>,
    position:   (usize, usize),
    map:        &'a Array2<u8>,
}

impl<'a> Explorer<'a>
{
    fn get_moves(&self) -> Vec<(usize, usize)>
    {
        let pos = (self.position.0 as i64, self.position.1 as i64);
        let moves = vec![(pos.0 + 1, pos.1),
                         (pos.0 - 1, pos.1),
                         (pos.0, pos.1 + 1),
                         (pos.0, pos.1 - 1)];
        let moves = moves.into_iter().filter_map(|p| {
                                         if (p.0 > -1 && p.1 > -1)
                                         {
                                             Some((p.0 as usize, p.1 as usize))
                                         }
                                         else
                                         {
                                             None
                                         }
                                     });
        let moves = moves.into_iter()
                         .filter(|point| self.map.get(*point).is_some())
                         .filter(|point| *self.map.get(*point).unwrap() == self.elevation() + 1)
                         .collect();
        return moves;
    }

    fn elevation(&self) -> &u8
    {
        return self.map
                   .get(self.position)
                   .expect("Got elevation at invalid position");
    }

    fn is_at_peak(&self) -> bool
    {
        return *self.elevation() == 9;
    }

    fn move_to(&mut self, pos: (usize, usize))
    {
        self.map
            .get(pos)
            .expect(&format!("Attempted to move to an invalid position : {:?}", pos));
        self.position = pos;
    }

    fn score_trailhead(&mut self, start: (usize, usize)) -> u32
    {
        let mut peaks = SortedSet::new();
        self.position = start;
        assert!(*self.elevation() == 0, "trailhead had a nonzero elevation");
        self.untraveled = self.get_moves();
        while self.untraveled.len() > 0
        {
            let next_move = self.untraveled.pop().unwrap();
            self.move_to(next_move);
            if (self.is_at_peak())
            {
                peaks.push(self.position.clone());
            }
            self.untraveled.extend(self.get_moves());
        }
        return peaks.len() as u32;
    }

    fn rate_trailhead(&mut self, start: (usize, usize)) -> u32
    {
        let mut peaks = Vec::new();
        self.position = start;
        assert!(*self.elevation() == 0, "trailhead had a nonzero elevation");
        self.untraveled = self.get_moves();
        while self.untraveled.len() > 0
        {
            let next_move = self.untraveled.pop().unwrap();
            self.move_to(next_move);
            if (self.is_at_peak())
            {
                peaks.push(self.position.clone());
            }
            self.untraveled.extend(self.get_moves());
        }
        return peaks.len() as u32;
    }
}
