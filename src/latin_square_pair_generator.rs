use crate::{
    bitset::BitSet128,
    latin_square::{Cell, LatinSquarePair, PartialLatinSquare},
    pair_constraints::{CellOrValuePair, PairConstraints, ValuePair},
};

pub struct LatinSquarePairGenerator<const N: usize> {
    stack: Vec<(PairConstraints<N>, CellOrValuePair, usize)>,
}

pub type PartialLatinSquarePair<const N: usize> = (PartialLatinSquare<N>, PartialLatinSquare<N>);

impl<const N: usize> LatinSquarePairGenerator<N> {
    pub fn new() -> Self {
        let mut constraints = PairConstraints::new();

        for i in 0..N {
            let value = constraints
                .values_for_cell(0, i)
                .into_iter()
                .next()
                .unwrap();
            let value_pair = ValuePair::from_index::<N>(value);
            constraints.set(0, i, value_pair);
        }

        // for j in 1..N {
        //     let value = constraints
        //         .values_for_cell(j, 0)
        //         .into_iter()
        //         .next()
        //         .unwrap();
        //     let value_pair = ValuePair::from_index::<N>(value);
        //     constraints.set(j, 0, value_pair);
        // }

        LatinSquarePairGenerator {
            stack: vec![(
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

        let mut best = 0;

        'w: while let Some((constraints, cell_or_value, start_value)) = self.stack.last_mut() {
            // dbg!(&sq_pair);
            match *cell_or_value {
                CellOrValuePair::Cell(Cell(i, j)) => {
                    let values = constraints.values_for_cell(i, j);

                    for value in
                        values.intersect(BitSet128::all_less_than(*start_value).complement())
                    {
                        *start_value = value + 1;

                        let value_pair = ValuePair::from_index::<N>(value);

                        let mut new = constraints.clone();
                        new.set(i, j, value_pair);
                        new.find_and_set_singles();

                        if !new.is_solvable() {
                            continue;
                        }

                        match new.most_constrained() {
                            Some(cell_or_value) => {
                                self.stack.push((new.clone(), cell_or_value, 0));

                                if self.stack.len() >= best {
                                    best = self.stack.len();
                                    dbg!(new.sq_pair(), best);
                                }

                                continue 'w;
                            }
                            None => return Some((new.sq_pair().0.into(), new.sq_pair().1.into())),
                        }
                    }
                }
                CellOrValuePair::ValuePair(value_pair) => {
                    let cells = constraints.cells_for_value(value_pair);

                    for value in
                        cells.intersect(BitSet128::all_less_than(*start_value).complement())
                    {
                        *start_value = value + 1;

                        let cell = (value / N, value % N);

                        let mut new = constraints.clone();
                        new.set(cell.0, cell.1, value_pair);
                        new.find_and_set_singles();

                        if !new.is_solvable() {
                            continue;
                        }

                        match new.most_constrained() {
                            Some(cell_or_value) => {
                                self.stack.push((new.clone(), cell_or_value, 0));

                                if self.stack.len() >= best {
                                    best = self.stack.len();
                                    dbg!(new.sq_pair(), best);
                                }

                                continue 'w;
                            }
                            None => return Some((new.sq_pair().0.into(), new.sq_pair().1.into())),
                        }
                    }
                }
            }

            self.stack.pop();
        }

        None
    }
}
