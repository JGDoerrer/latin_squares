#![feature(portable_simd)]

use std::{
    collections::{HashMap, HashSet},
    io::{stdin, stdout, Read, Write},
    thread::{self},
    time::Duration,
    vec,
};

use bitset::BitSet16;
use clap::{self, Parser, Subcommand};

use cycles::{generate_minimize_rows_lookup, generate_minimize_rows_lookup_simd};
use isotopy_class_generator::IsotopyClassGenerator;
use latin_square::LatinSquare;

use latin_square_dyn::LatinSquareDyn;
use latin_square_generator::LatinSquareGeneratorDyn;
use mols::MOLS;

use latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait};
use mmcs_hitting_set_generator::MMCSHittingSetGenerator;

use partial_latin_square_dyn::PartialLatinSquareDyn;
use partial_square_generator::PartialSquareGeneratorDyn;
use permutation::factorial;
use permutation_dyn::PermutationDyn;
use random_latin_square_generator::RandomLatinSquareGenerator;
use threaded_main_class_generator::ThreadedMainClassGenerator;

mod bitset;
mod bitvec;
mod constants;
mod constraints;
mod cycles;
mod hitting_set_generator;
mod isotopy_class_generator;
mod latin_square;
mod latin_square_dyn;
mod latin_square_generator;
mod latin_square_trait;
mod mmcs_hitting_set_generator;
mod mols;
mod oa_constraints;
mod oa_generator;
mod orthogonal_array;
mod partial_latin_square;
mod partial_latin_square_dyn;
mod partial_oa_generator;
mod partial_orthogonal_array;
mod partial_square_generator;
mod permutation;
mod permutation_dyn;
mod permutation_simd;
mod random_latin_square_generator;
mod row_partial_latin_square;
mod threaded_main_class_generator;
mod tuple_iterator;

#[derive(Subcommand, Clone)]
enum Mode {
    PrettyPrint,
    /// Prints all solutions for a partial latin square
    Solve,
    Shuffle,
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
        #[arg(long)]
        max_threads: usize,
    },
    FindAllCS,
    FindSCS {
        #[arg(short, long)]
        reverse: bool,
    },
    FindLCS {
        #[arg(long)]
        max_threads: usize,
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
    ToTex,
    Encode {
        n: usize,
    },
    Decode {
        n: usize,
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
                _ => unimplemented!(),
            }
        };
    }

    match args.mode {
        Mode::Analyse { n } => match_n!(n, analyse),
        Mode::PrettyPrint => pretty_print(),
        Mode::NormalizeMainClass { n } => match_n!(n, normalize_main_class),
        Mode::GenerateLatinSquares { n } => generate_latin_squares(n),
        Mode::GenerateIsotopyClasses { n } => match_n!(n, generate_isotopy_classes),
        Mode::GenerateMainClasses { n, max_threads } => {
            match_n!(n, generate_main_classes, max_threads)
        }
        Mode::Solve => solve(),
        Mode::Shuffle => shuffle(),
        Mode::NumSubsquares { k } => num_subsquares(k),
        Mode::FindAllCS => find_all_cs(),
        Mode::FindLCS { max_threads } => find_lcs(max_threads),
        Mode::FindSCS { reverse } => find_scs(reverse),
        Mode::Random { n, seed } => match_n!(n, random_latin_squares, seed),
        Mode::FindOrthogonal { n, all } => match_n!(n, find_orthogonal, all),
        Mode::FindMOLS { n, mols } => match_n!(n, find_mols, mols),
        Mode::ToTex => to_tex(),
        Mode::Encode { n } => match_n!(n, encode),
        Mode::Decode { n } => match_n!(n, decode),
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
            let rcs: String = symmetry.apply_array(['R', 'C', 'S']).into_iter().collect();
            println!("{rcs}");
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

        let (isotopy_class, perm) = sq.isotopy_class_permutation();
        if isotopy_class != sq {
            println!("Isotopy class: ");
            println!("{}", isotopy_class);
            println!("Row permutation: {:?}", perm[0].as_array());
            println!("Col permutation: {:?}", perm[1].as_array());
            println!("Sym permutation: {:?}", perm[2].as_array());

            pretty_print_sq(isotopy_class);
        } else {
            println!("Is isotopy class reduced");
        }

        let (main_class, rcs, perm) = sq.main_class_permutation();
        if main_class != sq {
            println!("Main class: ");
            println!("{}", main_class);
            println!(
                "Conjugate: {}",
                rcs.apply_array(['R', 'C', 'S'])
                    .into_iter()
                    .collect::<String>()
            );
            println!("Row permutation: {:?}", perm[0].as_array());
            println!("Col permutation: {:?}", perm[1].as_array());
            println!("Sym permutation: {:?}", perm[2].as_array());

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
    let lookup = generate_minimize_rows_lookup_simd::<N>();
    for (i, sq) in IsotopyClassGenerator::<N>::new(&lookup).enumerate() {
        dbg!(i + 1);

        if writeln!(stdout(), "{sq}").is_err() {
            return;
        }
    }
}

fn generate_main_classes<const N: usize>(max_threads: usize) {
    let lookup = generate_minimize_rows_lookup_simd::<N>();
    // for (i, sq) in IsotopyClassGenerator::<N>::new(&lookup)
    //     .filter(|sq| *sq == sq.main_class_lookup(&lookup))
    //     .enumerate()
    // {
    //     dbg!(i + 1);

    //     if writeln!(stdout(), "{sq}").is_err() {
    //         return;
    //     }
    // }

    ThreadedMainClassGenerator::<N>::new(&lookup).run(max_threads);
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
                'h: for hitting_set in hitting_sets.by_ref() {
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

fn find_lcs(max_threads: usize) {
    let mut threads = Vec::new();

    while let Some(sq) = read_sq_from_stdin() {
        let thread = thread::spawn(move || find_lcs_sq(sq));

        threads.push(thread);

        while threads.len() >= max_threads {
            thread::sleep(Duration::from_millis(1));
            for i in 0..threads.len() {
                if !threads[i].is_finished() {
                    continue;
                }

                let thread = threads.swap_remove(i);
                thread.join().unwrap();
                break;
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
    for hitting_set in hitting_sets {
        let partial_sq = sq.mask(hitting_set);

        if !partial_sq.is_critical_set_of(&sq) {
            let num_entries = partial_sq.num_entries();

            'l: loop {
                for partial_sq in PartialSquareGeneratorDyn::new_partial(
                    sq.clone(),
                    partial_sq.clone(),
                    dbg!((lcs.num_entries() + 1).max(num_entries + 1)),
                )
                .filter(|s| s.is_critical_set_of(&sq))
                {
                    if lcs.num_entries() < partial_sq.num_entries() {
                        lcs = partial_sq;
                        continue 'l;
                    }
                }
                break;
            }

            continue;
        }

        if lcs.num_entries() < partial_sq.num_entries() {
            lcs = partial_sq;
        }
    }

    let mut stdout = stdout().lock();

    writeln!(stdout, "{}", sq).unwrap();
    writeln!(stdout, "{lcs}").unwrap();
    writeln!(stdout,).unwrap();
}

fn find_all_cs() {
    while let Some(sq) = read_sq_from_stdin() {
        println!("{sq}");
        let unavoidable_sets = sq.all_unavoidable_sets_order_1();
        dbg!(unavoidable_sets.len());

        let hitting_sets = MMCSHittingSetGenerator::new(vec![unavoidable_sets], sq.n() * sq.n());

        for hitting_set in hitting_sets {
            let partial_sq = sq.mask(hitting_set);

            if !partial_sq.is_critical_set_of(&sq) {
                dbg!(partial_sq);
                unreachable!();
            }

            println!("{partial_sq}");
        }

        println!();
    }
}

fn find_mols<const N: usize>(mols: usize) {
    let mut found = HashSet::new();

    let lookup = generate_minimize_rows_lookup();

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
                        let mols = MOLS::new_unchecked(&current_mols);
                        let normalized_mols = mols.normalize_main_class_set(&lookup);

                        if found.insert(normalized_mols.clone()) {
                            println!("{normalized_mols}");
                        }

                        // println!(
                        //     "{}",
                        //     current_mols
                        //         .iter()
                        //         .map(|sq| sq.to_string())
                        //         .reduce(|a, b| format!("{a}{}{b}", SEPARATOR))
                        //         .unwrap()
                        // );

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

fn solve() {
    for sq in read_partial_sqs_from_stdin() {
        let solutions = LatinSquareGeneratorDyn::from_partial_sq(&sq);

        for solution in solutions {
            println!("{}", solution);
        }
    }
}

fn shuffle() {
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

    let mut state = [1, 2, 3, 4];

    for mut sq in read_partial_sqs_from_stdin() {
        let n = sq.n();
        let rank = xoshiro(&mut state) as usize % factorial(n);

        let permutations = PermutationDyn::from_rank(rank, n);

        sq.permute_vals(&permutations);

        println!("{sq}");
    }
}

fn to_tex() {
    while let Some(sq) = read_partial_sq_from_stdin() {
        let n = sq.n();

        let args = (1..=n)
            .map(|i| format!("#{i}"))
            .reduce(|a, b| format!("{a}, {b}"))
            .unwrap();
        println!("% {}", sq);
        println!(
            "\\begin{{tikzpicture}}[scale=0.5]
    \\begin{{scope}}
        \\newcommand{{\\makerow}}[{n}]{{
        \\setcounter{{col}}{{0}};
        \\foreach \\n in {{{args}}} {{
            \\edef\\x{{\\value{{col}} + 0.5}}
                \\edef\\y{{{}.5 - \\value{{row}}}}
                \\node[anchor=center] at (\\x, \\y) {{\\n}};
                \\stepcounter{{col}}
            }}
            \\stepcounter{{row}}
        }}
        \\draw (0, 0) grid ({n}, {n});
        \\setcounter{{row}}{{0}};",
            n - 1
        );
        for i in 0..n {
            print!("        \\makerow");
            for j in 0..n {
                if let Some(v) = sq.get_partial(i, j) {
                    print!("{{{}}}", v + 1);
                } else {
                    print!("{{}}");
                }
            }
            println!();
        }
        println!("    \\end{{scope}}");
        println!("\\end{{tikzpicture}}");
    }
}

fn encode<const N: usize>() {
    let mut prev_sq = None;
    let mut buffer = Vec::new();
    let mut stdout = stdout();

    while let Some(sq) = read_sq_n_from_stdin::<N>() {
        encode_sq::<N>(sq, prev_sq, &mut buffer);

        stdout.write(&buffer).unwrap();

        prev_sq = Some(sq);
        buffer.clear();
    }
}

const fn row_size<const N: usize>() -> usize {
    let row_size_bits = (N - 1).pow(N as u32 - 2).next_power_of_two().ilog2();
    row_size_bits.div_ceil(8) as usize
}

fn decode<const N: usize>() {
    let row_size_bytes = row_size::<N>();
    let mut stdin = stdin();

    let mut prev_sq = None;

    loop {
        let mut same_rows = [0u8];
        stdin.read(&mut same_rows).unwrap();
        let same_rows = same_rows[0];
        assert!(same_rows <= N as u8);

        let mut buffer = [[0u8; 8]; N];

        for i in 0..N - 1 - same_rows as usize {
            stdin.read_exact(&mut buffer[i][0..row_size_bytes]).unwrap();
        }

        let sq = decode_sq(prev_sq.as_ref(), same_rows.into(), &buffer);
        prev_sq = Some(sq);

        println!("{}", sq);
    }
}

fn encode_sq<const N: usize>(
    sq: LatinSquare<N>,
    prev_sq: Option<LatinSquare<N>>,
    buffer: &mut Vec<u8>,
) {
    debug_assert!(sq.is_reduced());
    debug_assert!(prev_sq.is_none_or(|s| s.is_reduced()));

    let row_size_bytes = row_size::<N>();

    let same_rows = if let Some(prev_sq) = prev_sq {
        sq.num_same_rows(&prev_sq)
    } else {
        0
    };

    buffer.push(same_rows as u8);

    for row_index in same_rows..N - 1 {
        let row = sq.get_row(row_index);

        let mut coded = 0u64;
        for i in 1..N - 1 {
            coded *= N as u64 - 1;

            let value = if row[i] > row[0] { row[i] - 1 } else { row[i] };

            coded += value as u64;
        }

        buffer.extend(&coded.to_le_bytes()[0..row_size_bytes]);
    }
}

fn decode_sq<const N: usize>(
    prev_sq: Option<&LatinSquare<N>>,
    same_rows: usize,
    buffer: &[[u8; 8]; N],
) -> LatinSquare<N> {
    let mut rows = [[0; N]; N];
    let mut cols = [BitSet16::all_less_than(N); N];

    assert!(prev_sq.is_some() || same_rows == 0);

    if let Some(prev_sq) = prev_sq {
        for i in 0..same_rows {
            rows[i] = prev_sq.get_row(i).clone();
            for j in 0..N {
                cols[j].remove(rows[i][j].into());
            }
        }
    }

    for i in same_rows as usize..N - 1 {
        let mut coded = u64::from_le_bytes(buffer[i - same_rows]);

        let mut row = [0; N];
        row[0] = i as u8;
        cols[0].remove(i);

        let mut values = BitSet16::all_less_than(N);
        values.remove(i);

        for j in (1..N - 1).rev() {
            let value = (coded % (N - 1) as u64) as u8;
            coded /= (N - 1) as u64;

            let value = if value >= i as u8 { value + 1 } else { value };

            row[j] = value;
            values.remove(value.into());
            cols[j].remove(value.into());
        }
        assert!(values.is_single());
        let value = values.into_iter().next().unwrap() as u8;
        row[N - 1] = value;
        cols[N - 1].remove(value.into());

        rows[i] = row;
    }

    let last_row = cols.map(|c| {
        assert!(c.is_single());
        c.into_iter().next().unwrap() as u8
    });

    rows[N - 1] = last_row;

    let sq = LatinSquare::try_from(rows).unwrap();

    sq
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

fn read_sq_n_from_stdin<const N: usize>() -> Option<LatinSquare<N>> {
    let mut line = String::new();
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
}

fn read_sqs_from_stdin() -> impl Iterator<Item = LatinSquareDyn> {
    (0..).map_while(|_| read_sq_from_stdin())
}

fn read_partial_sq_from_stdin() -> Option<PartialLatinSquareDyn> {
    let mut line = String::new();
    while stdin().read_line(&mut line).is_ok_and(|i| i != 0) {
        line = line.trim().into(); // remove newline
        match PartialLatinSquareDyn::try_from(line.as_str()) {
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

fn read_partial_sqs_from_stdin() -> impl Iterator<Item = PartialLatinSquareDyn> {
    (0..).map_while(|_| read_partial_sq_from_stdin())
}
