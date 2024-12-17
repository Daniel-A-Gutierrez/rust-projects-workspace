#![allow(unused_parens)]

use std::{collections::HashMap, fs::OpenOptions, io::Read};

use anyhow::Result;
use ndarray::Array2;

#[derive(Default, Debug, Clone, Copy)]
struct Plot
{
    region:    (char, u32),
    perimiter: u32,
}

type Regions = HashMap<(char, u32), Vec<Vec<(usize, usize)>>>;

pub fn run() -> Result<()>
{
    let matrix = load()?;
    let (regions, plots) = scan_regions(&matrix)?;
    println!("Total Cost : {}", calculate_cost(&regions, &plots));
    let discounted_cost = calculate_discounted_cost(&regions, &plots, &matrix);
    println!("Discounted Cost : {}", discounted_cost);
    return Ok(());
}

fn load() -> Result<(Array2<char>)>
{
    let mut file = OpenOptions::new().read(true)
                                     .open("aoc-2024/day-12/input.txt")?;
    let mut contents = String::with_capacity(20000);
    file.read_to_string(&mut contents)?;
    let width = contents.find("\n")
                        .expect("Couldn't find a newline in the input.");
    let chars = Vec::from_iter(contents.chars().filter(|c| c.is_alphabetic()));
    let height = chars.len() / (width);
    let matrix = Array2::from_shape_vec((height, width), chars)?;
    return Ok(matrix);
}

fn scan_regions(matrix: &Array2<char>) -> Result<(Regions, Array2<Plot>)>
{
    let mut regions_counter = [0; 26];
    let mut regions = HashMap::<(char, u32), Vec<Vec<(usize, usize)>>>::new();
    let shape = (*matrix.shape().get(0).unwrap(), *matrix.shape().get(1).unwrap());
    let mut plots = Array2::from_shape_fn(shape, |_| Plot::default());
    for e in matrix.indexed_iter()
    {
        let character = *e.1;
        let pos = e.0;
        let br_perimeter = check_unvisited(&matrix, pos, character);
        process_visited(&mut plots,
                        pos,
                        character,
                        br_perimeter,
                        &mut regions,
                        &mut regions_counter)?;
    }
    return Ok((regions, plots));
}

fn calculate_cost(regions: &Regions, plots: &Array2<Plot>) -> u64
{
    let mut total_cost: u64 = 0;
    for region in regions
    {
        let flat: Vec<(usize, usize)> = region.1.iter().flatten().map(|&e| e).collect();
        let area = flat.len();
        let perim = flat.iter()
                        .fold(0, |a, p| a + plots.get(*p).unwrap().perimiter);
        let cost = area as u64 * perim as u64;
        total_cost += cost;
        // println!("Region : {:?}, Perimeter : {} , Area : {}",
        //          region.0, perim, area);
    }
    return total_cost;
}

fn calculate_discounted_cost(regions: &Regions, plots: &Array2<Plot>, matrix: &Array2<char>)
                             -> u64
{
    // println!("Calculating Discounted Cost");
    let mut d_cost = 0;
    for region in regions
    {
        let flat_region: Vec<&(usize, usize)> = region.1.iter().flatten().collect();
        let num_corners = scan_corners(matrix, &flat_region);
        let area = flat_region.len();
        // println!("Region : {:?}, Number Of Sides: {}, Area: {}",
        //          region.0, num_corners, area);
        d_cost += area as u64 * num_corners;
    }
    return d_cost;
}

fn scan_corners(matrix: &Array2<char>, region: &Vec<&(usize, usize)>) -> u64
{
    let mut corners = 0;
    //let mut log_grid = Array2::from_shape_fn((6,6), |_| 0);
    for &&pos in region
    {
        let character = matrix.get(pos).unwrap();
        let top = (pos.0 as i64 - 1, pos.1 as i64);
        let topleft = (pos.0 as i64 - 1, pos.1 as i64 - 1);
        let left = (pos.0 as i64, pos.1 as i64 - 1);
        let bottomleft = (pos.0 as i64 + 1, pos.1 as i64 - 1);
        let bottom = (pos.0 as i64 + 1, pos.1 as i64);
        let bottomright = (pos.0 as i64 + 1, pos.1 as i64 + 1);
        let right = (pos.0 as i64, pos.1 as i64 + 1);
        let topright = (pos.0 as i64 - 1, pos.1 as i64 + 1);
        let adjacent = [top,
                        topleft,
                        left,
                        bottomleft,
                        bottom,
                        bottomright,
                        right,
                        topright];
        let matches: [bool; 8] =
            adjacent.iter()
                    .map(|p| get_checked(&p, matrix).filter(|c| *c == character))
                    .map(|e| e.is_some())
                    .collect::<Vec<bool>>()
                    .try_into()
                    .unwrap();
        let [top, topleft, left, bottomleft, bottom, bottomright, right, topright] = matches;
        //look for corners
        let c = [tl(left, topleft, top),
                 tr(right, topright, top),
                 bl(left, bottomleft, bottom),
                 br(right, bottomright, bottom)];
        let num_corners = c.iter().filter(|x| **x).count() as u64;
        //log_grid[pos] = num_corners; 
        corners += num_corners;
    }
    //println!("{:?}"  , log_grid);
    return corners;
}

#[inline]
fn tl(left: bool, topleft: bool, top: bool) -> bool
{
    return (left & !topleft & top) || !(left | top);
}

#[inline]
fn tr(right: bool, topright: bool, top: bool) -> bool
{
    return (top & !topright & right) || !(top | right);
}

#[inline]
fn bl(left: bool, bleft: bool, bottom: bool) -> bool
{
    return (bottom & !bleft & left) || !(bottom | left);
}

#[inline]
fn br(right: bool, bright: bool, bottom: bool) -> bool
{
    return (bottom & !bright & right) || !(right | bottom);
}
/// check the groups that this cell could join in the top and left positions.
/// if there are groups to join, attempt to join or merge them.
/// returns perimeter needed for top and left positions.
fn get_visited_in_group(m: &Array2<Plot>,
                        pos: (usize, usize),
                        character: char)
                        -> (Option<Plot>, Option<Plot>)
{
    let pos = (pos.0 as i64, pos.1 as i64);
    let top_pos = (pos.0 - 1, pos.1);
    let left_pos = (pos.0, pos.1 - 1);
    let top_plot = get_checked(&top_pos, m).filter(|tplot| tplot.region.0 == character)
                                           .cloned();
    let left_plot = get_checked(&left_pos, m).filter(|lplot| lplot.region.0 == character)
                                             .cloned();
    return (top_plot, left_plot);
}

fn process_visited(plots: &mut Array2<Plot>,
                   pos: (usize, usize),
                   character: char,
                   br_perimeter: u32, //perimeter from bottom and right only
                   regions: &mut Regions,
                   regions_counter: &mut [u32; 26])
                   -> Result<()>
{
    let (top_plot, left_plot) = get_visited_in_group(plots, pos, character);
    match (top_plot, left_plot)
    {
        (None, None) =>
        {
            // make a new group, join it.
            let region_id = &mut regions_counter[alphabet_number(character)?];
            let region = (character, *region_id);
            regions.insert(region, vec![vec![pos]]);
            *region_id += 1;
            init_plot(plots, pos, region, br_perimeter + 2);
        }
        (Some(p), None) | (None, Some(p)) =>
        {
            // join group
            let region = p.region;
            let perimeter = br_perimeter + 1;
            regions.get_mut(&region)
                   .unwrap()
                   .get_mut(0)
                   .unwrap()
                   .push(pos);
            init_plot(plots, pos, region, perimeter);
        }
        (Some(p1), Some(p2)) if (p1.region != p2.region) =>
        {
            //merge groups

            //this would be a good place to do a 'union find' approach but idk if i need it yet.
            //instead of changing each plot, i just have a hashmap that indicates links between regions.

            let region2 = regions.remove(&p2.region).unwrap();
            let region1 = regions.get_mut(&p1.region).unwrap();
            for subregion in &region2
            {
                for coord in subregion
                {
                    plots.get_mut(*coord).unwrap().region = p1.region;
                }
            }
            region1.extend(region2);
            let perimeter = br_perimeter;
            let region = &p1.region;
            regions.get_mut(&region)
                   .unwrap()
                   .get_mut(0)
                   .unwrap()
                   .push(pos);
            init_plot(plots, pos, *region, perimeter);
        }
        (Some(p), Some(_)) =>
        {
            //dont need to merge, just join .
            let region = p.region;
            let perimeter = br_perimeter;
            regions.get_mut(&region)
                   .unwrap()
                   .get_mut(0)
                   .unwrap()
                   .push(pos);
            init_plot(plots, pos, region, perimeter);
        }
    };
    return Ok(());
}

fn init_plot(plots: &mut Array2<Plot>, pos: (usize, usize), region: (char, u32), perimeter: u32)
{
    let p = plots.get_mut(pos).unwrap();
    p.perimiter = perimeter;
    p.region = region;
}

#[rustfmt::skip]
fn get_checked<'a, 'b, T>(idx: &'a (i64, i64), m: &'b Array2<T>) -> Option<&'b T>
{
    if (idx.0 < 0 || idx.1 < 0) { return None; }
    else { return m.get((idx.0 as usize, idx.1 as usize)); }
}

/// count the matching characters in the bottom and right positions
fn check_unvisited(m: &Array2<char>, pos: (usize, usize), character: char) -> u32
{
    let pos = (pos.0 as i64, pos.1 as i64);
    let b_pos = (pos.0 + 1, pos.1);
    let r_pos = (pos.0, pos.1 + 1);
    let b = get_checked(&b_pos, m).filter(|c| **c == character);
    let r = get_checked(&r_pos, m).filter(|c| **c == character);
    return match (b, r)
    {
        (None, None) => 2,
        (Some(_), None) | (None, Some(_)) => 1,
        (Some(_), Some(_)) => 0,
    };
}

/// only valid for uppercase english letters
fn alphabet_number(c: char) -> Result<usize>
{
    let digit = c as u8 - 65;
    assert!(digit < 26, "Integer overflow while parsing characters.");
    return Ok(digit as usize);
}
/*
scan and bucket?

regions are idenitfied by a character and a u32.
regions contain a list of all the indexes within them.
a 'info map' exists that stores additional info for each space
    * how much fencing is on it
    * what region does it belong to

iterate over each space in the array
* if its unvisited
    * is it adjacent to something thats visited with the same character?
        yes => mark this space as part of that region

one pass method
iterate over each space in the array
    * check the surrounding spaces for spaces with the same character that are visited
    of these
    * if theres 1, join its group.
    * if theres more than 1, merge their groups if theyre different and join .
    * if theres none, make a new group and join it.
*/
