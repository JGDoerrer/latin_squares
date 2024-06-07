use crate::{constraints::Constraints, partial_latin_square::PartialLatinSquare};

#[derive(Debug)]
pub struct OrderlySqGenerator<const N: usize> {
    diagonal_symmetry: bool,
    k: usize,
    prev_gen: Option<Box<OrderlySqGenerator<N>>>,
    current_sq: Option<PartialLatinSquare<N>>,
    stack: Vec<StackEntry<N>>,
}

#[derive(Debug)]
struct StackEntry<const N: usize> {
    sq: PartialLatinSquare<N>,
    index: usize,
}

impl<const N: usize> OrderlySqGenerator<N> {
    pub fn new() -> Self {
        Self::new_k(N, false)
    }

    pub fn new_diagonal_symmetry() -> Self {
        Self::new_k(N, true)
    }

    fn new_k(k: usize, diagonal_symmetry: bool) -> Self {
        if k == 1 {
            let mut current_sq = PartialLatinSquare::new();
            // for i in 0..N {
            //     current_sq.set(i, 0, Some(i));
            //     current_sq.set(0, i, Some(i));
            // }
            current_sq.set(0, 0, Some(0));
            OrderlySqGenerator {
                k,
                diagonal_symmetry,
                current_sq: Some(current_sq),
                prev_gen: None,
                stack: Vec::new(),
            }
        } else {
            let mut prev_gen = Self::new_k(k - 1, diagonal_symmetry);
            let current_sq = prev_gen.next();
            OrderlySqGenerator {
                k,
                current_sq,
                diagonal_symmetry,
                prev_gen: Some(Box::new(prev_gen)),
                stack: vec![StackEntry {
                    sq: current_sq.unwrap(),
                    index: 0,
                }],
            }
        }
    }

    fn next_sq(&mut self) {
        self.current_sq = self.prev_gen.as_mut().map(|g| g.next()).flatten();
        self.stack.clear();
        if let Some(current_sq) = self.current_sq {
            self.stack.push(StackEntry {
                sq: current_sq,
                index: 0,
            });
        }
    }

    fn next_cell(k: usize, (row, col): (usize, usize)) -> Option<(usize, usize)> {
        if row < k - 1 {
            if row == k - 1 {
                Some((row, 0))
            } else {
                Some((row + 1, col))
            }
        } else {
            if col == k - 1 {
                Some((row, 0))
            } else if col != k - 2 {
                Some((row, col + 1))
            } else {
                None
            }
        }
    }
}

impl<const N: usize> Iterator for OrderlySqGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.k == 1 {
            return self.current_sq.take();
        }

        if self.k == N {
            'l: while let Some(current_sq) = self.current_sq.take() {
                let mut constraints = Constraints::new_partial(&current_sq);
                if !constraints.is_solvable() {
                    self.next_sq();
                    continue 'l;
                }

                constraints.find_singles();

                if constraints.is_solved() {
                    let partial: PartialLatinSquare<N> =
                        constraints.clone().to_latin_square().into();

                    if self.diagonal_symmetry {
                        if !partial.is_minimal_diagonal(self.k) {
                            continue;
                        }
                    } else {
                        if !partial.is_minimal_subsquare(self.k) {
                            continue;
                        }
                    }

                    let sq = constraints.to_latin_square();

                    self.next_sq();
                    return Some(sq.into());
                } else {
                    self.next_sq();
                    continue 'l;
                }
            }
            return None;
        }

        while self.current_sq.is_some() {
            'l: while !self.stack.is_empty() {
                let stack_index = self.stack.len() - 1;
                let StackEntry { sq, index } = self.stack.last_mut().unwrap();

                let constraints = Constraints::new_partial(sq);
                if !constraints.is_solvable() {
                    self.stack.pop();
                    continue;
                }

                let cell = if self.diagonal_symmetry || stack_index < self.k - 1 {
                    (stack_index, self.k - 1)
                } else {
                    (self.k - 1, stack_index + 1 - self.k)
                };

                let mut next_sq = sq.clone();

                let values = constraints.get(cell.0, cell.1);
                let value = if cell.0 == 0 || cell.1 == 0 {
                    (*index == 0).then(|| if cell.0 == 0 { cell.1 } else { cell.0 })
                } else {
                    values.into_iter().nth(*index)
                };
                *index += 1;

                let Some(value) = value else {
                    self.stack.pop();
                    continue;
                };

                next_sq.set(cell.0, cell.1, Some(value));

                if self.diagonal_symmetry {
                    next_sq.set(cell.1, cell.0, Some(value));
                }

                let max_index = if self.diagonal_symmetry {
                    self.k - 1
                } else {
                    self.k * 2 - 2
                };

                if stack_index == max_index {
                    if 2 * self.k >= N {
                        for i in 0..N {
                            if next_sq.count_val(i) < 2 * self.k - N {
                                continue 'l;
                            }
                        }
                    }

                    if !next_sq.is_minimal_subsquare(self.k) {
                        continue;
                    }

                    return Some(next_sq);
                };

                self.stack.push(StackEntry {
                    sq: next_sq,
                    index: 0,
                });
            }
            self.next_sq();
        }

        None
    }
}
