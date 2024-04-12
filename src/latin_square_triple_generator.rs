use std::time::Instant;

use crate::{
    latin_square::{Cell, LatinSquare, PartialLatinSquare},
    triple_constraints::{CellOrValueTriple, TripleConstraints},
};

pub struct LatinSquareTripleGenerator<const N: usize> {
    stack: Vec<(TripleConstraints<N>, CellOrValueTriple, usize)>,
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

        LatinSquareTripleGenerator {
            stack: vec![(constraints.clone(), CellOrValueTriple::Cell(Cell(1, 0)), 0)],
        }
    }
}

impl<const N: usize> Iterator for LatinSquareTripleGenerator<N> {
    type Item = [LatinSquare<N>; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        let start = Instant::now();
        let mut best = 0;

        'w: while let Some((constraints, cell_or_value, start_value)) = self.stack.last_mut() {
            if let CellOrValueTriple::Cell(Cell(i, 0)) = *cell_or_value {
                dbg!(constraints.squares());

                let cell = Cell(i, 0);
                let values = constraints.values_for_cell(cell);

                for (j, value) in values.into_iter().enumerate().skip(*start_value) {
                    *start_value = j + 1;

                    let mut new = constraints.clone();
                    new.set(cell, value);

                    if !new.is_solvable() {
                        continue;
                    }

                    if i == N - 1 {
                        match new.most_constrained() {
                            Some(cell_or_values) => {
                                self.stack.push((new.clone(), cell_or_values, 0));
                                if self.stack.len() > best {
                                    best = self.stack.len();
                                    dbg!(new.squares(), best, Instant::now() - start);
                                }

                                continue 'w;
                            }
                            None => {
                                if constraints.is_solved() {
                                    return Some(constraints.squares().map(|sq| sq.into()));
                                }
                            }
                        }
                        continue 'w;
                    } else {
                        self.stack
                            .push((new.clone(), CellOrValueTriple::Cell(Cell(i + 1, 0)), 0));
                        if self.stack.len() > best {
                            best = self.stack.len();
                            dbg!(new.squares(), best, Instant::now() - start);
                        }
                        continue 'w;
                    }
                }
            } else {
                match cell_or_value {
                    CellOrValueTriple::Cell(cell) => {
                        let cell = *cell;
                        let values = constraints.values_for_cell(cell);

                        for (i, value) in values.into_iter().enumerate().skip(*start_value) {
                            *start_value = i + 1;

                            let mut new = constraints.clone();
                            new.set(cell, value);

                            if !new.is_solvable() {
                                continue;
                            }

                            match new.most_constrained() {
                                Some(cell_or_values) => {
                                    self.stack.push((new.clone(), cell_or_values, 0));
                                    if self.stack.len() > best {
                                        best = self.stack.len();
                                        dbg!(new.squares(), best, Instant::now() - start);
                                    }
                                    continue 'w;
                                }
                                None => {
                                    if constraints.is_solved() {
                                        return Some(constraints.squares().map(|sq| sq.into()));
                                    }
                                }
                            }
                        }
                    }
                    CellOrValueTriple::ValueTriple(value) => {
                        let start_i = *start_value % N;

                        for i in (0..N).skip(start_i) {
                            let mut value = *value;
                            if value[0] == N {
                                value[0] = i;
                            } else if value[1] == N {
                                value[1] = i;
                            } else {
                                value[2] = i;
                            }

                            let cells = constraints.cells_for_value(value);

                            let start_cell = *start_value / N;

                            for (j, cell) in cells.into_iter().enumerate().skip(start_cell) {
                                *start_value = i + (j + 1) * N;

                                let mut new = constraints.clone();
                                new.set(cell, value);

                                if !new.is_solvable() {
                                    continue;
                                }

                                match new.most_constrained() {
                                    Some(cell_or_values) => {
                                        self.stack.push((new.clone(), cell_or_values, 0));
                                        if self.stack.len() > best {
                                            best = self.stack.len();
                                            dbg!(new.squares(), best, Instant::now() - start);
                                        }
                                        continue 'w;
                                    }
                                    None => {
                                        if constraints.is_solved() {
                                            return Some(constraints.squares().map(|sq| sq.into()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            self.stack.pop();
        }

        None
    }
}
