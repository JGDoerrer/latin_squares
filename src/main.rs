use crate::generator::{LatinSquareGenerator, OrthogonalGenerator, OrthogonalGenerator3};

mod bitset;
mod constants;
mod constraints;
mod generator;
mod latin_square;
mod types;

fn main() {
    let sqs = LatinSquareGenerator::<7>::new()
        .map(|sq| {
            OrthogonalGenerator::new(vec![sq.clone()])
                .next()
                .map(|sq2| vec![sq, sq2])
        })
        .flatten()
        .inspect(|sqs| {
            dbg!(sqs);
        })
        .map(|sqs| {
            OrthogonalGenerator::new(sqs.clone()).next().map(|new_sq| {
                let mut new_sqs = sqs;
                new_sqs.push(new_sq);
                new_sqs
            })
        })
        .flatten()
        .next();
    dbg!(&sqs);
}
