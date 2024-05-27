use clap::{self, Parser};

use latin_square_oa_generator::LatinSquareOAGenerator;

use crate::hitting_set_generator::HittingSetGenerator;

mod bitset;
mod bitvec;
mod compressed_latin_square;
mod constraints;
mod hitting_set_generator;
mod latin_square;
mod latin_square_generator;
mod latin_square_oa_generator;
mod orthogonal_array;
mod orthogonal_generator;
mod partial_square_generator;
mod permutation;

#[derive(Parser)]
struct Args {}

fn main() {
    let _args = Args::parse();

    find_min_entries_per_sq();

    // LatinSquareOAGenerator::load("0,3,0,0,1,1,2,0,0,2,0,1,1,3,0,1,1,1,1,1,1,1,0,1,1,1".to_string())
    //     .unwrap()
    // LatinSquareOAGenerator::new().for_each(|pair| println!("{pair:?}"));

    // generate_5_graph();
    // generate_7_graph();

    // LatinSquarePairGenerator::<N>::new().for_each(|pair| {
    //     println!("{pair:?}");

    // if let Some(third) = OrthogonalLatinSquareGenerator::new(vec![sq1, sq2]).next() {
    //     println!("{third:?}");
    // } else {
    //     println!("nope");
    // }
    // });

    // let mut threads = vec![];
    // for i in 0..thread::available_parallelism()
    //     .unwrap_or(NonZeroUsize::new(1).unwrap())
    //     .get()
    // threads.push(thread::spawn(move || {
    // let triple = LatinSquareTripleGenerator::<N>::load()
    //     .unwrap_or(LatinSquareTripleGenerator::new())
    //     .next();
    // let triple = LatinSquareTripleGenerator::<N>::new().next();
    // let sq = LatinSquareOAGenerator::new().next();
    // let sq = LatinSquareGenerator::<9>::new().next().unwrap();
    // println!("{sq:?}");

    // let triple = LatinSquareOAGenerator::load(std::env::args().nth(1).unwrap_or(String::new()))
    //     .unwrap_or(LatinSquareOAGenerator::new())
    //     .next();
    // println!("{triple:?}");
    // }));
    // }

    // threads
    //     .into_iter()
    //     .for_each(|thread| thread.join().unwrap());
}

fn find_min_entries_per_sq() {
    const N: usize = 9;

    let mut min = N * N;
    let con = N * N / 4;

    let mut sqs = vec![];

    for sq in LatinSquareOAGenerator::<N>::new_reduced_diagonal() {
        let sq = sq[0];
        let sq = sq.reduced_isotopic();

        if !sqs.contains(&sq) {
            sqs.push(sq);
            dbg!(sq, sqs.len());
        }
    }

    for sq in sqs {
        dbg!(sq);

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        'l: for i in (0..=con).rev() {
            // println!("{i}");
            let partial_squares = HittingSetGenerator::new(sq, unavoidable_sets.clone(), i);

            for partial_sq in partial_squares {
                // dbg!(partial_sq);
                let mut solutions = LatinSquareOAGenerator::from_partial(partial_sq);
                let first_solution = solutions.next();
                let second_solution = solutions.next();

                if second_solution.is_none()
                    && first_solution.is_some_and(|solution| solution[0] == sq)
                {
                    println!("{sq:?}");
                    println!("{partial_sq:?}");
                    println!("{}", partial_sq.num_entries());

                    min = min.min(partial_sq.num_entries());
                    continue 'l;
                }
            }

            break;
        }
    }

    println!("min: {min}");
}
