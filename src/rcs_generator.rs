use crate::{
    constraints::Constraints, latin_square::LatinSquare, partial_latin_square::PartialLatinSquare,
    permutation::PermutationIter,
};

pub struct RCSGenerator<const N: usize> {
    stack: Vec<StackEntry<N>>,
}

#[derive(Debug)]
struct StackEntry<const N: usize> {
    sq: PartialLatinSquare<N>,
    value_index: usize,
}

impl<const N: usize> RCSGenerator<N> {
    pub fn new() -> Self {
        RCSGenerator {
            stack: vec![StackEntry {
                sq: PartialLatinSquare::empty(),
                value_index: 0,
            }],
        }
    }

    fn is_minimal(sq: &LatinSquare<N>) -> bool {
        for permutation in PermutationIter::new() {
            let new_sq = sq
                .permute_rows_and_cols(&permutation)
                .permute_vals(&permutation);

            if new_sq < *sq {
                return false;
            }
        }

        true
    }
}

impl<const N: usize> Iterator for RCSGenerator<N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            let StackEntry { sq, value_index } = self.stack.last_mut().unwrap();

            let cell_index = sq.first_empty_index().unwrap();
            let row = cell_index / N;
            let col = cell_index % N;

            if cell_index == 0 && *value_index > 0 {
                self.stack.pop();
                continue;
            }
            if cell_index == 1 && *value_index > 2 {
                self.stack.pop();
                continue;
            }

            let mut constraints = Constraints::new_partial(sq);

            let Some(val) = constraints
                .get_possibilities(row, col)
                .into_iter()
                .nth(*value_index)
            else {
                self.stack.pop();
                continue;
            };
            *value_index += 1;

            let mut new_sq = *sq;

            if constraints.get_possibilities(row, col).contains(val) {
                new_sq.set(row, col, Some(val));
                constraints.set(row, col, val);
            } else {
                continue;
            }

            if constraints.get_possibilities(val, row).contains(col) {
                new_sq.set(val, row, Some(col));
                constraints.set(val, row, col);
            } else if new_sq.get(val, row) != Some(col) {
                continue;
            }

            if constraints.get_possibilities(col, val).contains(row) {
                new_sq.set(col, val, Some(row));
                constraints.set(col, val, row);
            } else if new_sq.get(col, val) != Some(row) {
                continue;
            }

            if new_sq.is_complete() {
                let sq = new_sq.try_into().unwrap();
                if Self::is_minimal(&sq) {
                    return Some(sq);
                } else {
                    continue;
                }
            }

            let mut new_partial = Constraints::new_partial(&new_sq);
            new_partial.find_singles();
            if !new_partial.is_solvable() {
                continue;
            }

            // dbg!(new_sq);
            self.stack.push(StackEntry {
                sq: new_sq,
                value_index: 0,
            });
        }

        None
    }
}
