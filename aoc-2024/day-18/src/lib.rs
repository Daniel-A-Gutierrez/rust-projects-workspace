mod input;
use anyhow::{anyhow, Result};
use input::*;
use ndarray::{Array2, Dim};
use pathfinding::prelude::dijkstra;
use rayon::prelude::*;

pub fn part1() -> Result<()>
{
    let mut input = load_input();
    input.truncate(1024);
    let mut map = Array2::from_shape_simple_fn((71, 71), || true);
    drop_bytes(&mut map, &input);

    //debug_solution(&map, &vec![]);
    let (path, length) = find_path(&map, &(70, 70)).ok_or(anyhow!("No path to the exit found!"))?;
    debug_solution(&map, &path);

    println!("Path length : {}", length);
    return Ok(());
}

pub fn part2() -> Result<()>
{
    let input = load_input();

    let doom = (1024..input.len()).into_par_iter()
                                  .by_exponential_blocks()
                                  .find_first(|byte_number| {
                                      let mut map = Array2::from_shape_simple_fn((71, 71), || true);
                                      drop_bytes(&mut map, &input[0..*byte_number]);
                                      !find_path(&map, &(70, 70)).is_some()
                                  })
                                  .expect("The map remained traversible for all dropped bytes.");

    //debug_solution(&map, &vec![]);

    println!("Dooming Byte Index: {:?}\nPosition: {:?}",
             doom - 1,
             (input[doom - 1].1, input[doom - 1].0));
    return Ok(());
}

fn drop_bytes(map: &mut Array2<bool>, bytes: &[(usize, usize)])
{
    bytes.iter().for_each(|&b| map[b] = false);
}

fn find_path(map: &Array2<bool>, destination: &(usize, usize))
             -> Option<(Vec<(usize, usize)>, u32)>
{
    let get_adjacent_traversible = |pos: &(usize, usize)| -> Vec<((usize, usize), u32)> {
        generate_adjacent(&pos).into_iter()
                               .filter(|&p| map.get(p).is_some_and(|b| *b))
                               .map(|p| (p, 1))
                               .collect()
    };

    let found_destination = |pos: &(usize, usize)| *pos == *destination;

    return dijkstra(&(0, 0), get_adjacent_traversible, found_destination);
}

fn generate_adjacent(pos: &(usize, usize)) -> [(usize, usize); 4]
{
    return [(pos.0 + 1, pos.1),
            (pos.0.overflowing_sub(1).0, pos.1),
            (pos.0, pos.1 + 1),
            (pos.0, pos.1.overflowing_sub(1).0)];
}

fn debug_solution(map: &Array2<bool>, path: &Vec<(usize, usize)>)
{
    let mut pretty_map: Array2<char> =
        Array2::from_shape_vec::<Dim<[usize; 2]>>(map.raw_dim(),
                                                  map.iter()
                                                     .map(|x| match x
                                                     {
                                                         false => '#',
                                                         true => '·',
                                                     })
                                                     .collect()).unwrap();
    path.iter().for_each(|e| pretty_map[*e] = 'O');
    let [rows, cols] = pretty_map.shape()
    else
    {
        panic!("AAaah")
    };
    for r in 0..*rows
    {
        for c in 0..*cols
        {
            print!("{}", pretty_map[(r, c)])
        }
        println!("");
    }
    //println!("{:?}", pretty_map);
}
