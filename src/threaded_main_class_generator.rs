use std::{
    io::{stdout, Write},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    cycles::{generate_minimize_rows_lookup_simd, PermutationSimdLookup},
    isotopy_class_generator::RowGenerator,
    latin_square::LatinSquare,
    row_partial_latin_square::RowPartialLatinSquare,
};

/// Generates latin squares by filling them one row at a time
pub struct ThreadedMainClassGenerator<'a, const N: usize> {
    row_generators: Vec<RowGenerator<'a, N>>,
    lookup: &'a PermutationSimdLookup,
    threads: Vec<JoinHandle<()>>,
}

impl<'a, const N: usize> ThreadedMainClassGenerator<'a, N> {
    pub fn new(lookup: &'a PermutationSimdLookup) -> Self {
        ThreadedMainClassGenerator {
            row_generators: vec![RowGenerator::new(
                RowPartialLatinSquare::new_first_row(),
                lookup,
            )],
            lookup,
            threads: Vec::new(),
        }
    }

    pub fn run(mut self, max_threads: usize) {
        while let Some(generator) = self.row_generators.last_mut() {
            let Some(sq) = generator.next() else {
                self.row_generators.pop();
                continue;
            };

            if sq.is_complete() && sq.is_minimal_main_class(self.lookup) {
                let sq: LatinSquare<N> = sq.try_into().unwrap();

                let mut stdout = stdout();
                writeln!(stdout, "{sq}").unwrap();

                continue;
            }

            if self.row_generators.len() <= 2 || max_threads == 1 {
                self.row_generators.push(RowGenerator::new(sq, self.lookup));
            } else {
                while self.threads.len() >= max_threads {
                    for i in 0..self.threads.len() {
                        if !self.threads[i].is_finished() {
                            continue;
                        }

                        let thread = self.threads.swap_remove(i);
                        thread.join().unwrap();
                        break;
                    }
                    thread::sleep(Duration::from_micros(10));
                }

                let thread = thread::spawn(|| Self::run_thread(sq));
                self.threads.push(thread);
            }
        }

        for thread in self.threads {
            thread.join().unwrap();
        }
    }

    fn run_thread(sq: RowPartialLatinSquare<N>) {
        let lookup_simd = &generate_minimize_rows_lookup_simd::<N>();

        let mut row_generators = vec![RowGenerator::new(sq, lookup_simd)];
        let mut sqs = Vec::with_capacity(1000);

        while let Some(generator) = row_generators.last_mut() {
            let Some(sq) = generator.next() else {
                row_generators.pop();
                continue;
            };

            if sq.is_complete() && sq.is_minimal_main_class(lookup_simd) {
                let sq: LatinSquare<N> = sq.try_into().unwrap();

                sqs.push(sq);

                if sqs.len() >= 1000 {
                    let mut stdout = stdout().lock();
                    for sq in sqs.drain(..) {
                        writeln!(stdout, "{sq}").unwrap();
                    }
                }

                continue;
            }

            row_generators.push(RowGenerator::new(sq, lookup_simd));
        }

        let mut stdout = stdout().lock();
        for sq in sqs.drain(..) {
            writeln!(stdout, "{sq}").unwrap();
        }
    }
}
