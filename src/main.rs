use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{stdin, BufRead, BufReader, Read},
    path::Path,
    vec,
};

use clap::{self, Parser, Subcommand};

use latin_square::LatinSquare;
use latin_square_oa_generator::LatinSquareOAGenerator;
use orderly_sq_generator::OrderlySqGenerator;

use partial_latin_square::PartialLatinSquare;
use random_latin_square_generator::RandomLatinSquareGenerator;

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
mod random_latin_square_generator;

#[derive(Subcommand, Clone)]
enum Mode {
    Analyse,
    PrettyPrint,
    NormalizeParatopy,
    GenerateParatopyClasses,
    FindSCS,
    Testing,
    GenerateLatinSquares,
    RandomLatinSquares,
    FindOrthogonal,
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

const N: usize = 6;

fn main() {
    let args = Args::parse();

    match args.mode {
        Mode::Analyse => analyse(),
        Mode::GenerateLatinSquares => generate_latin_squares(),
        Mode::PrettyPrint => pretty_print(),
        Mode::NormalizeParatopy => normalize_paratopy(),
        Mode::GenerateParatopyClasses => generate_paratopy_classes(),
        Mode::FindSCS => find_scs(),
        Mode::Testing => testing(),
        Mode::RandomLatinSquares => random_latin_squares(),
        Mode::FindOrthogonal => find_orthogonal(),
    }
}

fn find_orthogonal() {
    for _sq in read_sqs_from_stdin() {
        todo!()
    }
}

fn random_latin_squares() {
    for sq in RandomLatinSquareGenerator::<N>::new() {
        println!("{}", sq.to_string());
    }
}

fn analyse() {
    for sq in read_partial_sqs_from_stdin() {
        let sq = sq.sort_entries_top_left();
        if !sq.has_entry_determined_by_row_col() {
            for i in 0..N {
                for j in 0..N {
                    if let Some(value) = sq.get(i, j) {
                        print!("{} ", value);
                    } else {
                        print!(". ");
                    }
                }
                println!()
            }
            println!()
        }
    }
}

fn generate_latin_squares() {
    for sq in OrderlySqGenerator::<N>::new() {
        println!("{}", sq.to_string());
    }
}

fn pretty_print() {
    for sq in read_partial_sqs_from_stdin() {
        for i in 0..N {
            println!("+{}", "---+".repeat(N));
            print!("|");
            for j in 0..N {
                if let Some(value) = sq.get(i, j) {
                    print!(" {} |", value);
                } else {
                    print!("   |");
                }
            }
            println!()
        }
        println!("+{}", "---+".repeat(N));
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

    for sq in OrderlySqGenerator::<N>::new() {
        let sq: LatinSquare<N> = sq.into();
        let normalized = sq.reduced_paratopic();

        if !sqs.contains(&normalized) {
            sqs.insert(normalized);

            println!("{}", normalized.to_string());
            dbg!(sqs.len());
        } else {
            dbg!(normalized);
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

fn find_scs() {
    let mut min = N * N;
    let _con = N * N / 4;

    for sq in read_sqs_from_stdin() {
        println!("{}", sq.to_string());

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        'l: for i in 0..=N * N {
            dbg!(i);
            let partial_squares = HittingSetGenerator::new(sq, unavoidable_sets.clone(), i);

            let mut found = false;
            let mut scs = HashSet::new();
            for partial_sq in partial_squares {
                // dbg!(partial_sq);
                let mut solutions = LatinSquareOAGenerator::from_partial(partial_sq);
                let first_solution = solutions.next();
                let second_solution = solutions.next();

                if second_solution.is_none()
                    && first_solution.is_some_and(|solution| solution[0] == sq)
                {
                    // println!("{}", partial_sq.to_string());

                    min = min.min(partial_sq.num_entries());
                    found = true;
                    scs.insert(partial_sq);
                    // break;
                }
            }

            if found {
                for scs in scs {
                    println!("{}", scs.to_string());
                }
                break;
            }
        }
        println!();
    }

    println!("min: {min}");
}

fn testing() {
    for sq in read_partial_sqs_from_stdin() {
        // println!("{}", sq.to_string());
        println!("{}", sq.sort_entries_top_left().to_string());
    }
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

fn read_partial_sqs_from_stdin() -> impl Iterator<Item = PartialLatinSquare<N>> {
    let mut line = String::new();

    (0..).map_while(move |_| {
        if stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line.pop(); // remove newline
            let sq = PartialLatinSquare::try_from(line.as_str()).unwrap();
            line.clear();
            Some(sq)
        } else {
            None
        }
    })
}
