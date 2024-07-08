use std::cmp::Ordering;

use crate::{
    bitset::BitSet16,
    constraints::Constraints,
    partial_latin_square::PartialLatinSquare,
    permutation::{Permutation, PermutationDynIter},
};

#[derive(Debug)]
pub struct RCGenerator<const N: usize> {
    k: usize,
    prev_gen: Option<Box<RCGenerator<N>>>,
    current_sq: Option<PartialLatinSquare<N>>,
    stack: Vec<StackEntry<N>>,
    permutation: Permutation<N>,
    last_deny_permutations: Vec<(Permutation<N>, Permutation<N>)>,
}

#[derive(Debug)]
struct StackEntry<const N: usize> {
    sq: PartialLatinSquare<N>,
    index: usize,
}

impl<const N: usize> RCGenerator<N> {
    pub fn new(permutation: Permutation<N>) -> Self {
        Self::new_k(N, permutation)
    }

    fn new_k(k: usize, permutation: Permutation<N>) -> Self {
        if k == 0 {
            let current_sq = PartialLatinSquare::empty();
            // for i in 0..N {
            //     current_sq.set(i, 0, Some(i));
            //     current_sq.set(0, i, Some(i));
            // }
            // current_sq.set(0, 0, Some(0));
            RCGenerator {
                k,
                permutation,
                current_sq: Some(current_sq),
                prev_gen: None,
                stack: Vec::new(),
                last_deny_permutations: Vec::new(),
            }
        } else {
            let mut prev_gen = Self::new_k(k - 1, permutation.clone());
            let current_sq = prev_gen.next();
            let stack = if let Some(sq) = current_sq {
                vec![StackEntry { sq, index: 0 }]
            } else {
                Vec::new()
            };
            RCGenerator {
                k,
                permutation,
                current_sq,
                prev_gen: Some(Box::new(prev_gen)),
                stack,
                last_deny_permutations: Vec::new(),
            }
        }
    }

    fn next_sq(&mut self) {
        self.current_sq = self.prev_gen.as_mut().and_then(|g| g.next());
        self.stack.clear();
        if let Some(current_sq) = self.current_sq {
            self.stack.push(StackEntry {
                sq: current_sq,
                index: 0,
            });
        }
    }

    fn is_minimal_diagonal(&mut self, sq: PartialLatinSquare<N>) -> bool {
        let unique_entries = sq.unique_entries();

        if unique_entries.into_iter().last().unwrap() != unique_entries.len() - 1 {
            return false;
        }

        let k = self.k;
        let permutation = &self.permutation;

        for i in 0..k - 1 {
            if sq.get(i, i).unwrap() > sq.get(i + 1, i + 1).unwrap() {
                return false;
            }
        }

        if sq.cmp_diagonal(&sq.transposed()).is_gt() {
            return false;
        }

        for (val_permutation, row_permutation) in &self.last_deny_permutations {
            let permuted_sq = sq
                .permute_vals(val_permutation)
                .permute_rows_and_cols(row_permutation);

            'l: for i in 0..N {
                for j in (0..=i).rev() {
                    match sq.get(i, j).cmp(&permuted_sq.get(i, j)) {
                        Ordering::Greater => return false,
                        Ordering::Less => break 'l,
                        Ordering::Equal => {}
                    }
                }
            }
        }

        for val_permutation in PermutationDynIter::new(unique_entries.len()) {
            let val_permutation: Permutation<N> = val_permutation.pad_with_id();
            if permutation.conjugate_by(&val_permutation) != *permutation {
                continue;
            }

            let permuted_sq = sq.permute_vals(&val_permutation);

            'r: for row_permutation in PermutationDynIter::new(k) {
                if row_permutation.as_vec()[k - 1] == k - 1 {
                    continue;
                }

                let row_permutation = row_permutation.pad_with_id();
                let permuted_sq = permuted_sq.permute_rows_and_cols(&row_permutation);

                for i in 0..N {
                    for j in (0..=i).rev() {
                        match sq.get(i, j).cmp(&permuted_sq.get(i, j)) {
                            Ordering::Greater => {
                                self.last_deny_permutations
                                    .push((val_permutation, row_permutation.clone()));
                                return false;
                            }
                            Ordering::Less => continue 'r,
                            Ordering::Equal => {}
                        }
                    }
                }
            }
        }

        true
    }
}

impl<const N: usize> Iterator for RCGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let fixed_points: BitSet16 = self.permutation.fixed_points().collect();
        let num_fixed_points = self.permutation.num_fixed_points();

        if self.k == 0 {
            return self.current_sq.take();
        }

        if self.k == N {
            while let Some(current_sq) = self.current_sq.take() {
                let mut constraints = Constraints::new_partial(&current_sq);
                if !constraints.is_solvable() {
                    self.next_sq();
                    continue;
                }

                constraints.find_singles();

                if constraints.is_solved() {
                    let sq = constraints.to_latin_square();
                    let partial: PartialLatinSquare<N> = sq.into();

                    if !self.is_minimal_diagonal(partial) {
                        self.next_sq();
                        continue;
                    }

                    self.next_sq();
                    return Some(sq.into());
                } else {
                    self.next_sq();
                    continue;
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

                let cell = if stack_index < self.k - 1 {
                    (stack_index, self.k - 1)
                } else {
                    (self.k - 1, stack_index + 1 - self.k)
                };

                let mut next_sq = *sq;

                let values = if cell.0 == cell.1 {
                    fixed_points.intersect(constraints.get(cell.0, cell.1))
                } else if constraints.is_set(cell.1, cell.0) {
                    let value = constraints.partial_sq().get(cell.1, cell.0).unwrap();
                    [value, self.permutation.apply(value)]
                        .into_iter()
                        .collect::<BitSet16>()
                        .intersect(constraints.get(cell.0, cell.1))
                } else {
                    constraints.get(cell.0, cell.1)
                };

                let value = values.into_iter().nth(*index);
                *index += 1;

                let Some(value) = value else {
                    self.stack.pop();
                    continue;
                };

                next_sq.set(cell.0, cell.1, Some(value));

                let max_index = self.k * 2 - 2;

                if stack_index == max_index {
                    if 2 * self.k >= N + 1 {
                        for i in 0..N {
                            if next_sq.count_val(i) < 2 * self.k - N {
                                continue 'l;
                            }
                        }
                    }

                    let mut same_parity = 0;
                    for i in fixed_points {
                        if next_sq.count_val(i) % 2 == N % 2 {
                            same_parity += 1;
                        }
                    }

                    if same_parity + N < self.k + num_fixed_points {
                        continue;
                    }

                    if !self.is_minimal_diagonal(next_sq) {
                        continue;
                    }

                    dbg!(next_sq);

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
