#![feature(portable_simd)]

use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    io::{stdin, stdout, Read, Write},
    sync::Arc,
    thread::{self},
    time::Duration,
};

use bitset::{BitSet128, BitSet16};
use clap::{self, Parser, Subcommand};

use cycles::{generate_minimize_rows_lookup, generate_minimize_rows_lookup_simd};
use isotopy_class_generator::IsotopyClassGenerator;
use latin_square::LatinSquare;

use latin_square_dyn::LatinSquareDyn;
use latin_square_generator::LatinSquareGeneratorDyn;

use mmcs_hitting_set_generator::MMCSHittingSetGenerator;

use partial_latin_square_dyn::PartialLatinSquareDyn;
use partial_square_generator::PartialSquareGeneratorDyn;
use permutation::{factorial, Permutation};
use permutation_dyn::PermutationDyn;
use random_latin_square_generator::RandomLatinSquareGeneratorDyn;
use threaded_main_class_generator::ThreadedMainClassGenerator;

mod bitset;
mod bitvec;
mod constraints;
mod cycles;
mod isotopy_class_generator;
mod latin_square;
mod latin_square_dyn;
mod latin_square_generator;
mod mmcs_hitting_set_generator;
mod mols;
mod partial_latin_square;
mod partial_latin_square_dyn;
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
    /// Prints a latin square in a 2D grid
    PrettyPrint,
    /// Prints all solutions for a partial latin square
    Solve,
    CountSubsquares {
        k: usize,
    },
    CountEntries,
    /// Counts the number of isotopy classes in the given main classes
    CountIsotopyClasses {
        n: usize,
    },
    CountTransversals {
        n: usize,
    },
    SubTransversals {
        n: usize,
        k: usize,
    },
    /// Prints information about a latin square
    Analyse {
        n: usize,
    },
    /// Prints the main class representative of a latin square
    NormalizeMainClass {
        n: usize,
    },
    /// Generates all latin squares of an order n
    GenerateLatinSquares {
        n: usize,
    },
    /// Generates a representative of each isotopy class of an order n
    GenerateIsotopyClasses {
        n: usize,
    },
    /// Generates a representative of each main class of an order n
    GenerateMainClasses {
        n: usize,
        #[arg(long, default_value_t = 1)]
        max_threads: usize,
    },
    /// Generates all critical sets for a latin square in a binary format.
    /// The resulting data can be decoded with `decode-cs`
    FindAllCS,
    FindSCS {
        #[arg(short, long)]
        reverse: bool,
    },
    FindLCS {
        #[arg(long, default_value_t = 1)]
        max_threads: usize,
    },
    FindOrthogonal {
        n: usize,
        #[arg(short, long)]
        all: bool,
    },
    FindMOLS {
        n: usize,
        mols: usize,
    },
    FindAllMOLS {
        n: usize,
        #[arg(long, default_value_t = 1)]
        max_threads: usize,
        #[arg(long, default_value_t = 10)]
        buffer_size: usize,
    },
    ToTex {
        #[arg(long, default_value_t = false)]
        standalone: bool,
    },
    Encode {
        n: usize,
    },
    Decode {
        n: usize,
    },
    DecodeCS,
    Expand {
        n: usize,
    },
    // Generates pseudo-random latin squares
    Random {
        n: usize,
        seed: u64,
    },
    /// Permutes the symbols of a latin square randomly
    Shuffle {
        #[arg(short)]
        r: bool,
        #[arg(short)]
        c: bool,
        #[arg(short)]
        s: bool,
        #[arg(long)]
        seed: u64,
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
                11 => $f::<11>($($args),*),
                _ => unimplemented!(),
            }
        };
    }

    match args.mode {
        Mode::Analyse { n } => match_n!(n, analyse),
        Mode::CountSubsquares { k } => count_subsquares(k),
        Mode::CountEntries => count_entries(),
        Mode::CountIsotopyClasses { n } => match_n!(n, count_isotopy_classes),
        Mode::CountTransversals { n } => match_n!(n, count_transversals),
        Mode::SubTransversals { n, k } => match_n!(n, sub_transversals, k),
        Mode::PrettyPrint => pretty_print(),
        Mode::NormalizeMainClass { n } => match_n!(n, normalize_main_class),
        Mode::GenerateLatinSquares { n } => generate_latin_squares(n),
        Mode::GenerateIsotopyClasses { n } => match_n!(n, generate_isotopy_classes),
        Mode::GenerateMainClasses { n, max_threads } => {
            match_n!(n, generate_main_classes, max_threads)
        }
        Mode::Solve => solve(),
        Mode::Shuffle { r, c, s, seed } => shuffle(seed, r, c, s),
        Mode::FindAllCS => find_all_cs(),
        Mode::FindLCS { max_threads } => find_lcs(max_threads),
        Mode::FindSCS { reverse } => find_scs(reverse),
        Mode::Random { n, seed } => random_latin_squares(n, seed),
        Mode::FindOrthogonal { n, all } => match_n!(n, find_orthogonal, all),
        Mode::FindMOLS { n, mols } => match_n!(n, find_mols, mols),
        Mode::FindAllMOLS {
            n,
            max_threads,
            buffer_size,
        } => match_n!(n, find_all_mols, max_threads, buffer_size),
        Mode::ToTex { standalone } => to_tex(standalone),
        Mode::Encode { n } => match_n!(n, encode),
        Mode::Decode { n } => match_n!(n, decode),
        Mode::DecodeCS => decode_cs(),
        Mode::Expand { n } => match_n!(n, expand),
    }
}

fn count_subsquares(k: usize) {
    while let Some(sq) = read_sq_from_stdin() {
        println!("{}", sq.num_subsquares_dyn(k));
    }
}

fn find_orthogonal<const N: usize>(all: bool) {
    while let Some(sq) = read_sq_from_stdin_n::<N>() {
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

fn random_latin_squares(n: usize, seed: u64) {
    for sq in RandomLatinSquareGeneratorDyn::new(n, seed) {
        if writeln!(stdout(), "{}", sq).is_err() {
            return;
        }
    }
}

fn analyse<const N: usize>() {
    let lookup = generate_minimize_rows_lookup();

    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        pretty_print_sq_n(sq);

        for i in 2..N {
            println!("Subsquares order {i}: {}", sq.num_subsquares(i));
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
            sq.full_disjoint_transversals_bitset().len()
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

        let (isotopy_class, perm) = sq.isotopy_class_permutations(&lookup);
        if isotopy_class != sq {
            println!("Isotopy class: ");
            println!("{}", isotopy_class);
            println!("Row permutation: {:?}", perm[0][0].as_array());
            println!("Col permutation: {:?}", perm[0][1].as_array());
            println!("Sym permutation: {:?}", perm[0][2].as_array());

            pretty_print_sq_n(isotopy_class);
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

            pretty_print_sq_n(main_class);
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
    while let Some(sq) = read_partial_sq_from_stdin() {
        pretty_print_sq(sq);
    }
}

fn pretty_print_sq(sq: PartialLatinSquareDyn) {
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

fn pretty_print_sq_n<const N: usize>(sq: LatinSquare<N>) {
    let n = N;

    for i in 0..n {
        println!("+{}", "---+".repeat(n));
        print!("|");
        for j in 0..n {
            let value = sq.get(i, j);
            print!(" {} |", value);
        }
        println!()
    }
    println!("+{}", "---+".repeat(n));
    println!()
}

fn normalize_main_class<const N: usize>() {
    let lookup = generate_minimize_rows_lookup();

    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        if writeln!(stdout(), "{}", sq.main_class_lookup(&lookup)).is_err() {
            return;
        }
    }
}

fn generate_isotopy_classes<const N: usize>() {
    let lookup = generate_minimize_rows_lookup_simd::<N>();
    for sq in IsotopyClassGenerator::<N>::new(&lookup) {
        if writeln!(stdout(), "{sq}").is_err() {
            return;
        }
    }
}

fn generate_main_classes<const N: usize>(max_threads: usize) {
    let lookup = generate_minimize_rows_lookup_simd::<N>();

    ThreadedMainClassGenerator::<N>::new(&lookup).run(max_threads);
}

const KNOWN_SCS: [usize; 9] = [0, 0, 1, 2, 4, 6, 9, 12, 16];

fn find_scs(reverse: bool) {
    while let Some(sq) = read_sq_from_stdin() {
        let differences = sq.differences();
        dbg!(differences.len());

        let start = *KNOWN_SCS.get(sq.n()).unwrap_or(&sq.n());
        let end = sq.n().pow(2) - 1;

        if !reverse {
            for i in start..=end {
                dbg!(i);
                let hitting_sets = MMCSHittingSetGenerator::new(differences.clone(), i);

                let mut found = false;
                'h: for hitting_set in hitting_sets {
                    let partial_sq = sq.mask(hitting_set);

                    for partial_sq in
                        PartialSquareGeneratorDyn::new_partial(sq.clone(), partial_sq.clone(), i)
                    {
                        if partial_sq.is_uniquely_completable_to(&sq) {
                            found = true;
                            println!("{sq}");
                            println!("{partial_sq}");
                            break 'h;
                        }
                    }
                }

                if found {
                    break;
                }
            }
        } else {
            let mut hitting_sets = MMCSHittingSetGenerator::new(differences, end);
            let mut scs = PartialLatinSquareDyn::empty(sq.n());
            for i in (start..=end).rev() {
                dbg!(i);

                let mut found = false;
                'h: for hitting_set in hitting_sets.by_ref() {
                    let partial_sq = sq.mask(hitting_set);

                    for partial_sq in
                        PartialSquareGeneratorDyn::new_partial(sq.clone(), partial_sq, i)
                    {
                        if partial_sq.is_uniquely_completable_to(&sq) {
                            found = true;
                            scs = partial_sq;
                            dbg!(scs.to_string());
                            break 'h;
                        }
                    }
                }
                hitting_sets.decrease_max_entries();

                if !found {
                    println!("{sq}");
                    println!("{scs}");
                    break;
                }
            }
        }
        println!();
    }
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

    for thread in threads {
        thread.join().unwrap();
    }
}

fn find_lcs_sq(sq: LatinSquareDyn) {
    let differences = sq.differences();

    let hitting_sets = MMCSHittingSetGenerator::new(differences, sq.n() * sq.n());

    let mut lcs = PartialLatinSquareDyn::empty(sq.n());
    let mut all_lcs = Vec::new();

    for hitting_set in hitting_sets {
        let partial_sq = sq.mask(hitting_set);

        if !partial_sq.is_critical_set_of(&sq) {
            let num_entries = partial_sq.num_entries();

            'l: loop {
                for partial_sq in PartialSquareGeneratorDyn::new_partial(
                    sq.clone(),
                    partial_sq.clone(),
                    (lcs.num_entries() + 1).max(num_entries + 1),
                )
                .filter(|s| s.is_critical_set_of(&sq))
                {
                    if lcs.num_entries() < partial_sq.num_entries() {
                        lcs = partial_sq.clone();
                        all_lcs = vec![partial_sq];
                        continue 'l;
                    } else if lcs.num_entries() == partial_sq.num_entries() {
                        all_lcs.push(partial_sq);
                    }
                }
                break;
            }
        } else {
            if lcs.num_entries() < partial_sq.num_entries() {
                lcs = partial_sq.clone();
                all_lcs = vec![partial_sq];
            } else if lcs.num_entries() == partial_sq.num_entries() {
                all_lcs.push(partial_sq);
            }
        }
    }

    let mut stdout = stdout().lock();

    writeln!(stdout, "{}", sq).unwrap();
    for lcs in all_lcs {
        writeln!(stdout, "{lcs}").unwrap();
    }
    writeln!(stdout,).unwrap();
}

fn find_all_cs() {
    while let Some(sq) = read_sq_from_stdin() {
        let mut differences = sq.differences();
        dbg!(differences.len());

        let hitting_sets = MMCSHittingSetGenerator::new(differences.clone(), sq.n() * sq.n());

        for hitting_set in hitting_sets {
            let partial_sq = sq.mask(hitting_set);

            if !partial_sq.is_critical_set_of(&sq) {
                // dbg!(&partial_sq);
                for solution in LatinSquareGeneratorDyn::from_partial_sq(&partial_sq) {
                    let difference = sq.difference_mask(&solution);

                    if !difference.is_empty()
                        && !differences.iter().any(|s| s.is_subset_of(difference))
                    {
                        differences.retain(|s| !difference.is_subset_of(*s));
                        differences.push(difference);
                        dbg!(differences.len());
                    }
                }
            }
        }

        let critical_sets = MMCSHittingSetGenerator::new(differences.clone(), sq.n() * sq.n());

        let bytes_needed = (sq.n() * sq.n()).div_ceil(8);

        let mut stdout = stdout();

        for set in critical_sets {
            let partial_sq = sq.mask(set);

            if !partial_sq.is_critical_set_of(&sq) {
                dbg!(partial_sq);
                unreachable!();
            }

            stdout
                .write_all(&set.bits().to_le_bytes()[0..bytes_needed])
                .unwrap();
        }
    }
}

fn decode_cs() {
    let Some(sq) = read_sq_from_stdin() else {
        eprintln!("No square provided");
        return;
    };

    let bytes_needed = (sq.n() * sq.n()).div_ceil(8);

    let mut stdin = stdin();

    let mut buffer = [0; 16];

    while stdin.read_exact(&mut buffer[0..bytes_needed]).is_ok() {
        let bitset = BitSet128::from_bits(u128::from_le_bytes(buffer));

        let partial_sq = sq.mask(bitset);

        println!("{partial_sq}");
    }
}

fn find_mols<const N: usize>(mols: usize) {
    let lookup = generate_minimize_rows_lookup();

    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        let mols = sq.kmols(mols, lookup.as_slice());
        let mut stdout = stdout().lock();
        for mols in mols {
            writeln!(stdout, "{mols}").unwrap();
        }
    }
}

fn find_all_mols<const N: usize>(max_threads: usize, buffer_size: usize) {
    let lookup = Arc::new(generate_minimize_rows_lookup());

    if max_threads == 1 {
        while let Some(sq) = read_sq_from_stdin_n() {
            find_all_mols_for_sq(sq, lookup.clone());
        }
        return;
    }

    let mut threads = Vec::new();

    let mut buffer: Vec<LatinSquare<N>> = Vec::new();

    while let Some(sq) = read_sq_from_stdin_n() {
        buffer.push(sq);

        if buffer.len() < buffer_size {
            continue;
        }

        let lookup = lookup.clone();
        let move_buffer = std::mem::take(&mut buffer);

        let thread = thread::spawn(move || {
            for sq in move_buffer {
                find_all_mols_for_sq(sq, lookup.clone())
            }
        });

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
    let lookup = lookup.clone();
    let move_buffer = std::mem::take(&mut buffer);

    let thread = thread::spawn(move || {
        for sq in move_buffer {
            find_all_mols_for_sq(sq, lookup.clone())
        }
    });

    threads.push(thread);

    for thread in threads {
        thread.join().unwrap();
    }
}

fn find_all_mols_for_sq<const N: usize>(
    sq: LatinSquare<N>,
    lookup: Arc<Vec<Vec<(Permutation<N>, Permutation<N>)>>>,
) {
    let mols = sq.mols(lookup.as_slice());
    let mut stdout = stdout().lock();
    for mols in mols {
        writeln!(stdout, "{mols}").unwrap();
    }
}

fn solve() {
    while let Some(sq) = read_partial_sq_from_stdin() {
        let solutions = LatinSquareGeneratorDyn::from_partial_sq(&sq);

        for solution in solutions {
            println!("{}", solution);
        }
    }
}

fn count_entries() {
    let mut counts = Vec::new();
    while let Some(sq) = read_partial_sq_from_stdin() {
        let size = sq.n().pow(2);
        if size > counts.len() {
            counts.resize(size + 1, 0);
        }

        let num_entries = sq.num_entries();
        counts[num_entries] += 1;

        println!("{sq}");
    }

    for (num_entries, count) in counts.into_iter().enumerate() {
        println!("{num_entries}: {count}");
    }
}

fn count_isotopy_classes<const N: usize>() {
    let lookup = generate_minimize_rows_lookup();
    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        println!("{}", sq.num_isotopy_classes(&lookup));
    }
}

fn sub_transversals<const N: usize>(k: usize) {
    assert!(k <= N);

    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        let subsquares = sq.subsquares_bitset(k);
        let transversals = sq.transversals_bitset();

        let mut subtransversals_per_subsq = Vec::new();

        for subsquare in &subsquares {
            let mut subtransversals = Vec::new();

            for transversal in &transversals {
                if subsquare.intersect(*transversal).len() == k {
                    subtransversals.push(sq.mask(*transversal));
                }
            }

            if !subtransversals.is_empty() {
                subtransversals_per_subsq.push((subsquare, subtransversals));
            }
        }

        if !subtransversals_per_subsq.is_empty() {
            println!("{sq}");
            for (subsq, subtransversals) in subtransversals_per_subsq {
                println!("{}", sq.mask(*subsq));

                for subtransversal in subtransversals {
                    println!("{subtransversal}")
                }

                println!()
            }
            println!()
        }
    }
}

fn expand<const N: usize>() {
    let lookup = generate_minimize_rows_lookup();

    // let mut last_layer = HashSet::new();
    // let mut next_layer = HashSet::new();
    // let mut queue = HashSet::new();

    // while let Some(sq) = read_sq_from_stdin_n::<N>() {
    //     queue.insert(sq);
    // }

    // while !queue.is_empty() {
    //     for sq in queue.iter() {
    //         println!("{sq}");
    //         for mate in sq
    //             .orthogonal_squares()
    //             .map(|sq| sq.main_class_lookup(&lookup))
    //         {
    //             next_layer.insert(mate);
    //         }
    //     }
    //     last_layer.clear();
    //     std::mem::swap(&mut last_layer, &mut queue);
    //     std::mem::swap(&mut next_layer, &mut queue);
    // }

    let mut queue = BinaryHeap::new();
    let mut found = HashSet::new();

    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        let sq = sq.main_class_lookup(&lookup);
        found.insert(sq);
        queue.push((sq.num_transversals(), sq));
    }

    while let Some((t, sq)) = queue.pop() {
        dbg!(t, queue.len(), found.len());
        println!("{sq}");

        let mut mates: Vec<_> = sq
            .orthogonal_squares()
            .map(|s| s.main_class_lookup(&lookup))
            .collect();
        mates.sort();
        mates.dedup();

        for mate in mates {
            if found.insert(mate) {
                queue.push((mate.num_transversals(), mate));
            }
        }
    }
}

fn count_transversals<const N: usize>() {
    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        println!("{}", sq.num_transversals());
    }
}

fn shuffle(seed: u64, rows: bool, cols: bool, vals: bool) {
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

    let mut state = [seed, 2, 3, 4];

    for _ in 0..100 {
        xoshiro(&mut state);
    }

    while let Some(mut sq) = read_partial_sq_from_stdin() {
        let n = sq.n();

        if rows {
            let rank = xoshiro(&mut state) as usize % factorial(n);
            let permutations = PermutationDyn::from_rank(rank, n);

            sq.permute_rows(&permutations);
        }

        if cols {
            let rank = xoshiro(&mut state) as usize % factorial(n);
            let permutations = PermutationDyn::from_rank(rank, n);

            sq.permute_cols(&permutations);
        }

        if vals {
            let rank = xoshiro(&mut state) as usize % factorial(n);
            let permutations = PermutationDyn::from_rank(rank, n);

            sq.permute_vals(&permutations);
        }

        println!("{sq}");
    }
}

fn to_tex(standalone: bool) {
    if standalone {
        println!(
            "\\documentclass[preview]{{standalone}}
\\usepackage{{tikz}}
\\newcounter{{row}}
\\newcounter{{col}}
\\begin{{document}}"
        );
    }
    println!("\\begin{{tikzpicture}}[scale=0.5]");

    let mut first_n = None;
    let mut x = 0;
    let mut y = 0;
    while let Some(sq) = read_partial_sq_from_stdin() {
        let n = sq.n();

        if first_n.is_none() {
            first_n = Some(n);
        }

        if n != first_n.unwrap() {
            eprintln!("All squares must be the same size");
            return;
        }

        println!("% {}", sq);
        println!(
            "    \\begin{{scope}}[xshift = {}cm, yshift = {}cm]
        \\draw (0, 0) grid ({n}, {n});",
            x * (n + 1),
            y * (n + 1)
        );

        dbg!(x, y);
        if x == y {
            y = x + 1;
            x = 0;
        } else if x < y {
            x += 1;
            if x == y {
                y = 0;
            }
        } else if x > y {
            y += 1;
        }

        if n <= 9 {
            let args = (1..=n)
                .map(|i| format!("#{i}"))
                .reduce(|a, b| format!("{a}, {b}"))
                .unwrap();
            println!(
                "        \\newcommand{{\\makerow}}[{n}]{{
        \\setcounter{{col}}{{0}}
        \\foreach \\n in {{{args}}} {{
            \\edef\\x{{\\value{{col}} + 0.5}}
                \\edef\\y{{{}.5 - \\value{{row}}}}
                \\node[anchor=center] at (\\x, \\y) {{\\n}};
                \\stepcounter{{col}}
            }}
            \\stepcounter{{row}}
        }}
        \\setcounter{{row}}{{0}}",
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
        } else {
            for i in 0..n {
                for j in 0..n {
                    if let Some(v) = sq.get_partial(i, j) {
                        print!(
                            "\\node[anchor=center] at ({j}.5, {}.5) {{{}}};",
                            n - i - 1,
                            v + 1
                        );
                    }
                }
                println!();
            }
        }
        println!("    \\end{{scope}}");
    }
    println!("\\end{{tikzpicture}}");

    if standalone {
        println!("\\end{{document}}");
    }
}

fn encode<const N: usize>() {
    let mut prev_sq = None;
    let mut buffer = Vec::new();
    let mut stdout = stdout();

    while let Some(sq) = read_sq_from_stdin_n::<N>() {
        encode_sq::<N>(sq, prev_sq, &mut buffer);

        stdout.write_all(&buffer).unwrap();

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
        stdin.read_exact(&mut same_rows).unwrap();
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
            rows[i] = *prev_sq.get_row(i);
            for j in 0..N {
                cols[j].remove(rows[i][j].into());
            }
        }
    }

    for i in same_rows..N - 1 {
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

    LatinSquare::try_from(rows).unwrap()
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

fn read_sq_from_stdin_n<const N: usize>() -> Option<LatinSquare<N>> {
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
