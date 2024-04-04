use crate::{
    bitset::BitSet,
    constraints::{self, Constraints},
    latin_square::LatinSquare,
    types::Value,
};

pub struct LatinSquareGenerator<const N: usize> {
    stack: Vec<(Constraints<N>, usize, usize, Value)>,
}

impl<const N: usize> LatinSquareGenerator<N> {
    pub fn new() -> Self {
        LatinSquareGenerator {
            stack: vec![(Constraints::new_reduced(), 1, 1, 0)],
        }
    }
}

impl<const N: usize> Iterator for LatinSquareGenerator<N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        'w: while let Some((constraints, i, j, start_value)) = self.stack.last_mut() {
            let (constraints, i, j) = (constraints.clone(), *i, *j);

            let values = constraints.get(i, j).bitset();

            for value in values {
                if value < (*start_value).into() {
                    continue;
                }
                *start_value = value as Value + 1;

                let mut new = constraints.clone();
                new.set(i, j, value as Value);

                if let Some((i, j)) = new.first_unsolved() {
                    if new.is_solvable() {
                        self.stack.push((new, i, j, 0));
                    }
                    continue 'w;
                }
                if new.is_solved() {
                    return Some(new.into());
                }
            }

            self.stack.pop();
        }

        None
    }
}

#[derive(Debug)]
enum State<const N: usize> {
    First {
        constraints: Constraints<N>,
        constraints2: Constraints<N>,
        constraints3: Constraints<N>,
        i: usize,
        j: usize,
        start_value: Value,
    },
    Second {
        sq: LatinSquare<N>,
        constraints: Constraints<N>,
        constraints2: Constraints<N>,
        i: usize,
        j: usize,
        start_value: Value,
    },
    Third {
        sq: LatinSquare<N>,
        sq2: LatinSquare<N>,
        constraints: Constraints<N>,
        i: usize,
        j: usize,
        start_value: Value,
    },
}
pub struct OrthogonalGenerator<const N: usize> {
    stack: Vec<State<N>>,
}

impl<const N: usize> OrthogonalGenerator<N> {
    pub fn new() -> Self {
        OrthogonalGenerator {
            stack: vec![State::First {
                constraints: Constraints::new_reduced(),
                constraints2: Constraints::new_first_row(),
                constraints3: Constraints::new_first_row(),
                i: 1,
                j: 0,
                start_value: 0,
            }],
        }
    }
}

impl<const N: usize> Iterator for OrthogonalGenerator<N> {
    type Item = (LatinSquare<N>, LatinSquare<N>, LatinSquare<N>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        'w: while let Some(state) = self.stack.last_mut() {
            match state {
                State::First {
                    constraints,
                    constraints2,
                    constraints3,
                    i,
                    j,
                    start_value,
                } => {
                    let values = constraints.get(*i, *j).bitset();

                    for value in
                        values.intersect(BitSet::all_less_than(*start_value as usize).complement())
                    {
                        *start_value = value as Value + 1;

                        let mut new = constraints.clone();
                        let new2 = constraints2.clone();
                        let new3 = constraints3.clone();

                        new.set(*i, *j, value as Value);
                        new.find_singles();

                        if let Some((i, j)) = new.first_unsolved() {
                            if new.is_solvable() && new2.is_solvable() {
                                self.stack.push(State::First {
                                    constraints: new,
                                    constraints2: new2,
                                    constraints3: new3,
                                    i,
                                    j,
                                    start_value: 0,
                                });
                            }
                            continue 'w;
                        } else if new.is_solved() {
                            let sq = new.into();

                            self.stack.push(State::Second {
                                sq,
                                constraints: new2.clone(),
                                constraints2: new3.clone(),
                                i: 1,
                                j: 0,
                                start_value: 0,
                            });
                            continue 'w;
                        }
                    }

                    self.stack.pop();
                }
                State::Second {
                    sq,
                    constraints,
                    constraints2,
                    i,
                    j,
                    start_value,
                } => {
                    let values = constraints.get(*i, *j).bitset();

                    for value in
                        values.intersect(BitSet::all_less_than(*start_value as usize).complement())
                    {
                        *start_value = value as Value + 1;

                        let mut new = constraints.clone();
                        let new2 = constraints2.clone();

                        new.set(*i, *j, value as Value);
                        new.make_orthogonal_to_sq(&sq);
                        new.find_singles();
                        // new2.make_orthogonal_to_sq(&sq);

                        if let Some((i, j)) = new.first_unsolved() {
                            if new.is_solvable() && new2.is_solvable() {
                                let sq = sq.clone();
                                self.stack.push(State::Second {
                                    sq,
                                    constraints: new,
                                    constraints2: new2,
                                    i,
                                    j,
                                    start_value: 0,
                                });
                            }
                            continue 'w;
                        } else if new.is_solved() {
                            let sq = sq.clone();
                            let sq2 = new.into();

                            if !sq.is_orthogonal_to(&sq2) {
                                continue;
                            }

                            dbg!((&sq, &sq2));
                            self.stack.push(State::Third {
                                sq,
                                sq2,
                                constraints: new2.clone(),
                                i: 1,
                                j: 0,
                                start_value: 0,
                            });
                            continue 'w;
                        }
                    }

                    self.stack.pop();
                }
                State::Third {
                    sq,
                    sq2,
                    constraints,
                    i,
                    j,
                    start_value,
                } => {
                    let values = constraints.get(*i, *j).bitset();

                    for value in
                        values.intersect(BitSet::all_less_than(*start_value as usize).complement())
                    {
                        *start_value = value as Value + 1;

                        let mut new = constraints.clone();

                        new.set(*i, *j, value as Value);
                        new.make_orthogonal_to_sq(&sq);
                        new.make_orthogonal_to_sq(&sq2);
                        new.find_singles();

                        if let Some((i, j)) = new.first_unsolved() {
                            if new.is_solvable() {
                                let sq = sq.clone();
                                let sq2 = sq2.clone();
                                self.stack.push(State::Third {
                                    sq,
                                    sq2,
                                    constraints: new,
                                    i,
                                    j,
                                    start_value: 0,
                                });
                            }
                            continue 'w;
                        } else if new.is_solved() {
                            let sq3 = new.into();

                            if !sq2.is_orthogonal_to(&sq3) || !sq.is_orthogonal_to(&sq3) {
                                continue;
                            }

                            return Some((sq.clone(), sq2.clone(), sq3));
                        }
                    }

                    self.stack.pop();
                }
            }
        }

        None
    }
}
