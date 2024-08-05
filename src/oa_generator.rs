use std::time::Instant;

use crate::{
    oa_constraints::OAConstraints, orthogonal_array::OrthogonalArray,
    partial_latin_square::PartialLatinSquare, partial_orthogonal_array::PartialOrthogonalArray,
};

pub struct OAGenerator<const N: usize, const MOLS: usize> {
    stack: Vec<(OAConstraints<N, MOLS>, (usize, usize), usize)>,
}

impl<const N: usize, const MOLS: usize> OAGenerator<N, MOLS> {
    pub fn new() -> Self {
        let constraints = OAConstraints::new();

        let cell = constraints.most_constrained_cell().unwrap();
        OAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    pub fn new_reduced() -> Self {
        let constraints = OAConstraints::new_reduced();

        let cell = constraints.most_constrained_cell().unwrap_or((0, 0));
        OAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    pub fn from_partial_sq(sq: PartialLatinSquare<N>) -> Self {
        let constraints = OAConstraints::from_partial_sq(sq);

        let cell = constraints.most_constrained_cell().unwrap_or((0, 0));
        OAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    pub fn from_partial_sq_reduced(sq: PartialLatinSquare<N>) -> Self {
        let constraints = OAConstraints::from_partial_sq_reduced(sq);

        let cell = constraints.most_constrained_cell().unwrap_or((0, 0));
        OAGenerator {
            stack: vec![(constraints, cell, 0)],
        }
    }

    pub fn from_partial_oa(oa: &PartialOrthogonalArray<N, MOLS>) -> Self {
        let constraints = OAConstraints::from_partial_oa(oa);

        let cell = constraints.most_constrained_cell().unwrap_or((0, 0));
        OAGenerator {
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
            let (constraints, cell, start_value) = new.stack.last_mut()?;
            let val = val?;

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

impl<const N: usize, const MOLS: usize> Iterator for OAGenerator<N, MOLS> {
    type Item = OrthogonalArray<N, MOLS>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        let _start = Instant::now();
        let mut last_write = Instant::now();
        let mut best = 0;

        'w: while let Some((constraints, cell, start_value)) = self.stack.last_mut() {
            if constraints.is_solved() {
                let sqs = constraints.squares().map(|sq| sq.try_into().unwrap());
                self.stack.pop();
                return Some(OrthogonalArray::new(sqs));
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
                            return Some(OrthogonalArray::new(
                                new.squares().map(|sq| sq.try_into().unwrap()),
                            ));
                        }
                    }
                }
            }

            self.stack.pop();
        }

        None
    }
}
