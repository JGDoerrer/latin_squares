use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{stdin, stdout, BufRead, BufReader, Write},
    path::Path,
    vec,
};

use clap::{self, Parser, Subcommand};

use latin_square::LatinSquare;

use latin_square_oa_generator::LatinSquareOAGenerator;

use orthogonal_array::OrthogonalArray;
use partial_latin_square::PartialLatinSquare;

use partial_oa_generator::PartialOAGenerator;
use partial_square_generator::PartialSquareGenerator;
use permutation::{factorial, PermutationIter};
use random_latin_square_generator::RandomLatinSquareGenerator;

use rcs_generator::RCSGenerator;

use crate::hitting_set_generator::HittingSetGenerator;

mod bitset;
mod bitvec;
mod constants;
mod constraints;
mod hitting_set_generator;
mod latin_square;
mod latin_square_generator;
mod latin_square_oa_generator;
mod orthogonal_array;
mod orthogonal_generator;
mod partial_latin_square;
mod partial_oa_generator;
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
        #[arg(short, long, default_value_t = 0)]
        start: usize,
        #[arg(short, long, default_value_t = usize::MAX)]
        end: usize,
    },
    FindMOLSSCS {
        mols: usize,
        #[arg(short, long, default_value_t = 0)]
        start: usize,
        #[arg(short, long, default_value_t = usize::MAX)]
        end: usize,
        #[arg(short, long)]
        all: bool,
    },
    GenerateLatinSquares,
    RandomLatinSquares {
        seed: u64,
    },
    FindOrthogonal {
        #[arg(short, long)]
        all: bool,
    },
    Solve,
    SortByIntercalates,
    NumSubsquares {
        k: usize,
    },
    GenerateMOLS {
        mols: usize,
    },
    ShuffleMOLS {
        mols: usize,
        seed: u64,
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
                Mode::FindMOLSSCS {
                    mols,
                    start,
                    end,
                    all,
                } => find_mols_scs_n::<$N>(mols, start, end, all),
                Mode::RandomLatinSquares { seed } => random_latin_squares::<$N>(seed),
                Mode::FindOrthogonal { all } => find_orthogonal::<$N>(all),
                Mode::Solve => solve::<$N>(),
                Mode::SortByIntercalates => sort_by_intercalates::<$N>(),
                Mode::NumSubsquares { k } => num_subsquares::<$N>(k),
                Mode::GenerateMOLS { mols } => generate_mols_n::<$N>(mols),
                Mode::ShuffleMOLS { mols, seed } => shuffle_mols_n::<$N>(mols, seed),
            }
        };
    }

    match args.n {
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
        if writeln!(stdout(), "{}", sq).is_err() {
            return;
        }
    }
}

fn find_orthogonal<const N: usize>(all: bool) {
    for sq in read_sqs_from_stdin::<N>() {
        println!("{}", sq);
        if all {
            for [_, sq] in LatinSquareOAGenerator::<N, 2>::from_partial_sq_reduced(sq.into())
                .map(|oa| oa.squares())
            {
                println!("{}", sq);
            }
        } else if let Some([_, sq]) =
            LatinSquareOAGenerator::<N, 2>::from_partial_sq_reduced(sq.into())
                .map(|oa| oa.squares())
                .next()
        {
            println!("{}", sq);
        }
        println!()
    }
}

fn random_latin_squares<const N: usize>(seed: u64) {
    for sq in RandomLatinSquareGenerator::<N>::new(seed) {
        println!("{}", sq);
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
        println!("{}", sq);
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
        println!("{}", sq.reduced_paratopic());
    }
}

fn generate_paratopy_classes<const N: usize>() {
    // dbg!(LatinSquareOAGenerator::<N>::new_reduced().count(), return);

    let mut sqs = HashSet::new();

    for sq in LatinSquareOAGenerator::<N, 1>::new_reduced() {
        let sq: LatinSquare<N> = sq.squares()[0];
        let normalized = sq.reduced_paratopic();

        if !sqs.contains(&normalized) {
            sqs.insert(normalized);

            println!("{}", normalized);
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
        println!("{}", sq);

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        let end = end.min(N * N);

        if start <= end {
            for i in start..=end {
                dbg!(i);
                let hitting_sets = HittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    let partial_sq = sq.mask(hitting_set);

                    for partial_sq in PartialSquareGenerator::new_partial(sq, partial_sq, i) {
                        // dbg!(partial_sq);
                        let mut solutions =
                            LatinSquareOAGenerator::<N, 1>::from_partial_sq(partial_sq);
                        let first_solution = solutions.next();
                        let second_solution = solutions.next();

                        if second_solution.is_none()
                            && first_solution.is_some_and(|solution| solution.squares()[0] == sq)
                        {
                            // println!("{}", partial_sq.to_string());

                            min = min.min(partial_sq.num_entries());
                            found = true;
                            if scs.insert(partial_sq) {
                                println!("{}", partial_sq);
                            }
                            break 'h;
                        }
                    }
                }

                if found {
                    break;
                }
            }
        } else {
            for i in (end..=start).rev() {
                dbg!(i);
                let hitting_sets = HittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    let partial_sq = sq.mask(hitting_set);

                    for partial_sq in PartialSquareGenerator::new_partial(sq, partial_sq, i) {
                        // dbg!(partial_sq);
                        let mut solutions =
                            LatinSquareOAGenerator::<N, 1>::from_partial_sq(partial_sq);
                        let first_solution = solutions.next();
                        let second_solution = solutions.next();

                        if second_solution.is_none()
                            && first_solution.is_some_and(|solution| solution.squares()[0] == sq)
                        {
                            // println!("{}", partial_sq.to_string());

                            min = min.min(partial_sq.num_entries());
                            found = true;
                            if scs.insert(partial_sq) {
                                println!("{}", partial_sq);
                            }
                            break 'h;
                        }
                    }
                }

                if !found {
                    break;
                }
            }
        }
        println!();
    }

    // println!("min: {min}");
}

fn generate_mols_n<const N: usize>(mols: usize) {
    assert!(mols > 0);
    assert!(mols < N);

    macro_rules! match_mols {
        ($( $i : literal),+) => {
            match mols {
                $(
                    $i => generate_mols::<N, $i>(),
                )*
                _ => unreachable!(),
            }
        };
    }

    match N {
        3 => match_mols!(1, 2),
        4 => match_mols!(1, 2, 3),
        5 => match_mols!(1, 2, 3, 4),
        6 => match_mols!(1, 2, 3, 4, 5),
        7 => match_mols!(1, 2, 3, 4, 5, 6),
        8 => match_mols!(1, 2, 3, 4, 5, 6, 7),
        _ => todo!(),
    }
}

fn generate_mols<const N: usize, const MOLS: usize>() {
    for mols in LatinSquareOAGenerator::<N, MOLS>::new_reduced() {
        for (i, sq) in mols.squares().into_iter().enumerate() {
            print!("{sq}");
            if i != MOLS - 1 {
                print!("-");
            }
        }
        println!()
    }
}

fn find_mols_scs_n<const N: usize>(mols: usize, start: usize, end: usize, all: bool) {
    assert!(mols > 0);
    assert!(mols < N);

    macro_rules! match_mols {
        ($( $i : literal),+) => {
            match mols {
                $(
                    $i => find_mols_scs::<N, $i>(start,end, all),
                )*
                _ => unreachable!(),
            }
        };
    }

    match N {
        3 => match_mols!(1, 2),
        4 => match_mols!(1, 2, 3),
        5 => match_mols!(1, 2, 3, 4),
        6 => match_mols!(1, 2, 3, 4, 5),
        7 => match_mols!(1, 2, 3, 4, 5, 6),
        8 => match_mols!(1, 2, 3, 4, 5, 6, 7),
        _ => todo!(),
    }
}

fn find_mols_scs<const N: usize, const MOLS: usize>(start: usize, end: usize, all: bool) {
    for oa in read_mols_from_stdin::<N, MOLS>() {
        dbg!(&oa);

        let unavoidable_sets = oa.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        let end = end.min(MOLS * N * N);

        if start <= end {
            for i in start..=end {
                dbg!(i);
                let hitting_sets = HittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    let partial_sq = oa.mask(hitting_set);

                    for partial_oa in PartialOAGenerator::new_partial(oa.clone(), partial_sq, i) {
                        let mut solutions =
                            LatinSquareOAGenerator::<N, MOLS>::from_partial_oa(&partial_oa);
                        let first_solution = solutions.next();
                        let second_solution = solutions.next();

                        if second_solution.is_none()
                            && first_solution.is_some_and(|solution| solution == oa)
                        {
                            found = true;
                            if scs.insert(partial_oa.clone()) {
                                println!("{}", partial_oa);
                            }
                            if !all {
                                break 'h;
                            }
                        }
                    }
                }

                if found {
                    break;
                }
            }
        } else {
            todo!()
        }
    }
}

fn solve<const N: usize>() {
    for sq in read_partial_sqs_from_stdin::<N>() {
        let solutions =
            LatinSquareOAGenerator::<N, 1>::from_partial_sq(sq).map(|sq| sq.squares()[0]);

        for solution in solutions {
            println!("{}", solution);
        }
    }
}

fn shuffle_mols_n<const N: usize>(mols: usize, seed: u64) {
    assert!(mols > 0);
    assert!(mols < N);

    macro_rules! match_mols {
        ($( $i : literal),+) => {
            match mols {
                $(
                    $i => shuffle_mols::<N, $i>(seed),
                )*
                _ => unreachable!(),
            }
        };
    }

    match N {
        3 => match_mols!(1, 2),
        4 => match_mols!(1, 2, 3),
        5 => match_mols!(1, 2, 3, 4),
        6 => match_mols!(1, 2, 3, 4, 5),
        7 => match_mols!(1, 2, 3, 4, 5, 6),
        8 => match_mols!(1, 2, 3, 4, 5, 6, 7),
        _ => todo!(),
    }
}

fn shuffle_mols<const N: usize, const MOLS: usize>(seed: u64) {
    let mut state = [seed, 1, 2, 3];

    fn xoshiro(state: &mut [u64; 4]) -> u64 {
        let result = state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);

        let new_state = [
            state[0] ^ state[1] ^ state[3],
            state[0] ^ state[1] ^ state[2],
            state[2] ^ state[0] ^ (state[1] << 17),
            (state[3] ^ state[1]).rotate_left(45),
        ];

        *state = new_state;

        result
    }

    for mut mols in read_mols_from_stdin::<N, MOLS>() {
        let row_perm = xoshiro(&mut state) as usize % factorial(N);
        let col_perm = xoshiro(&mut state) as usize % factorial(N);
        let val_perms = [0; MOLS].map(|_| xoshiro(&mut state) as usize % factorial(N));

        mols = mols.permute_rows(&PermutationIter::new().nth(row_perm).unwrap());

        println!("{}", mols);
    }

    todo!()
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

fn read_mols_from_stdin<const N: usize, const MOLS: usize>(
) -> impl Iterator<Item = OrthogonalArray<N, MOLS>> {
    let mut line = String::new();

    (0..).map_while(move |_| {
        while stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line = line.trim().into(); // remove newline
            match OrthogonalArray::try_from(line.as_str()) {
                Ok(oa) => {
                    line.clear();

                    return Some(oa);
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
