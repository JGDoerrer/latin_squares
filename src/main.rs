#![feature(portable_simd)]

use std::{
    collections::{HashMap, HashSet},
    io::{stdin, stdout, Write},
    thread::{self, available_parallelism},
    vec,
};

use clap::{self, Parser, Subcommand};

use isotopy_class_generator::IsotopyClassGenerator;
use latin_square::{generate_minimize_rows_lookup, LatinSquare};

use latin_square_dyn::LatinSquareDyn;
use latin_square_generator::LatinSquareGeneratorDyn;
use oa_generator::OAGenerator;

use latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait};
use mmcs_hitting_set_generator::MMCSHittingSetGenerator;
use orthogonal_array::{OrthogonalArray, SEPARATOR};

use partial_latin_square_dyn::PartialLatinSquareDyn;
use partial_oa_generator::PartialOAGenerator;
use partial_orthogonal_array::PartialOrthogonalArray;
use partial_square_generator::PartialSquareGeneratorDyn;
use random_latin_square_generator::RandomLatinSquareGenerator;

mod bitset;
mod bitvec;
mod constants;
mod constraints;
mod hitting_set_generator;
mod isotopy_class_generator;
mod latin_square;
mod latin_square_dyn;
mod latin_square_generator;
mod latin_square_trait;
mod mmcs_hitting_set_generator;
mod oa_constraints;
mod oa_generator;
mod orthogonal_array;
mod partial_latin_square;
mod partial_latin_square_dyn;
mod partial_oa_generator;
mod partial_orthogonal_array;
mod partial_square_generator;
mod permutation;
mod random_latin_square_generator;
mod rc_generator;
mod rcs_generator;
mod row_partial_latin_square;
mod tuple_iterator;

#[derive(Subcommand, Clone)]
enum Mode {
    PrettyPrint,
    /// Prints all solutions for a partial latin square
    Solve,
    Analyse {
        n: usize,
    },
    NormalizeMainClass {
        n: usize,
    },
    GenerateLatinSquares {
        n: usize,
    },
    GenerateIsotopyClasses {
        n: usize,
    },
    GenerateMainClasses {
        n: usize,
    },
    FindSCS {
        #[arg(short, long)]
        reverse: bool,
    },
    FindLCS,
    FindMOLSSCS {
        mols: usize,
        #[arg(short, long, default_value_t = 0)]
        start: usize,
        #[arg(short, long, default_value_t = usize::MAX)]
        end: usize,
        #[arg(short, long)]
        all: bool,
    },
    Random {
        n: usize,
        seed: u64,
    },
    FindOrthogonal {
        n: usize,
        #[arg(short, long)]
        all: bool,
    },
    NumSubsquares {
        k: usize,
    },
    FindMOLS {
        n: usize,
        mols: usize,
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
        Mode::GenerateLatinSquares { n } => generate_latin_squares(n),
        Mode::GenerateIsotopyClasses { n } => match_n!(n, generate_isotopy_classes),
        Mode::GenerateMainClasses { n } => match_n!(n, generate_main_classes),
        Mode::FindSCS { reverse } => find_scs(reverse),
        Mode::Solve => solve(),
        Mode::NumSubsquares { k } => num_subsquares(k),
        Mode::FindLCS => find_lcs(),
        Mode::Random { n, seed } => match_n!(n, random_latin_squares, seed),
        Mode::FindOrthogonal { n, all } => match_n!(n, find_orthogonal, all),
        Mode::FindMOLS { n, mols } => match_n!(n, find_mols, mols),
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
        println!("{sq}");

        if all {
            for sq in sq.orthogonal_squares() {
                println!("{sq}");
            }
        } else if let Some(sq) = sq.orthogonal_squares().next() {
            println!("{sq}");
        }

        println!()
    }
}

fn random_latin_squares<const N: usize>(seed: u64) {
    for sq in RandomLatinSquareGenerator::<N>::new(seed) {
        if writeln!(stdout(), "{}", sq).is_err() {
            return;
        }
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

        println!("Transversals: {}", sq.num_transversals());
        println!(
            "Max disjoint transversals: {}",
            sq.max_disjoint_transversals()
        );
        println!(
            "Full disjoint transversal count: {}",
            sq.full_disjoint_transversals().count()
        );
        println!();

        println!("Cycles:");
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

        let (isotopy_class, _) = sq.isotopy_class_permutation();
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

fn generate_latin_squares(n: usize) {
    for sq in LatinSquareGeneratorDyn::new(n) {
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

fn generate_isotopy_classes<const N: usize>() {
    let lookup = generate_minimize_rows_lookup();
    for (i, sq) in IsotopyClassGenerator::<N>::new(&lookup).enumerate() {
        dbg!(i + 1);

        if writeln!(stdout(), "{sq}").is_err() {
            return;
        }
    }
}

fn generate_main_classes<const N: usize>() {
    let lookup = generate_minimize_rows_lookup();
    for (i, sq) in IsotopyClassGenerator::<N>::new(&lookup)
        .filter(|sq| *sq == sq.main_class_lookup(&lookup))
        .enumerate()
    // for (i, sq) in MainClassGenerator::<N>::new(&lookup).enumerate()
    {
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

        // for set in &unavoidable_sets[0] {
        //     set.print_sq(sq.n());
        // }
        // println!();

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
            let mut hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), end);
            for i in (start..=end).rev() {
                dbg!(i);

                let mut found = false;
                let mut scs = HashSet::new();
                'h: while let Some(hitting_set) = hitting_sets.next() {
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
                hitting_sets.decrease_max_entries();

                if !found {
                    break;
                }
            }
        }
        println!();
    }

    // println!("min: {min}");
}

fn find_lcs() {
    let mut threads = Vec::new();

    while let Some(sq) = read_sq_from_stdin() {
        let thread = thread::spawn(move || find_lcs_sq(sq));

        threads.push(thread);

        while threads.len()
            >= available_parallelism()
                .unwrap_or(1.try_into().unwrap())
                .into()
        {
            for i in 0..threads.len() {
                if !threads[i].is_finished() {
                    continue;
                }

                let thread = threads.swap_remove(i);
                thread.join().unwrap();
            }
        }
    }
}

fn find_lcs_sq(sq: LatinSquareDyn) {
    let unavoidable_sets = sq.unavoidable_sets();
    unavoidable_sets.iter().for_each(|sets| {
        dbg!(sets.len());
    });

    let hitting_sets = MMCSHittingSetGenerator::new(unavoidable_sets.clone(), sq.n() * sq.n());

    let mut lcs = PartialLatinSquareDyn::empty(sq.n());
    'h: for hitting_set in hitting_sets {
        let partial_sq = sq.mask(hitting_set);

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
                        continue 'h;
                    }
                }
            }

            // println!("{}", partial_sq.to_string());

            if lcs.num_entries() < partial_sq.num_entries() {
                lcs = partial_sq;
            }
        }
    }

    let mut stdout = stdout().lock();

    writeln!(stdout, "{}", sq).unwrap();
    writeln!(stdout, "{lcs}").unwrap();
    writeln!(stdout,).unwrap();
}

fn find_mols<const N: usize>(mols: usize) {
    for sq in read_sqs_from_stdin_n::<N>() {
        let has_orthogonal = sq
            .full_disjoint_transversals()
            .nth(mols.saturating_sub(2))
            .is_some();

        if !has_orthogonal {
            continue;
        }

        let mut current_mols = vec![sq];
        let mut indices = vec![0];
        let orthogonal: Vec<_> = sq.orthogonal_squares().collect();

        'i: while let Some(index) = indices.last_mut() {
            for orthogonal in orthogonal.iter().skip(*index) {
                *index += 1;
                if current_mols
                    .iter()
                    .all(|sq| sq.is_orthogonal_to(orthogonal))
                {
                    current_mols.push(*orthogonal);

                    if current_mols.len() == mols {
                        println!(
                            "{}",
                            current_mols
                                .iter()
                                .map(|sq| sq.to_string())
                                .reduce(|a, b| format!("{a}{}{b}", SEPARATOR))
                                .unwrap()
                        );

                        current_mols.pop();
                    } else {
                        let next_index = *index;
                        indices.push(next_index);
                    }

                    continue 'i;
                }
            }

            current_mols.pop();
            indices.pop();
        }
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

fn read_sq_from_stdin() -> Option<LatinSquareDyn> {
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
}

fn read_sqs_from_stdin() -> impl Iterator<Item = LatinSquareDyn> {
    (0..).map_while(|_| read_sq_from_stdin())
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
