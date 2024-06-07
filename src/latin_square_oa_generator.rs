use std::{
    fs::OpenOptions,
    io::{BufRead, Write},
    time::Instant,
};

use crate::{
    latin_square::LatinSquare,
    orthogonal_array::{OAConstraints, MOLS},
    partial_latin_square::PartialLatinSquare,
};

pub struct LatinSquareOAGenerator<const N: usize> {
    stack: Vec<(OAConstraints<N>, (usize, usize), usize)>,
}

impl<const N: usize> LatinSquareOAGenerator<N> {
    pub fn new() -> Self {
        let constraints = OAConstraints::new();

        let cell = constraints.most_constrained_cell().unwrap();
        LatinSquareOAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    pub fn new_reduced() -> Self {
        let constraints = OAConstraints::new_reduced(false);

        let cell = constraints.most_constrained_cell().unwrap_or((0, 0));
        LatinSquareOAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    pub fn new_reduced_diagonal() -> Self {
        let constraints = OAConstraints::new_reduced(true);

        let cell = constraints.most_constrained_cell().unwrap_or((0, 0));
        LatinSquareOAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    pub fn from_partial(sq: PartialLatinSquare<N>) -> Self {
        let constraints = OAConstraints::from_partial(sq);

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

        let _total = self
            .stack
            .iter()
            .map(|(constraints, cell, _)| constraints.values_for_cell(cell.0, cell.1).len() as f64)
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

        let mut new = Self::new_reduced();
        for val in vals {
            let Some((constraints, cell, start_value)) = new.stack.last_mut() else {
                return None;
            };
            let Some(val) = val else {
                return None;
            };

            let values = constraints.values_for_cell(cell.0, cell.1);

            let mut new_constraints: Vec<_> = values
                .into_iter()
                .map(|value| {
                    let mut new = constraints.clone();
                    new.set_and_propagate(cell.0, cell.1, value);
                    new.find_and_set_singles();
                    new
                })
                .collect();
            new_constraints.sort_by_cached_key(|c| {
                (
                    c.possible_values_log() as u64,
                    c.filled_cells().wrapping_neg(),
                )
            });

            let (i, constraints) = new_constraints.into_iter().enumerate().nth(val).unwrap();
            *start_value = i + 1;

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
            .map(|(constraints, cell, _)| constraints.values_for_cell(cell.0, cell.1).len() as f64)
            .collect();

        self.stack
            .iter()
            .enumerate()
            .map(|(i, (_, _, val))| {
                val.saturating_sub(1) as f64
                    / totals[0..=i]
                        .iter()
                        .copied()
                        .reduce(|a, b| a * b)
                        .unwrap_or(1.0)
            })
            .reduce(|a, b| a + b)
            .unwrap()
    }
}

impl<const N: usize> Iterator for LatinSquareOAGenerator<N> {
    type Item = [LatinSquare<N>; MOLS];

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        let _start = Instant::now();
        let mut last_write = Instant::now();
        let mut best = 0;

        'w: while let Some((constraints, cell, start_value)) = self.stack.last_mut() {
            if constraints.is_solved() {
                let map = constraints.squares().map(|sq| sq.into());
                self.stack.pop();
                return Some(map);
            }

            let cell = *cell;
            let values = constraints.values_for_cell(cell.0, cell.1);

            let mut new_constraints = values
                .into_iter()
                .map(|value| {
                    let mut new = constraints.clone();
                    new.set_and_propagate(cell.0, cell.1, value);
                    new.find_and_set_singles();
                    // dbg!(new.squares(), new.is_solvable());

                    new
                })
                .collect::<Vec<_>>();
            new_constraints
                .sort_by_cached_key(|c| (c.possible_values_log() as u64, c.filled_cells()));

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
                            // dbg!(new.squares(), best, Instant::now() - start);
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
