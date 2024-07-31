use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{stdin, stdout, BufRead, BufReader, Write},
    path::Path,
    vec,
};

use clap::{self, Parser, Subcommand};

use latin_square::LatinSquare;

use latin_square_dyn::LatinSquareDyn;
use latin_square_generator::{LatinSquareGenerator, LatinSquareGeneratorDyn};
use latin_square_oa_generator::LatinSquareOAGenerator;

use latin_square_trait::PartialLatinSquareTrait;
use main_class_generator::MainClassGenerator;
use new_hitting_set_generator::NewHittingSetGenerator;
use orthogonal_array::OrthogonalArray;
use partial_latin_square::PartialLatinSquare;

use partial_latin_square_dyn::PartialLatinSquareDyn;
use partial_oa_generator::PartialOAGenerator;
use partial_orthogonal_array::PartialOrthogonalArray;
use partial_square_generator::PartialSquareGeneratorDyn;
use permutation::{factorial, PermutationIter};
use random_latin_square_generator::RandomLatinSquareGenerator;

use crate::hitting_set_generator::HittingSetGenerator;

mod bitset;
mod bitvec;
mod constants;
mod constraints;
mod hitting_set_generator;
mod latin_square;
mod latin_square_dyn;
mod latin_square_generator;
mod latin_square_oa_generator;
mod latin_square_trait;
mod main_class_generator;
mod new_hitting_set_generator;
mod oa_constraints;
mod orthogonal_array;
mod orthogonal_generator;
mod partial_latin_square;
mod partial_latin_square_dyn;
mod partial_oa_generator;
mod partial_orthogonal_array;
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
    NormalizeMainClass,
    GenerateMainClasses,
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
    SolveMOLS {
        mols: usize,
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
                Mode::NormalizeMainClass => normalize_main_class::<$N>(),
                Mode::GenerateMainClasses => generate_main_classes::<$N>(),
                Mode::FindSCS { start, end } => find_scs(start, end),
                Mode::FindMOLSSCS {
                    mols,
                    start,
                    end,
                    all,
                } => find_mols_scs_n::<$N>(mols, start, end, all),
                // Mode::RandomLatinSquares { seed } => random_latin_squares::<$N>(seed),
                Mode::FindOrthogonal { all } => find_orthogonal::<$N>(all),
                Mode::Solve => solve(),
                // Mode::SortByIntercalates => sort_by_intercalates::<$N>(),
                // Mode::NumSubsquares { k } => num_subsquares::<$N>(k),
                Mode::GenerateMOLS { mols } => generate_mols_n::<$N>(mols),
                // Mode::ShuffleMOLS { mols, seed } => shuffle_mols_n::<$N>(mols, seed),
                // Mode::SolveMOLS { mols } => solve_mols_n::<$N>(mols),
                _ => todo!(),
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
    for sq in read_sqs_from_stdin_n::<N>() {
        println!("{}", sq.num_subsquares_dyn(k));
    }
}

fn sort_by_intercalates<const N: usize>() {
    let mut sqs: Vec<_> = read_sqs_from_stdin_n::<N>().collect();
    sqs.sort_by_key(|a| a.intercalates());
    for sq in sqs {
        if writeln!(stdout(), "{}", sq).is_err() {
            return;
        }
    }
}

fn find_orthogonal<const N: usize>(all: bool) {
    for sq in read_sqs_from_stdin_n::<N>() {
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
    for sq in read_sqs_from_stdin_n::<N>() {
        pretty_print_sq(sq);

        for i in 2..N {
            println!("Subsquares order {i}: {}", sq.num_subsquares_dyn(i));
        }
        println!();

        println!("Symmetries: ");
        let symmetries = sq.symmetries();
        for symmetry in symmetries {
            let rcv: String = symmetry.apply_array(['R', 'C', 'V']).into_iter().collect();
            println!("{rcv}");
        }
        println!();

        for cycles in [sq.row_cycles(), sq.col_cycles(), sq.val_cycles()] {
            let mut counts: Vec<_> = {
                let mut map = HashMap::new();

                for cycle in cycles {
                    if let Some(count) = map.get_mut(&cycle) {
                        *count += 1;
                    } else {
                        map.insert(cycle, 1usize);
                    }
                }

                map.into_iter().collect()
            };
            counts.sort();

            for (cycle, count) in counts {
                println!("{cycle:?}: {count}");
            }
            println!();
        }

        let isotopy_class = sq.reduced_isotopic();
        if isotopy_class != sq {
            println!("Isotopy class: ");
            println!("{}", isotopy_class);
            pretty_print_sq(isotopy_class);
        } else {
            println!("Is isotopy class reduced");
        }

        let main_class = sq.main_class_reduced();
        if main_class != sq {
            println!("Main class: ");
            println!("{}", main_class);
            pretty_print_sq(main_class);
        } else {
            println!("Is main class reduced");
        }
    }
}

fn generate_latin_squares<const N: usize>() {
    for sq in LatinSquareGenerator::<N>::new() {
        println!("{sq}");
    }
}

fn pretty_print<const N: usize>() {
    for sq in read_partial_sqs_from_stdin_n::<N>() {
        pretty_print_sq(sq);
    }
}

fn pretty_print_sq(sq: impl PartialLatinSquareTrait) {
    let n = sq.n();

    for i in 0..n {
        println!("+{}", "---+".repeat(n));
        print!("|");
        for j in 0..n {
            if let Some(value) = sq.get_partial(i, j) {
                print!(" {} |", value);
            } else {
                print!("   |");
            }
        }
        println!()
    }
    println!("+{}", "---+".repeat(n));
    println!()
}

fn normalize_main_class<const N: usize>() {
    for sq in read_sqs_from_stdin_n::<N>() {
        println!("{}", sq.main_class_reduced());
    }
}

fn generate_main_classes<const N: usize>() {
    for (i, sq) in MainClassGenerator::<N>::new().enumerate() {
        dbg!(i + 1);

        if writeln!(stdout(), "{sq}").is_err() {
            return;
        }
    }
}

fn find_scs(start: usize, end: usize) {
    for sq in read_sqs_from_stdin() {
        println!("{}", sq);

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        let end = end.min(sq.n() * sq.n());

        if start <= end {
            for i in start..=end {
                dbg!(i);
                let hitting_sets = NewHittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    if hitting_set.len() > i {
                        continue;
                    }

                    let partial_sq = sq.mask(hitting_set);

                    for partial_sq in
                        PartialSquareGeneratorDyn::new_partial(sq.clone(), partial_sq.clone(), i)
                    {
                        // dbg!(partial_sq);
                        let mut solutions = LatinSquareGeneratorDyn::from_partial_sq(&partial_sq);
                        let first_solution = solutions.next();
                        let second_solution = solutions.next();

                        if second_solution.is_none()
                            && first_solution.is_some_and(|solution| solution == sq)
                        {
                            // println!("{}", partial_sq.to_string());

                            found = true;
                            if scs.insert(partial_sq.clone()) {
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
                let hitting_sets = NewHittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    let partial_sq = sq.mask(hitting_set);

                    for partial_sq in
                        PartialSquareGeneratorDyn::new_partial(sq.clone(), partial_sq, i)
                    {
                        // dbg!(partial_sq);
                        let mut solutions = LatinSquareGeneratorDyn::from_partial_sq(&partial_sq);
                        let first_solution = solutions.next();
                        let second_solution = solutions.next();

                        if second_solution.is_none()
                            && first_solution.is_some_and(|solution| solution == sq)
                        {
                            // println!("{}", partial_sq.to_string());

                            found = true;
                            if scs.insert(partial_sq.clone()) {
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
        10 => match_mols!(2, 3),
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
        3 => match_mols!(2),
        4 => match_mols!(2, 3),
        5 => match_mols!(2, 3, 4),
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
                let hitting_sets = NewHittingSetGenerator::new(unavoidable_sets.clone(), i);

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
            for i in (end..=start).rev() {
                dbg!(i);
                let hitting_sets = NewHittingSetGenerator::new(unavoidable_sets.clone(), i);

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

                if !found {
                    break;
                }
            }
        }
    }
}

fn solve() {
    for sq in read_partial_sqs_from_stdin() {
        let solutions = LatinSquareGeneratorDyn::from_partial_sq(&sq);

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

    todo!()
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

fn solve_mols_n<const N: usize>(mols: usize) {
    assert!(mols > 0);
    assert!(mols < N);

    macro_rules! match_mols {
        ($( $i : literal),+) => {
            match mols {
                $(
                    $i => solve_mols::<N, $i>(),
                )*
                _ => unreachable!(),
            }
        };
    }

    match N {
        3 => match_mols!(2),
        4 => match_mols!(2, 3),
        5 => match_mols!(2, 3, 4),
        10 => match_mols!(2, 3),
        _ => todo!(),
    }
}

fn solve_mols<const N: usize, const MOLS: usize>() {
    for oa in read_partial_mols_from_stdin() {
        let solutions = LatinSquareOAGenerator::<N, MOLS>::from_partial_oa(&oa);

        for solution in solutions {
            if writeln!(stdout(), "{}", solution).is_err() {
                return;
            }
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

fn read_sqs_from_stdin_n<const N: usize>() -> impl Iterator<Item = LatinSquare<N>> {
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

fn read_sqs_from_stdin() -> impl Iterator<Item = LatinSquareDyn> {
    (0..).map_while(|_| {
        let mut line = String::new();
        while stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line = line.trim().into(); // remove newline
            match LatinSquareDyn::try_from(line.as_str()) {
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

fn read_partial_sqs_from_stdin_n<const N: usize>() -> impl Iterator<Item = PartialLatinSquare<N>> {
    (0..).map_while(|_| {
        let mut line = String::new();
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

fn read_partial_sqs_from_stdin() -> impl Iterator<Item = PartialLatinSquareDyn> {
    (0..).map_while(|_| {
        let mut line = String::new();
        while stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line = line.trim().into(); // remove newline
            match PartialLatinSquareDyn::try_from(line.as_str()) {
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
    (0..).map_while(|_| {
        let mut line: String = String::new();
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

fn read_partial_mols_from_stdin<const N: usize, const MOLS: usize>(
) -> impl Iterator<Item = PartialOrthogonalArray<N, MOLS>> {
    (0..).map_while(|_| {
        let mut line = String::new();
        while stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
            line = line.trim().into(); // remove newline
            match PartialOrthogonalArray::try_from(line.as_str()) {
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
