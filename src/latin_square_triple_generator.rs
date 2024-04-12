use crate::{
    latin_square::{Cell, LatinSquarePair, LatinSquareTriple, PartialLatinSquare},
    pair_constraints::{CellOrValuePair, PairConstraints},
    triple_constraints::{CellOrValueTriple, TripleConstraints},
};

pub struct LatinSquareTripleGenerator<const N: usize> {
    stack: Vec<(
        PartialLatinSquareTriple<N>,
        TripleConstraints<N>,
        CellOrValueTriple,
        usize,
    )>,
}
pub type PartialLatinSquareTriple<const N: usize> = (
    PartialLatinSquare<N>,
    PartialLatinSquare<N>,
    PartialLatinSquare<N>,
);

impl<const N: usize> LatinSquareTripleGenerator<N> {
    pub fn new() -> Self {
        let mut sq_triple = (
            PartialLatinSquare::new(),
            PartialLatinSquare::new(),
            PartialLatinSquare::new(),
        );
        let mut constraints = TripleConstraints::new();

        for i in 0..N {
            let value = constraints
                .values_for_cell(Cell(0, i))
                .into_iter()
                .next()
                .unwrap();
            constraints.set(Cell(0, i), value);
            sq_triple.0.set(0, i, value.0);
            sq_triple.1.set(0, i, value.1);
            sq_triple.2.set(0, i, value.2);
        }

        LatinSquareTripleGenerator {
            stack: vec![(
                sq_triple,
                constraints.clone(),
                constraints.most_constrained().unwrap(),
                0,
            )],
        }
    }
}

impl<const N: usize> Iterator for LatinSquareTripleGenerator<N> {
    type Item = LatinSquareTriple<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        let mut best = 0;

        'w: while let Some((sq_triple, constraints, cell_or_value, start_value)) =
            self.stack.last_mut()
        {
            match cell_or_value {
                CellOrValueTriple::Cell(cell) => {
                    let cell = *cell;
                    let values = constraints.values_for_cell(cell);

                    for value in values {
                        if value.to_index::<N>() < (*start_value).into() {
                            continue;
                        }
                        *start_value = value.to_index::<N>() + 1;

                        let mut new = constraints.clone();
                        let mut sq_triple = sq_triple.clone();
                        new.set(cell, value);
                        sq_triple.0.set(cell.0, cell.1, value.0);
                        sq_triple.1.set(cell.0, cell.1, value.1);
                        sq_triple.2.set(cell.0, cell.1, value.2);

                        if !new.is_solvable() {
                            continue;
                        }

                        match new.most_constrained() {
                            Some(cell_or_values) => {
                                self.stack.push((sq_triple.clone(), new, cell_or_values, 0));
                                if self.stack.len() > best {
                                    best = self.stack.len();
                                    dbg!(&sq_triple, best);
                                }
                                // dbg!(self.stack.len());
                                continue 'w;
                            }
                            None => {
                                if sq_triple.0.next_unknown().is_none() {
                                    return Some((
                                        sq_triple.0.into(),
                                        sq_triple.1.into(),
                                        sq_triple.2.into(),
                                    ));
                                }
                            }
                        }
                    }
                }
                CellOrValueTriple::ValueTriple(value) => {
                    let start_i = *start_value % N;

                    for i in start_i..N {
                        let mut value = *value;
                        if value.0 == N {
                            value.0 = i;
                        } else if value.1 == N {
                            value.1 = i;
                        } else {
                            value.2 = i;
                        }

                        let cells = constraints.cells_for_value(value);

                        let start_cell = *start_value / N;

                        for cell in cells {
                            if cell.to_index::<N>() < start_cell {
                                continue;
                            }
                            *start_value = i + (cell.to_index::<N>() + 1) * N;

                            let mut new = constraints.clone();
                            let mut sq_triple = sq_triple.clone();
                            new.set(cell, value);
                            sq_triple.0.set(cell.0, cell.1, value.0);
                            sq_triple.1.set(cell.0, cell.1, value.1);
                            sq_triple.2.set(cell.0, cell.1, value.2);

                            if !new.is_solvable() {
                                continue;
                            }

                            match new.most_constrained() {
                                Some(cell_or_values) => {
                                    self.stack.push((sq_triple.clone(), new, cell_or_values, 0));
                                    if self.stack.len() >= best {
                                        best = self.stack.len();
                                        dbg!(&sq_triple, best);
                                    }
                                    // dbg!(self.stack.len());
                                    continue 'w;
                                }
                                None => {
                                    if sq_triple.0.next_unknown().is_none() {
                                        return Some((
                                            sq_triple.0.into(),
                                            sq_triple.1.into(),
                                            sq_triple.2.into(),
                                        ));
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
