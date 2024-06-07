use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{stdin, BufRead, BufReader, Read},
    path::Path,
};

use clap::{self, Parser, Subcommand};

use latin_square::LatinSquare;
use latin_square_oa_generator::LatinSquareOAGenerator;
use orderly_sq_generator::OrderlySqGenerator;

use crate::hitting_set_generator::HittingSetGenerator;

mod bitset;
mod bitvec;
mod compressed_latin_square;
mod constraints;
mod hitting_set_generator;
mod latin_square;
mod latin_square_generator;
mod latin_square_oa_generator;
mod orderly_sq_generator;
mod orthogonal_array;
mod orthogonal_generator;
mod partial_latin_square;
mod partial_square_generator;
mod permutation;

#[derive(Subcommand, Clone)]
enum Mode {
    PrettyPrint,
    NormalizeParatopy,
    GenerateParatopyClasses,
    FindSCS,
    Testing,
    GenerateLatinSquares,
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

const N: usize = 9;

fn main() {
    let args = Args::parse();

    match args.mode {
        Mode::GenerateLatinSquares => generate_latin_squares(),
        Mode::PrettyPrint => pretty_print(),
        Mode::NormalizeParatopy => normalize_paratopy(),
        Mode::GenerateParatopyClasses => generate_paratopy_classes(),
        Mode::FindSCS => find_scs(),
        Mode::Testing => testing(),
    }
}

fn pretty_print() {
    for sq in read_sqs_from_stdin() {
        for i in 0..N {
            for j in 0..N {
                print!("{} ", sq.get(i, j))
            }
            println!()
        }
        println!()
    }
}

fn normalize_paratopy() {
    for sq in read_sqs_from_stdin() {
        println!("{}", sq.reduced_paratopic().to_string());
    }
}

fn generate_paratopy_classes() {
    // dbg!(LatinSquareOAGenerator::<N>::new_reduced().count(), return);

    let mut sqs = HashSet::new();

    for sq in OrderlySqGenerator::<N>::new_diagonal_symmetry() {
        let sq: LatinSquare<N> = sq.into();
        let sq = sq.reduced_paratopic();

        if !sqs.contains(&sq) {
            sqs.insert(sq);

            println!("{}", sq.to_string());
            dbg!(sqs.len());
        }
    }

    // let file = OpenOptions::new()
    //     .write(true)
    //     .truncate(true)
    //     .create(true)
    //     .open(format!("latin_mc{N}.txt"))
    //     .unwrap();
    // let mut writer = BufWriter::new(file);

    // for sq in sqs {
    //     println!("{}", sq.to_string());
    // }
}

fn generate_latin_squares() {
    for sq in OrderlySqGenerator::<N>::new_diagonal_symmetry() {
        println!("{}", sq.to_string());
    }
}

fn find_scs() {
    let mut min = N * N;
    let _con = N * N / 4;

    for sq in read_sqs_from_stdin() {
        println!("{}", sq.to_string());

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        'l: for i in 0..N * N {
            // println!("{i}");
            let partial_squares = HittingSetGenerator::new(sq, unavoidable_sets.clone(), i);

            let mut found = false;
            for partial_sq in partial_squares {
                // dbg!(partial_sq);
                let mut solutions = LatinSquareOAGenerator::from_partial(partial_sq);
                let first_solution = solutions.next();
                let second_solution = solutions.next();

                if second_solution.is_none()
                    && first_solution.is_some_and(|solution| solution[0] == sq)
                {
                    println!("{}", partial_sq.to_string());

                    min = min.min(partial_sq.num_entries());
                    found = true;
                    break;
                }
            }

            if found {
                break;
            }
        }
        println!();
    }

    println!("min: {min}");
}

fn testing() {
    let sqs: Vec<_> = read_sqs_from_stdin().collect();
    dbg!(&sqs);

    let mut pairs: Vec<_> = sqs
        .iter()
        .map(|sq| {
            let min_diff = sqs
                .iter()
                .filter(|other| *other != sq)
                .min_by_key(|other| sq.difference_mask(&other).len())
                .unwrap();

            (sq, min_diff, sq.difference_mask(min_diff).len())
        })
        .collect();

    pairs.sort_by_key(|(_, _, i)| *i);

    pairs.into_iter().for_each(|(sq, other, _)| {
        println!("{}", sq.to_string());
        println!("{}", other.to_string());
        println!("{:?}", sq.difference_mask(other).len());
        println!();
    })
}

fn read_sqs_from_file(path: &Path) -> Vec<LatinSquare<N>> {
    let file = OpenOptions::new().read(true).open(path).unwrap();

    let mut reader = BufReader::new(file);

    let mut line = String::new();
    let mut sqs = vec![];
    while reader.read_line(&mut line).is_ok_and(|i| i != 0) {
        line.pop(); // remove newline
        let sq = LatinSquare::try_from(line.as_str()).unwrap();
        sqs.push(sq);
        line.clear();
    }
    sqs
}

fn read_sqs_from_stdin() -> impl Iterator<Item = LatinSquare<N>> {
    let mut line = String::new();

    (0..).map_while(move |_| {
        if stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line.pop(); // remove newline
            let sq = LatinSquare::try_from(line.as_str()).unwrap();
            line.clear();
            Some(sq)
        } else {
            None
        }
    })
}
