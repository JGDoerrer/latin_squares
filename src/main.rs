use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{stdin, stdout, BufRead, BufReader, Read, Write},
    path::Path,
    vec,
};

use clap::{self, Parser, Subcommand};

use latin_square::LatinSquare;

use latin_square_generator::LatinSquareGenerator;
use latin_square_oa_generator::LatinSquareOAGenerator;

use partial_latin_square::PartialLatinSquare;

use permutation::PermutationIter;
use random_latin_square_generator::RandomLatinSquareGenerator;
use rc_generator::RCGenerator;
use rcs_generator::RCSGenerator;

use crate::hitting_set_generator::HittingSetGenerator;

mod bitset;
mod bitvec;
mod compressed_latin_square;
mod constraints;
mod hitting_set_generator;
mod latin_square;
mod latin_square_generator;
mod latin_square_oa_generator;
mod orthogonal_array;
mod orthogonal_generator;
mod partial_latin_square;
mod partial_square_generator;
mod permutation;
mod random_latin_square_generator;
mod rc_generator;
mod rcs_generator;
mod tuple_iterator;

#[derive(Subcommand, Clone)]
enum Mode {
    Analyse,
    PrettyPrint,
    NormalizeParatopy,
    GenerateParatopyClasses,
    FindSCS {
        #[arg(long, default_value_t = 0)]
        start: usize,
        #[arg(long, default_value_t = usize::MAX)]
        end: usize,
    },
    GenerateLatinSquares,
    RandomLatinSquares {
        seed: u64,
    },
    FindOrthogonal,
    Solve,
    SortByIntercalates,
    NumSubsquares {
        k: usize,
    },
}

#[derive(Parser)]
struct Args {
    n: usize,
    #[command(subcommand)]
    mode: Mode,
}

fn main() {
    let args = Args::parse();

    macro_rules! match_mode {
        ($N: expr) => {
            match args.mode {
                Mode::Analyse => analyse::<$N>(),
                Mode::GenerateLatinSquares => generate_latin_squares::<$N>(),
                Mode::PrettyPrint => pretty_print::<$N>(),
                Mode::NormalizeParatopy => normalize_paratopy::<$N>(),
                Mode::GenerateParatopyClasses => generate_paratopy_classes::<$N>(),
                Mode::FindSCS { start, end } => find_scs::<$N>(start, end),
                Mode::RandomLatinSquares { seed } => random_latin_squares::<$N>(seed),
                Mode::FindOrthogonal => find_orthogonal::<$N>(),
                Mode::Solve => solve::<$N>(),
                Mode::SortByIntercalates => sort_by_intercalates::<$N>(),
                Mode::NumSubsquares { k } => num_subsquares::<$N>(k),
            }
        };
    }

    match args.n {
        0 => match_mode!(0),
        1 => match_mode!(1),
        2 => match_mode!(2),
        3 => match_mode!(3),
        4 => match_mode!(4),
        5 => match_mode!(5),
        6 => match_mode!(6),
        7 => match_mode!(7),
        8 => match_mode!(8),
        9 => match_mode!(9),
        10 => match_mode!(10),
        11 => match_mode!(11),
        _ => todo!(),
    }
}

fn num_subsquares<const N: usize>(k: usize) {
    for sq in read_sqs_from_stdin::<N>() {
        println!("{}", sq.num_subsquares_dyn(k));
    }
}

fn sort_by_intercalates<const N: usize>() {
    let mut sqs: Vec<_> = read_sqs_from_stdin::<N>().collect();
    sqs.sort_by_key(|a| a.intercalates());
    for sq in sqs {
        if writeln!(stdout(), "{}", sq.to_string()).is_err() {
            return;
        }
    }
}

fn find_orthogonal<const N: usize>() {
    for _sq in read_sqs_from_stdin::<N>() {
        todo!()
    }
}

fn random_latin_squares<const N: usize>(seed: u64) {
    for sq in RandomLatinSquareGenerator::<N>::new(seed) {
        println!("{}", sq.to_string());
    }
}

fn analyse<const N: usize>() {
    for sq in read_sqs_from_stdin::<N>() {
        pretty_print_sq(sq);

        for i in 2..N {
            println!("Subsquares order {i}: {}", sq.num_subsquares_dyn(i));
        }

        let main_class = sq.reduced_paratopic();
        if main_class != sq {
            println!("Main class: ");
            pretty_print_sq(main_class);
        } else {
            println!("Is main class reduced");
        }
    }
}

fn generate_latin_squares<const N: usize>() {
    // for permutation in PermutationIter::new() {
    //     if permutation.num_fixed_points() < 1 || permutation.order() > 2 {
    //         continue;
    //     }
    //     dbg!(&permutation);
    //     for sq in RCGenerator::<N>::new(permutation) {
    //         println!("{}", sq.to_string());
    //     }
    // }

    for sq in RCSGenerator::<N>::new() {
        println!("{}", sq.to_string());
    }
}

fn pretty_print<const N: usize>() {
    for sq in read_partial_sqs_from_stdin::<N>() {
        pretty_print_sq(sq);
    }
}

fn pretty_print_sq<const N: usize>(sq: impl Into<PartialLatinSquare<N>>) {
    let sq = sq.into();
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

fn normalize_paratopy<const N: usize>() {
    for sq in read_sqs_from_stdin::<N>() {
        println!("{}", sq.reduced_paratopic().to_string());
    }
}

fn generate_paratopy_classes<const N: usize>() {
    // dbg!(LatinSquareOAGenerator::<N>::new_reduced().count(), return);

    let mut sqs = HashSet::new();

    for sq in LatinSquareOAGenerator::new_reduced() {
        let sq: LatinSquare<N> = sq[0];
        let normalized = sq.reduced_paratopic();

        if !sqs.contains(&normalized) {
            sqs.insert(normalized);

            println!("{}", normalized.to_string());
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

fn find_scs<const N: usize>(start: usize, end: usize) {
    let mut min = N * N;

    for sq in read_sqs_from_stdin::<N>() {
        println!("{}", sq.to_string());

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        for i in start..=end {
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
                    if scs.insert(partial_sq) {
                        println!("{}", partial_sq.to_string());
                    }
                    break;
                }
            }

            if found {
                break;
            }
        }
        println!();
    }

    // println!("min: {min}");
}

fn solve<const N: usize>() {
    for sq in read_partial_sqs_from_stdin::<N>() {
        let solutions = LatinSquareOAGenerator::from_partial(sq).map(|sq| sq[0]);

        for solution in solutions {
            println!("{}", solution.to_string());
        }
    }
}

fn read_sqs_from_file<const N: usize>(path: &Path) -> Vec<LatinSquare<N>> {
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

fn read_sqs_from_stdin<const N: usize>() -> impl Iterator<Item = LatinSquare<N>> {
    let mut line = String::new();

    (0..).map_while(move |_| {
        while stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line = line.trim().into(); // remove newline
            match LatinSquare::try_from(line.as_str()) {
                Ok(sq) => {
                    line.clear();
                    return Some(sq);
                }
                Err(err) => {
                    eprintln!("{err}");
                    line.clear();
                    continue;
                }
            }
        }
        None
    })
}

fn read_partial_sqs_from_stdin<const N: usize>() -> impl Iterator<Item = PartialLatinSquare<N>> {
    let mut line = String::new();

    (0..).map_while(move |_| {
        while stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line = line.trim().into(); // remove newline
            match PartialLatinSquare::try_from(line.as_str()) {
                Ok(sq) => {
                    line.clear();

                    return Some(sq);
                }
                Err(err) => {
                    line.clear();
                    eprintln!("{}", err);
                    continue;
                }
            }
        }
        None
    })
}
