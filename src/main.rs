use std::{collections::HashMap, vec};

use latin_square_pair_generator::LatinSquarePairGenerator;

use crate::{
    latin_square_generator::LatinSquareGenerator,
    orthogonal_generator::OrthogonalLatinSquareGenerator,
};

mod bitset;
mod constraints;
mod latin_square;
mod latin_square_generator;
mod latin_square_pair_generator;
mod orthogonal_generator;
mod pair_constraints;

fn main() {
    // let mut all_pairs = vec![];
    // let mut squares: HashMap<_, Vec<_>> = HashMap::new();
    LatinSquarePairGenerator::<10>::new().for_each(|pair| {
        println!("{pair:?}");

        // all_pairs.push(pair);
        // if let Some(sqs) = squares.get_mut(&pair.0) {
        //     sqs.push(pair.1)
        // } else {
        //     squares.insert(pair.0, vec![pair.1]);
        // }
        // if let Some(sqs) = squares.get_mut(&pair.1) {
        //     sqs.push(pair.0)
        // } else {
        //     squares.insert(pair.1, vec![pair.0]);
        // }

        // dbg!(&squares);

        if let Some(third) = OrthogonalLatinSquareGenerator::new(vec![pair.0, pair.1]).next() {
            println!("{third:?}");
        } else {
            println!("nope");
        }
    });
}
