use std::vec;

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
    // LatinSquarePairGenerator::<10>::new().for_each(|pair| {
    //     println!("{pair:?}");

    //     if let Some(third) = OrthogonalLatinSquareGenerator::new(vec![pair.0, pair.1]).next() {
    //         println!("{third:?}");
    //     } else {
    //         println!("nope");
    //     }
    // });
    dbg!(LatinSquareTripleGenerator::<10>::new().next());
}
