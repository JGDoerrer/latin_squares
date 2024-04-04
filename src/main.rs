use generator::LatinSquareGenerator;

use crate::generator::OrthogonalGenerator;

mod bitset;
mod constants;
mod constraints;
mod generator;
mod latin_square;
mod types;

fn main() {
    let n = 7;

    // let sqs = LatinSquareGenerator::new(n)
    //     // .enumerate()
    //     // .inspect(|(i, _)| {
    //     //     dbg!(i);
    //     // })
    //     // .map(|(_, sq)| sq)
    //     .filter_map(|sq| OrthogonalGenerator::new(sq.clone()).next().map(|s| (sq, s)))
    //     .inspect(|_| {
    //         dbg!(1);
    //     })
    //     .count();
    // dbg!(&sqs);

    let sqs: Vec<_> = LatinSquareGenerator::new(n).collect();

    dbg!(sqs.len());
    let m: Vec<_> = sqs
        .iter()
        .enumerate()
        .inspect(|(i, _)| {
            dbg!(i);
        })
        .filter(|(i, sq)| {
            sqs.iter()
                .skip(*i)
                .find(|other| sq.is_orthogonal_to(other))
                .is_some()
        })
        .inspect(|(a, b)| {
            dbg!(a, b);
        })
        .collect();

    dbg!(m);
}
