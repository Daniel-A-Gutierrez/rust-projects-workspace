//testing the throughput of various horizontal simd ops vs serial.
//result : it seems like horizontal simd is absolutely worth using over serial calculations.
//even if its twice as slow as optimal simd, thats still much faster than serial...
//also the larger the vector the better vertical is vs horizontal.
//gather/scatter are super duper slow. so slow theyre 10x worse than serial. 
use crate::random_numbers;
use std::simd::prelude::*;
#[cfg(test)]
mod test
{
    extern crate test;
    use super::*;
    use test::Bencher;

    const CHUNK_SIZE : usize = 4;

    mod benchmarks
    {
        use super::*;
        //550 ns
        #[bench]
        fn min(b : &mut Bencher)
        {
            let mut sample = random_numbers(4096,0,10_000);
            let mut collector = 0;
            b.iter(||
            {
                collector = *(sample.iter().min().unwrap());
            });
        }

        //not the optimal way to do this ik but its a test.
        //181 ns at chunk size 16.
        //174 ns at chunk size 8
        //300 ns at chunk size 4
        #[bench]
        fn min_simd(b : &mut Bencher)
        {
            
            let sample = random_numbers(4096,0,10_000);
            let mut x = 0;
            b.iter(||
            {
                let chunks = sample.array_chunks::<CHUNK_SIZE>();
                let remainder = chunks.remainder();
                let mut collector = i32::MAX;
                for chunk in chunks 
                {
                    let s = Simd::from_array(*chunk);
                    collector = collector.min(s.reduce_min());
                }
                for r in remainder
                {
                    collector = collector.min(*r);
                }
                x = collector;
            });
        }

        //56ns  at chunk size 16.
        //92 ns at chunk size 8
        //194 ns at chunk size 4.
        #[bench]
        fn min_simd_good(b : &mut Bencher)
        {
            let sample = random_numbers(4096,0,10_000);
            let mut x = 0;
            b.iter(||
            {
                let chunks = sample.array_chunks::<CHUNK_SIZE>();
                let remainder = chunks.remainder();
                let mut collector = Simd::<i32, CHUNK_SIZE>::splat(i32::MAX);
                for chunk in chunks 
                {
                    collector = collector.simd_min(Simd::from_array(*chunk));
                }
                let mut min = collector.reduce_min();
                for r in remainder
                {
                    min = min.min(*r);
                }
                x = min;
            });
        }
        
        //85 ns
        #[bench]
        fn sum(b : &mut Bencher)
        {
            let mut sample = random_numbers(4096,0,10_000);
            let mut collector = 0;
            b.iter(||
            {
                collector = (sample.iter().sum());
            });
        }

        //160 ns at x8'=
        //170ns at x16
        #[bench]
        fn sum_simd(b : &mut Bencher)
        {
            
            let sample = random_numbers(4096,0,10_000);
            let mut x = 0;
            b.iter(||
            {
                let chunks = sample.array_chunks::<CHUNK_SIZE>();
                let remainder = chunks.remainder();
                let mut collector = 0;
                for chunk in chunks 
                {
                    let s = Simd::from_array(*chunk);
                    collector += (s.reduce_sum());
                }
                for r in remainder
                {
                    collector += (*r);
                }
                x = collector;
            });
        }

        //380ns on saturating add?!
        //58ns on x8 with default add. so saturating add is worse than serial
        //56ns on x16
        #[bench]
        fn sum_simd_good(b : &mut Bencher)
        {
            let sample = random_numbers(4096,0,10_000);
            let mut x = 0;
            b.iter(||
            {
                let chunks = sample.array_chunks::<CHUNK_SIZE>();
                let remainder = chunks.remainder();
                let mut collector = Simd::<i32, CHUNK_SIZE>::splat(0);
                for chunk in chunks 
                {
                    collector = collector + (Simd::from_array(*chunk));
                }
                let mut min = collector.reduce_sum();
                for r in remainder
                {
                    min = (*r);
                }
                x = min;
            });
        }

        //305 ns at x8
        //946 ns at x32
        #[bench]
        fn index_of_min(b : &mut Bencher)
        {
            let sample = random_numbers(4096,0,10_000);
            let mut x = 0;
            b.iter(||
            {
                for chunk in sample.array_chunks::<CHUNK_SIZE>()
                {
                    let mut i = 0 ; 
                    let mut min = i32::MIN;
                    for e in 0..CHUNK_SIZE 
                    {
                        min = chunk[e].min(min);
                        i = (min == chunk[e]).select_unpredictable(e,i);
                    }
                    x += i;
                }
            });
        }


        //222 ns at x8 - faster but not much.
        //270ns at x16
        //201 at x32
        #[bench]
        fn index_of_min_hor(b : &mut Bencher)
        {
            let sample = random_numbers(4096,0,10_000);
            let mut x = 0;
            b.iter(||
            {
                for chunk in sample.array_chunks::<CHUNK_SIZE>()
                {
                    let s = Simd::from_array(*chunk);
                    let min = Simd::splat(s.reduce_min());
                    let i = min.simd_eq(s).to_bitmask().leading_zeros() - (64-CHUNK_SIZE) as u32;
                    x += i;
                }
            });
        }

        //57 us finding the index of the minimum array with simd. 
        //wow thats slow as fuck!
        //6 us at x4
        #[bench]
        fn gathering_min_hor(b : &mut Bencher)
        {
            let mut samples = [ [0; 4096] ; CHUNK_SIZE];
            for arr in &mut samples 
            {
                arr.copy_from_slice(&random_numbers(4096,0,10_000));
            }

            let mut x = 0;
            let ptrs = Simd::from_array(samples.map(|a| a.as_ptr()));
            b.iter(||
            {
                for i in 0..4096
                {
                    let i = Simd::splat(i);
                    // safe because I is bounded by fixed array size, 4096. 
                    let s = unsafe { Simd::gather_ptr(ptrs.wrapping_add(i)) };
                    let min = Simd::splat(s.reduce_min());
                    let mi = min.simd_eq(s).to_bitmask().leading_zeros() - (64-CHUNK_SIZE) as u32;
                    x += mi;
                }
            });
        }

        //553 ns at x4 - much better. still 20% slower than serial.
        //8000 ns at x16
        //6681 ns optimized to not preload the matrix...
        //6099 ns saving reduce sum for the very end.
        //16 us at chunk size 32, making it 2.5x faster than serial at this large size. 
        //19.5 us replacing assign with to_int() , 19.7 when we multiply that by ci.
        //wait actually considering its doing 32x the work, doing it in 16x the time is quite good. 
        //so this is actually working well. it does 4x the work in 5/8ths the time too. ok nevermind im satisfied.
        //so avoid gather at all costs. it makes simd not worth it? 
        #[bench]
        fn gathering_min_square(b : &mut Bencher)
        {
            let mut samples = [ [0; 4096] ; CHUNK_SIZE];
            for arr in &mut samples 
            {
                arr.copy_from_slice(&random_numbers(4096,0,10_000));
            }

            let mut x = 0;
            b.iter(||
            {
                let mut sums = Simd::splat(0);
                for i in (0..4096).step_by(CHUNK_SIZE)
                {
                    //let matrix : [Simd<_,CHUNK_SIZE>;  CHUNK_SIZE] = std::array::from_fn(|ci| Simd::from_slice(&samples[ci][i..i+CHUNK_SIZE]));
                    let mut minimums : Simd<i32, CHUNK_SIZE> = Simd::from_slice(&samples[0][i..i+CHUNK_SIZE]);
                    let mut indexes = Simd::splat(0);
                    for ci in 1..CHUNK_SIZE
                    {
                        let row = Simd::from_slice(&samples[ci][i..i+CHUNK_SIZE]);
                        minimums = row.simd_min(minimums);
                        let assign = minimums.simd_eq(row);
                        indexes = assign.select( Simd::splat(ci as i32), indexes );
                    }
                    sums += indexes;
                }
                x += sums.reduce_sum();
            });
        }

        // 42 us at x32
        // 432 at x4
        // 5211 ns at x16
        #[bench]
        fn gathering_min(b : &mut Bencher)
        {
            let mut samples = [ [0; 4096] ; CHUNK_SIZE];
            for arr in &mut samples 
            {
                arr.copy_from_slice(&random_numbers(4096,0,10_000));
            }

            let mut x = 0;
            b.iter(||
            {
                for i in 0..4096
                {
                    let mut mi = 0 ; 
                    let mut min = i32::MIN;
                    for c in 0..CHUNK_SIZE
                    {
                        min = samples[c][i].min(min);
                        mi = (min == samples[c][i]).select_unpredictable(c,mi);
                    }
                    x += mi;
                }
            });
        }

        const fn reverse_indexes() -> [usize; CHUNK_SIZE]
        {
            let mut o = [0usize ;CHUNK_SIZE];
            let mut i = 0;
            loop 
            {
                o[i] = CHUNK_SIZE - i - 1;
                i += 1;
                if i > (CHUNK_SIZE-1) {return o;}
            }
        }

        //1445 ns
        //1464 ns without reversing - so reversing is basically free here. 
        //1530 ns load or default
        //178 ns GODDAMNIT IT WAS THE PRINTLN
        //so for an operation thats purely load/store , simd isnt worth it. 
        //also smaller registers arent faster. 
        #[bench]
        fn simd_reverse(b : &mut Bencher)
        {
            let mut sample = random_numbers(4096,0,10_000);
            sample.sort();
            //const R_IDX : [usize; CHUNK_SIZE] = reverse_indexes();
            
            b.iter(||
            {
                let mut back = sample.len();
                for i in (0..(4096/2)).step_by(CHUNK_SIZE)
                {
                    back -= CHUNK_SIZE;
                    //println!("{}", back);
                    let to_back = Simd::<_,CHUNK_SIZE>::from_slice(&sample[i..i+CHUNK_SIZE]);
                    let to_front = Simd::<_,CHUNK_SIZE>::from_slice(&sample[back .. back + CHUNK_SIZE]);
                    to_back.reverse().copy_to_slice(&mut sample[back..back+CHUNK_SIZE]);
                    to_front.reverse().copy_to_slice(&mut sample[i..i+CHUNK_SIZE]);
                }
            });
        }

        //1093ns per iter. 
        //so gathering is VASTLY SLOWER even if the things you're gathering are continuous.
        //smaller registers arent faster. 
        #[bench]
        fn gathering_reverse(b : &mut Bencher)
        {
            let mut sample = random_numbers(4096,0,10_000);
            sample.sort();
            const R_IDX : Simd<usize, CHUNK_SIZE> = Simd::from_array(reverse_indexes());

            
            b.iter(||
            {
                let mut back = sample.len();
                for i in (0..(4096/2)).step_by(CHUNK_SIZE)
                {
                    back -= CHUNK_SIZE;
                    //println!("{}", back);
                    let to_back : Simd::<_,CHUNK_SIZE> = unsafe {Simd::gather_ptr(Simd::splat(sample[i..i+CHUNK_SIZE].as_ptr()).wrapping_add( R_IDX ))};
                    let to_front : Simd::<_,CHUNK_SIZE> = unsafe { Simd::gather_ptr(Simd::splat(sample[back..back+CHUNK_SIZE].as_ptr()).wrapping_add(R_IDX)) };
                    to_back.copy_to_slice(&mut sample[back..back+CHUNK_SIZE]);
                    to_front.copy_to_slice(&mut sample[i..i+CHUNK_SIZE]);
                }
            });
        }

        //105ns
        #[bench]
        fn reversing(b : &mut Bencher)
        {
            let mut sample = random_numbers(4096,0,10_000);
            sample.sort();
            //const R_IDX : [usize; CHUNK_SIZE] = reverse_indexes();
            
            b.iter(||
            {
                sample.reverse();
            });
        }

        #[test]
        fn xor_set()
        {
            let v = vec![1,2,3,4,5,];
            let xor = v.iter().skip(1).fold(v[0], |a,b| a^b);
            println!("{}", xor);
        }

    }
}


/*
ok so estimating the relative cost of operations

reverse : 0 lol
add : 1
min : 1
load : 1
store : 1
to_bitmask : 1
leading_zeros : 1
comparison : 1
mask.select : 1
reducing add : log(N)
reducing min : log(N)
saturating add : 7
gather : 20 - 50

loading and storing should be intermixed with computation to give the ram time to work

*/