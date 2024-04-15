use std::{num::NonZeroUsize, thread};

use latin_square::LatinSquare;
use latin_square_pair_generator::LatinSquarePairGenerator;
use latin_square_triple_generator::LatinSquareTripleGenerator;

use crate::orthogonal_generator::OrthogonalLatinSquareGenerator;

mod bitset;
mod constraints;
mod latin_square;
mod latin_square_generator;
mod latin_square_pair_generator;
mod latin_square_triple_generator;
mod orthogonal_generator;
mod pair_constraints;
mod triple_constraints;

fn main() {
    const N: usize = 10;

    // LatinSquarePairGenerator::<N>::new().for_each(|pair| {
    //     println!("{pair:?}");

    //     if let Some(third) = OrthogonalLatinSquareGenerator::new(vec![pair.0, pair.1]).next() {
    //         println!("{third:?}");
    //     } else {
    //         println!("nope");
    //     }
    // });

    // let mut threads = vec![];
    // for i in 0..thread::available_parallelism()
    //     .unwrap_or(NonZeroUsize::new(1).unwrap())
    //     .get()
    // threads.push(thread::spawn(move || {
    let triple = LatinSquareTripleGenerator::<N>::load()
        .unwrap_or(LatinSquareTripleGenerator::new())
        .next();
    println!("{triple:?}");
    // }));
    // }

    // threads
    //     .into_iter()
    //     .for_each(|thread| thread.join().unwrap());
}
