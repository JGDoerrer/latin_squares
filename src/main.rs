use std::{num::NonZeroUsize, thread};

use latin_square::LatinSquare;
use latin_square_pair_generator::LatinSquarePairGenerator;
use latin_square_triple_generator::LatinSquareTripleGenerator;

use crate::{
    latin_square_oa_generator::LatinSquareOAGenerator,
    orthogonal_generator::OrthogonalLatinSquareGenerator,
};

mod bitset;
mod constraints;
mod latin_square;
mod latin_square_generator;
mod latin_square_oa_generator;
mod latin_square_pair_generator;
mod latin_square_triple_generator;
mod orthogonal_array;
mod orthogonal_generator;
mod pair_constraints;
mod triple_constraints;

fn main() {
    const N: usize = 7;

    // let sq1 = LatinSquare::new([
    //     [0, 7, 8, 6, 9, 3, 5, 4, 1, 2],
    //     [6, 1, 7, 8, 0, 9, 4, 5, 2, 3],
    //     [5, 0, 2, 7, 8, 1, 9, 6, 3, 4],
    //     [9, 6, 1, 3, 7, 8, 2, 0, 4, 5],
    //     [3, 9, 0, 2, 4, 7, 8, 1, 5, 6],
    //     [8, 4, 9, 1, 3, 5, 7, 2, 6, 0],
    //     [7, 8, 5, 9, 2, 4, 6, 3, 0, 1],
    //     [4, 5, 6, 0, 1, 2, 3, 7, 8, 9],
    //     [1, 2, 3, 4, 5, 6, 0, 9, 7, 8],
    //     [2, 3, 4, 5, 6, 0, 1, 8, 9, 7],
    // ]);

    // let sq2 = LatinSquare::new([
    //     [0, 4, 1, 7, 2, 9, 8, 3, 6, 5],
    //     [8, 1, 5, 2, 7, 3, 9, 4, 0, 6],
    //     [9, 8, 2, 6, 3, 7, 4, 5, 1, 0],
    //     [5, 9, 8, 3, 0, 4, 7, 6, 2, 1],
    //     [7, 6, 9, 8, 4, 1, 5, 0, 3, 2],
    //     [6, 7, 0, 9, 8, 5, 2, 1, 4, 3],
    //     [3, 0, 7, 1, 9, 8, 6, 2, 5, 4],
    //     [1, 2, 3, 4, 5, 6, 0, 7, 8, 9],
    //     [2, 3, 4, 5, 6, 0, 1, 8, 9, 7],
    //     [4, 5, 6, 0, 1, 2, 3, 9, 7, 8],
    // ]);

    // assert!(sq1.is_orthogonal_to(&sq2));

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
    let triple = LatinSquareOAGenerator::new().next();
    // let triple = LatinSquareOAGenerator::load()
    //     .unwrap_or(LatinSquareOAGenerator::new())
    //     .next();
    println!("{triple:?}");
    // }));
    // }

    // threads
    //     .into_iter()
    //     .for_each(|thread| thread.join().unwrap());
}
