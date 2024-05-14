use std::{
    collections::hash_map::DefaultHasher,
    fs::OpenOptions,
    hash::Hasher,
    io::{BufRead, BufReader, Write},
    time::Instant,
};

use crate::{
    latin_square::{Cell, LatinSquare},
    triple_constraints::{CellOrValueTriple, TripleConstraints, ValueTriple},
};

pub struct LatinSquareTripleGenerator<const N: usize> {
    stack: Vec<(TripleConstraints<N>, Cell, usize)>,
}
impl<const N: usize> LatinSquareTripleGenerator<N> {
    pub fn new() -> Self {
        let mut constraints = TripleConstraints::new();

        for i in 0..N {
            let value = constraints
                .values_for_cell(Cell(0, i))
                .into_iter()
                .next()
                .unwrap();
            constraints.set(Cell(0, i), value);
        }

        for i in 1..N {
            constraints.set_first_value(Cell(i, 0), i);
        }

        LatinSquareTripleGenerator {
            stack: vec![(constraints.clone(), Cell(1, 0), 0)],
        }
    }

    fn shuffle(seed: usize, vec: &mut Vec<ValueTriple>) {
        vec.sort_by_key(|vals| {
            let mut default_hasher = DefaultHasher::new();
            default_hasher
                .write_usize((vals[0] + vals[1] * N + vals[2] * N.pow(2) + seed) % N.pow(3));
            default_hasher.finish()
        })
    }

    fn save_indices(&self) {
        return;
        let vals: Vec<_> = self
            .stack
            .iter()
            .map(|(_, _, val)| val.saturating_sub(1))
            .collect();

        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("stack.txt")
        else {
            return;
        };

        let string = vals
            .into_iter()
            .map(|val| format!("{val}"))
            .reduce(|a, b| format!("{a}, {b}"))
            .unwrap();

        writeln!(file, "{string}").unwrap();
    }

    pub fn load() -> Option<Self> {
        let Ok(file) = OpenOptions::new().read(true).open("stack.txt") else {
            return None;
        };

        let string = BufReader::new(file).lines().last().unwrap().unwrap();

        let vals: Vec<usize> = string
            .split(',')
            .map(|val| val.trim().parse().unwrap())
            .collect();

        let mut new = Self::new();
        for val in vals {
            let Some((constraints, cell, start_value)) = new.stack.last_mut() else {
                return None;
            };
            let cell = *cell;
            let values = constraints.values_for_cell(cell);
            let (i, value) = values.into_iter().enumerate().skip(val).next().unwrap();
            *start_value = i + 1;

            let mut constraints = constraints.clone();
            constraints.set(cell, value);
            constraints.find_and_set_singles();
            match constraints.most_constrained() {
                Some(CellOrValueTriple::Cell(cell)) => {
                    new.stack.push((constraints, cell, 0));
                }
                _ => return None,
            }
        }

        Some(new)
    }
}

impl<const N: usize> Iterator for LatinSquareTripleGenerator<N> {
    type Item = [LatinSquare<N>; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        dbg!(self.stack.last().unwrap().0.squares());

        let start = Instant::now();
        let _last_write = Instant::now();
        let mut best = 0;

        'w: while let Some((constraints, cell, start_value)) = self.stack.last_mut() {
            // if let CellOrValueTriple::Cell(Cell(i, 0)) = *cell_or_value {
            //     dbg!(constraints.squares());

            //     let cell = Cell(i, 0);
            //     let values = constraints.values_for_cell(cell);

            //     for (j, value) in values.into_iter().enumerate().skip(*start_value) {
            //         *start_value = j + 1;

            //         let mut new = constraints.clone();
            //         new.set(cell, value);

            //         if !new.is_solvable() {
            //             continue;
            //         }

            //         if i == N - 1 {
            //             match new.most_constrained() {
            //                 Some(cell_or_values) => {
            //                     self.stack.push((new.clone(), cell_or_values, 0));
            //                     if self.stack.len() > best {
            //                         best = self.stack.len();
            //                         dbg!(new.squares(), best, Instant::now() - start);
            //                     }

            //                     continue 'w;
            //                 }
            //                 None => {
            //                     if constraints.is_solved() {
            //                         return Some(constraints.squares().map(|sq| sq.into()));
            //                     }
            //                 }
            //             }
            //             continue 'w;
            //         } else {
            //             self.stack
            //                 .push((new.clone(), CellOrValueTriple::Cell(Cell(i + 1, 0)), 0));
            //             if self.stack.len() > best {
            //                 best = self.stack.len();
            //                 dbg!(new.squares(), best, Instant::now() - start);
            //             }
            //             continue 'w;
            //         }
            //     }
            // } else
            {
                // match cell_or_value {
                //     CellOrValueTriple::Cell(cell) => {
                let cell = *cell;
                let values = constraints.values_for_cell(cell);

                // if cell.1 == 0 {
                // Self::shuffle(self.seed + cell.to_index::<N>(), &mut values);
                // }

                for (i, value) in values.into_iter().enumerate().skip(*start_value) {
                    *start_value = i + 1;

                    let mut new = constraints.clone();
                    new.set(cell, value);
                    new.find_and_set_singles();

                    if !new.is_solvable() {
                        // if (Instant::now() - last_write).as_secs() >= 10 {
                        self.save_indices();
                        //     last_write = Instant::now();
                        // }
                        continue 'w;
                    }

                    match new.most_constrained() {
                        Some(CellOrValueTriple::Cell(cell)) => {
                            self.stack.push((new.clone(), cell, 0));
                            if new.filled_cells() >= best {
                                best = new.filled_cells();
                                dbg!(new.squares(), best, Instant::now() - start);
                            }
                            // if (Instant::now() - last_write).as_secs() >= 10 {
                            self.save_indices();
                            //     last_write = Instant::now();
                            // }
                            continue 'w;
                        }

                        None => {
                            if new.is_solved() {
                                return Some(new.squares().map(|sq| sq.into()));
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            // CellOrValueTriple::ValueTriple(value) => {
            //     unreachable!();
            //     let start_i = *start_value % N;

            //     for i in (0..N).skip(start_i) {
            //         let mut value = *value;
            //         if value[0] == N {
            //             value[0] = i;
            //         } else if value[1] == N {
            //             value[1] = i;
            //         } else {
            //             value[2] = i;
            //         }

            //         let cells = constraints.cells_for_value(value);

            //         let start_cell = *start_value / N;

            //         for (j, cell) in cells.into_iter().enumerate().skip(start_cell) {
            //             *start_value = i + (j + 1) * N;

            //             let mut new = constraints.clone();
            //             new.set(cell, value);

            //             if !new.is_solvable() {
            //                 continue;
            //             }

            //             match new.most_constrained() {
            //                 Some(cell_or_values) => {
            //                     self.stack.push((new.clone(), cell_or_values, 0));
            //                     if new.filled_cells() >= best {
            //                         best = new.filled_cells();
            //                         dbg!(new.squares(), best, Instant::now() - start);
            //                     }
            //                     continue 'w;
            //                 }
            //                 None => {
            //                     if new.is_solved() {
            //                         return Some(new.squares().map(|sq| sq.into()));
            //                     }
            //                 }
            //             }
            //         }
            //     }
            // }
            // }
            // }

            self.stack.pop();
        }

        None
    }
}
