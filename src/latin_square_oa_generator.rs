use std::{
    cell,
    collections::hash_map::DefaultHasher,
    fs::OpenOptions,
    hash::Hasher,
    io::{BufRead, BufReader, Write},
    time::{Duration, Instant},
};

use crate::{
    latin_square::{Cell, LatinSquare, PartialLatinSquare},
    orthogonal_array::{self, OAConstraints, MOLS, N},
    pair_constraints::CellOrValuePair,
    triple_constraints::{CellOrValueTriple, TripleConstraints, ValueTriple},
};

pub struct LatinSquareOAGenerator {
    stack: Vec<(OAConstraints, Cell, usize)>,
}

impl LatinSquareOAGenerator {
    pub fn new() -> Self {
        let mut constraints = OAConstraints::new();

        let cell = constraints.most_constrained_cell().unwrap();
        LatinSquareOAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    fn save_indices(&self) {
        let string = self
            .stack
            .iter()
            .map(|(_, _, val)| val.saturating_sub(1))
            .map(|val| format!("{val}"))
            .reduce(|a, b| format!("{a},{b}"))
            .unwrap();

        let total = self
            .stack
            .iter()
            .map(|(constraints, cell, _)| constraints.values_for_cell(*cell).len() as f64)
            .reduce(|a, b| a * b)
            .unwrap();

        dbg!(self.progress());
        dbg!(string);
        return;

        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("stack_oa.txt")
        else {
            return;
        };

        writeln!(file, "{string}").unwrap();
    }

    pub fn load(string: String) -> Option<Self> {
        // let Ok(file) = OpenOptions::new().read(true).open("stack_oa.txt") else {
        //     return None;
        // };

        // let string = BufReader::new(file).lines().last().unwrap().unwrap();

        let vals: Vec<_> = string
            .split(',')
            .map(|val| val.trim().parse().ok())
            .collect();

        let mut new = Self::new();
        for val in vals {
            let Some((constraints, cell, start_value)) = new.stack.last_mut() else {
                return None;
            };
            let Some(val) = val else {
                return None;
            };

            let cell = *cell;
            let values = constraints.values_for_cell(cell);
            let (i, value) = values.into_iter().enumerate().skip(val).next().unwrap();
            *start_value = i + 1;

            let mut constraints = constraints.clone();
            constraints.set_and_propagate(cell, value);
            constraints.find_and_set_singles();
            match constraints.most_constrained_cell() {
                Some(cell) => {
                    new.stack.push((constraints, cell, 0));
                }
                _ => return None,
            }
        }

        Some(new)
    }

    fn progress(&self) -> f64 {
        let totals: Vec<_> = self
            .stack
            .iter()
            .map(|(constraints, cell, _)| constraints.values_for_cell(*cell).len() as f64)
            .collect();

        self.stack
            .iter()
            .enumerate()
            .map(|(i, (_, _, val))| {
                val.saturating_sub(1) as f64
                    / totals[0..=i]
                        .iter()
                        .map(|val| (*val) as f64)
                        .reduce(|a, b| a * b)
                        .unwrap_or(1.0)
            })
            .reduce(|a, b| a + b)
            .unwrap()
    }
}

impl Iterator for LatinSquareOAGenerator {
    type Item = [LatinSquare<N>; MOLS];

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        let start = Instant::now();
        let mut last_write = Instant::now();
        let mut best = 0;

        'w: while let Some((constraints, cell, start_value)) = self.stack.last_mut() {
            let cell = *cell;
            let values = constraints.values_for_cell(cell);

            let mut new_constraints: Vec<_> = values
                .into_iter()
                .map(|value| {
                    let mut new = constraints.clone();
                    new.set_and_propagate(cell, value);
                    new.find_and_set_singles();
                    new
                })
                .collect();
            new_constraints
                .sort_by_cached_key(|c| (c.sum_possible_values() as u64, c.filled_cells()));
            new_constraints.reverse();

            for (i, new) in new_constraints.into_iter().enumerate().skip(*start_value) {
                *start_value = i + 1;

                if !new.is_solvable() {
                    let time_passed = (Instant::now() - last_write).as_secs_f64();
                    if time_passed >= 1.0 {
                        self.save_indices();

                        last_write = Instant::now();
                    }
                    continue 'w;
                }

                match new.most_constrained_cell() {
                    Some(cell) => {
                        self.stack.push((new.clone(), cell, 0));
                        if new.filled_cells() >= best {
                            best = new.filled_cells();
                            dbg!(new.squares(), best, Instant::now() - start);
                        }
                        let time_passed = (Instant::now() - last_write).as_secs_f64();
                        if time_passed >= 1.0 {
                            self.save_indices();

                            last_write = Instant::now();
                        }
                        continue 'w;
                    }

                    None => {
                        if new.is_solved() {
                            return Some(new.squares().map(|sq| sq.into()));
                        }
                    }
                }
            }

            self.stack.pop();
        }

        None
    }
}
