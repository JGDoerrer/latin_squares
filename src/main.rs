use latin_square_pair_generator::LatinSquarePairGenerator;

use crate::latin_square_generator::LatinSquareGenerator;

mod bitset;
mod constraints;
mod latin_square;
mod latin_square_generator;
mod latin_square_pair_generator;
mod pair_constraints;
mod types;

fn main() {
    dbg!(LatinSquarePairGenerator::<10>::new()
        .inspect(|sq| {
            dbg!(sq);
        })
        .count());
}
