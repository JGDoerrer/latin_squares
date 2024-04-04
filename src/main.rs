use crate::generator::OrthogonalGenerator;

mod bitset;
mod constants;
mod constraints;
mod generator;
mod latin_square;
mod types;

fn main() {
    let sqs = OrthogonalGenerator::<7>::new()
        .inspect(|i| {
            dbg!(i);
        })
        .next();
    dbg!(&sqs);
}
