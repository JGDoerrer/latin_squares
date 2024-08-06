use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{stdin, stdout, BufRead, BufReader, Write},
    path::Path,
    vec,
};

use clap::{self, Parser, Subcommand};

use latin_square::{generate_minimize_rows_lookup, LatinSquare};

use latin_square_dyn::LatinSquareDyn;
use latin_square_generator::{LatinSquareGenerator, LatinSquareGeneratorDyn};
use oa_generator::OAGenerator;

use latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait};
use main_class_generator::MainClassGenerator;
use mmcs_hitting_set_generator::MMCSHittingSetGenerator;
use orthogonal_array::OrthogonalArray;
use partial_latin_square::PartialLatinSquare;

use partial_latin_square_dyn::PartialLatinSquareDyn;
use partial_oa_generator::PartialOAGenerator;
use partial_orthogonal_array::PartialOrthogonalArray;
use partial_square_generator::PartialSquareGeneratorDyn;
use permutation::{factorial, PermutationIter};
use random_latin_square_generator::RandomLatinSquareGenerator;

mod bitset;
mod bitvec;
mod constants;
mod constraints;
mod hitting_set_generator;
mod latin_square;
mod latin_square_dyn;
mod latin_square_generator;
mod latin_square_trait;
mod main_class_generator;
mod mmcs_hitting_set_generator;
mod oa_constraints;
mod oa_generator;
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
    Analyse {
        n: usize,
    },
    PrettyPrint,
    NormalizeMainClass {
        n: usize,
    },
    GenerateMainClasses {
        n: usize,
    },
    FindSCS {
        #[arg(short, long)]
        reverse: bool,
    },
    FindLCS {
        #[arg(short, long)]
        reverse: bool,
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
    GenerateLatinSquares {
        n: usize,
    },
    Random {
        n: usize,
        seed: u64,
    },
    FindOrthogonal {
        #[arg(short, long)]
        all: bool,
    },
    Solve,
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
    #[command(subcommand)]
    mode: Mode,
}

fn main() {
    let args = Args::parse();

    macro_rules! match_n {
        ($n: expr, $f: ident $(, $args: expr)*) => {
            match $n {
                1 => $f::<1>($($args),*),
                2 => $f::<2>($($args),*),
                3 => $f::<3>($($args),*),
                4 => $f::<4>($($args),*),
                5 => $f::<5>($($args),*),
                6 => $f::<6>($($args),*),
                7 => $f::<7>($($args),*),
                8 => $f::<8>($($args),*),
                9 => $f::<9>($($args),*),
                10 => $f::<10>($($args),*),
                _ => todo!(),
            }
        };
    }

    match args.mode {
        Mode::Analyse { n } => match_n!(n, analyse),
        Mode::PrettyPrint => pretty_print(),
        Mode::NormalizeMainClass { n } => match_n!(n, normalize_main_class),
        Mode::GenerateMainClasses { n } => match_n!(n, generate_main_classes),
        Mode::FindSCS { reverse } => find_scs(reverse),
        Mode::GenerateLatinSquares { n } => match_n!(n, generate_latin_squares),
        Mode::Solve => solve(),
        Mode::NumSubsquares { k } => num_subsquares(k),
        Mode::FindLCS { reverse } => find_lcs(reverse),
        Mode::Random { n, seed } => match_n!(n, random_latin_squares, seed),
        _ => todo!(),
    }
}

fn num_subsquares(k: usize) {
    for sq in read_sqs_from_stdin() {
        println!("{}", sq.num_subsquares_dyn(k));
    }
}

fn find_orthogonal<const N: usize>(all: bool) {
    for sq in read_sqs_from_stdin_n::<N>() {
        println!("{}", sq);
        if all {
            for [_, sq] in
                OAGenerator::<N, 2>::from_partial_sq_reduced(sq.into()).map(|oa| oa.squares())
            {
                println!("{}", sq);
            }
        } else if let Some([_, sq]) = OAGenerator::<N, 2>::from_partial_sq_reduced(sq.into())
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

        let isotopy_class = sq.isotopy_class();
        if isotopy_class != sq {
            println!("Isotopy class: ");
            println!("{}", isotopy_class);
            pretty_print_sq(isotopy_class);
        } else {
            println!("Is isotopy class reduced");
        }

        let main_class = sq.main_class();
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

fn pretty_print() {
    for sq in read_partial_sqs_from_stdin() {
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
        println!("{}", sq.main_class());
    }
}

fn generate_main_classes<const N: usize>() {
    let lookup = generate_minimize_rows_lookup();
    for (i, sq) in MainClassGenerator::<N>::new(&lookup).enumerate() {
        dbg!(i + 1);

        if writeln!(stdout(), "{sq}").is_err() {
            return;
        }
    }
}

const KNOWN_SCS: [usize; 9] = [0, 0, 1, 2, 4, 6, 9, 12, 16];

fn find_scs(reverse: bool) {
    for sq in read_sqs_from_stdin() {
        println!("{}", sq);

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        let start = *KNOWN_SCS.get(sq.n()).unwrap_or(&sq.n());
        let end = sq.n().pow(2);

        if !reverse {
            for i in start..=end {
                dbg!(i);
                let hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
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
            for i in (start..=end).rev() {
                dbg!(i);
                let hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), i);

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

fn find_lcs(reverse: bool) {
    for sq in read_sqs_from_stdin() {
        println!("{}", sq);

        let unavoidable_sets = sq.unavoidable_sets();
        unavoidable_sets.iter().for_each(|sets| {
            dbg!(sets.len());
        });

        let start = *KNOWN_SCS.get(sq.n()).unwrap_or(&sq.n());
        let end = sq.n().pow(2);

        if reverse {
            for i in (start..=end).rev() {
                dbg!(i);
                let hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut lcs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    let partial_sq = sq.mask(hitting_set);

                    'sq: for partial_sq in
                        PartialSquareGeneratorDyn::new_partial(sq.clone(), partial_sq, i)
                    {
                        // dbg!(partial_sq.to_string());
                        let mut solutions = LatinSquareGeneratorDyn::from_partial_sq(&partial_sq);
                        let first_solution = solutions.next();
                        let second_solution = solutions.next();

                        if second_solution.is_some()
                            || first_solution.is_none()
                            || first_solution.is_some_and(|solution| solution != sq)
                        {
                            continue;
                        }

                        // check if removing one entry results in >= 2 solutions
                        for i in 0..sq.n() {
                            for j in 0..sq.n() {
                                if partial_sq.get_partial(i, j).is_none() {
                                    continue;
                                }
                                let mut sq = partial_sq.clone();
                                sq.set(i, j, None);

                                let mut solutions = LatinSquareGeneratorDyn::from_partial_sq(&sq);
                                solutions.next();
                                let second_solution = solutions.next();
                                if second_solution.is_none() {
                                    continue 'sq;
                                }
                            }
                        }

                        // println!("{}", partial_sq.to_string());

                        found = true;
                        if lcs.insert(partial_sq.clone()) {
                            println!("{}", partial_sq);
                        }
                        break 'h;
                    }
                }

                if found {
                    break;
                }
            }
        } else {
            let mut found_first = false;
            let mut lcs = PartialLatinSquareDyn::empty(sq.n());

            for i in start..=end {
                dbg!(i);
                let hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                'h: for hitting_set in hitting_sets {
                    if hitting_set.len() > i {
                        continue;
                    }

                    let partial_sq = sq.mask(hitting_set);

                    'sq: for partial_sq in
                        PartialSquareGeneratorDyn::new_partial(sq.clone(), partial_sq.clone(), i)
                    {
                        // dbg!(partial_sq);
                        let mut solutions = LatinSquareGeneratorDyn::from_partial_sq(&partial_sq);
                        let first_solution = solutions.next();
                        let second_solution = solutions.next();

                        if second_solution.is_some()
                            || first_solution.is_none()
                            || first_solution.is_some_and(|solution| solution != sq)
                        {
                            continue;
                        }

                        // check if removing one entry results in >= 2 solutions
                        for i in 0..sq.n() {
                            for j in 0..sq.n() {
                                if partial_sq.get_partial(i, j).is_none() {
                                    continue;
                                }
                                let mut sq = partial_sq.clone();
                                sq.set(i, j, None);

                                let mut solutions = LatinSquareGeneratorDyn::from_partial_sq(&sq);
                                solutions.next();
                                let second_solution = solutions.next();
                                if second_solution.is_none() {
                                    continue 'sq;
                                }
                            }
                        }

                        // println!("{}", partial_sq.to_string());
                        found_first = true;
                        found = true;
                        lcs = partial_sq;
                        break 'h;
                    }
                }

                if found_first && !found {
                    println!("{lcs}");
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
    for mols in OAGenerator::<N, MOLS>::new_reduced() {
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
                let hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    let partial_sq = oa.mask(hitting_set);

                    for partial_oa in PartialOAGenerator::new_partial(oa.clone(), partial_sq, i) {
                        let mut solutions = OAGenerator::<N, MOLS>::from_partial_oa(&partial_oa);
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
                let hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: for hitting_set in hitting_sets {
                    let partial_sq = oa.mask(hitting_set);

                    for partial_oa in PartialOAGenerator::new_partial(oa.clone(), partial_sq, i) {
                        let mut solutions = OAGenerator::<N, MOLS>::from_partial_oa(&partial_oa);
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
        let solutions = OAGenerator::<N, MOLS>::from_partial_oa(&oa);

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
