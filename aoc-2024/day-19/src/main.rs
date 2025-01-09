#![feature(test)]
use library::{part1, part2};
fn main() -> Result<(), anyhow::Error>
{
    part1()?;
    part2()
}

#[cfg(test)]
mod benchmarks
{
    extern crate test;
    use super::*;
    use test::Bencher;

    #[bench]
    fn day_19_bench_part1(b: &mut Bencher)
    {
        b.iter(|| part1());
    }

    #[bench]
    fn day_19_bench_part2(b: &mut Bencher)
    {
        b.iter(|| part2());
    }
}
