use crate::{
    latin_square::{
        Cell, CellOrValuePair, LatinSquarePair, PartialLatinSquare, PartialLatinSquarePair,
        ValuePair,
    },
    pair_constraints::PairConstraints,
};

pub struct LatinSquarePairGenerator<const N: usize> {
    stack: Vec<(
        PartialLatinSquarePair<N>,
        PairConstraints<N>,
        CellOrValuePair,
        usize,
    )>,
}

impl<const N: usize> LatinSquarePairGenerator<N> {
    pub fn new() -> Self {
        let mut sq_pair = (PartialLatinSquare::new(), PartialLatinSquare::new());
        let mut constraints = PairConstraints::new();

        for i in 0..N {
            let value = constraints
                .values_for_cell(0, i)
                .into_iter()
                .next()
                .unwrap();
            let value_pair = ((value % N) as usize, (value / N) as usize);
            constraints.set(0, i, value_pair);
            sq_pair.0.set(0, i, value_pair.0);
            sq_pair.1.set(0, i, value_pair.1);
        }

        // for j in 1..N {
        //     let value = constraints
        //         .values_for_cell(j, 0)
        //         .into_iter()
        //         .next()
        //         .unwrap();
        //     let value_pair = ((value % N) as usize, (value / N) as usize);
        //     constraints.set(j, 0, value_pair);
        //     sq_pair.0.set(j, 0, value_pair.0);
        //     sq_pair.1.set(j, 0, value_pair.1);
        // }

        LatinSquarePairGenerator {
            stack: vec![(
                sq_pair,
                constraints.clone(),
                constraints.most_constrained().unwrap(),
                0,
            )],
        }
    }
}

impl<const N: usize> Iterator for LatinSquarePairGenerator<N> {
    type Item = LatinSquarePair<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        'w: while let Some((sq_pair, constraints, cell_or_value, start_value)) =
            self.stack.last_mut()
        {
            // dbg!(&sq_pair);
            match *cell_or_value {
                CellOrValuePair::Cell(Cell(i, j)) => {
                    let values = constraints.values_for_cell(i, j);

                    for value in values {
                        if value < (*start_value).into() {
                            continue;
                        }
                        *start_value = value + 1;

                        let value_pair = ((value % N) as usize, (value / N) as usize);

                        let mut new = constraints.clone();
                        let mut sq_pair = sq_pair.clone();
                        new.set(i, j, value_pair);
                        sq_pair.0.set(i, j, value_pair.0);
                        sq_pair.1.set(i, j, value_pair.1);

                        if !new.is_solvable() {
                            continue;
                        }

                        match new.most_constrained() {
                            Some(cell_or_value) => {
                                self.stack.push((sq_pair, new, cell_or_value, 0));
                                continue 'w;
                            }
                            None => {
                                if sq_pair.0.next_unknown().is_none() {
                                    return Some((sq_pair.0.into(), sq_pair.1.into()));
                                }
                            }
                        }
                    }
                }
                CellOrValuePair::ValuePair(value_pair) => {
                    let cells = constraints.cells_for_value((value_pair.0, value_pair.1));

                    for value in cells {
                        if value < (*start_value).into() {
                            continue;
                        }
                        *start_value = value + 1;

                        let cell = (value / N, value % N);

                        let mut new = constraints.clone();
                        let mut sq_pair = sq_pair.clone();
                        new.set(cell.0, cell.1, (value_pair.0, value_pair.1));
                        sq_pair.0.set(cell.0, cell.1, value_pair.0);
                        sq_pair.1.set(cell.0, cell.1, value_pair.1);

                        if !new.is_solvable() {
                            continue;
                        }

                        match new.most_constrained() {
                            Some(cell_or_value) => {
                                self.stack.push((sq_pair, new, cell_or_value, 0));
                                continue 'w;
                            }
                            None => {
                                if sq_pair.0.next_unknown().is_none() {
                                    return Some((sq_pair.0.into(), sq_pair.1.into()));
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
