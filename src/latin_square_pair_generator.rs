use crate::{latin_square::LatinSquarePair, pair_constraints::PairConstraints, types::Value};

pub struct LatinSquarePairGenerator<const N: usize> {
    stack: Vec<(PairConstraints<N>, usize, usize, usize)>,
}

impl<const N: usize> LatinSquarePairGenerator<N> {
    pub fn new() -> Self {
        LatinSquarePairGenerator {
            stack: vec![(PairConstraints::new_first_row(), 0, 0, 0)],
        }
    }
}

impl<const N: usize> Iterator for LatinSquarePairGenerator<N> {
    type Item = LatinSquarePair<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        'w: while let Some((constraints, i, j, start_value)) = self.stack.last_mut() {
            let (constraints, i, j) = (constraints.clone(), *i, *j);

            let values = constraints.get(i, j);

            for value in values {
                if value < (*start_value).into() {
                    continue;
                }
                *start_value = value + 1;

                let value_pair = ((value % N) as Value, (value / N) as Value);

                let mut new = constraints.clone();
                new.set(i, j, value_pair);

                if !new.is_solvable() {
                    continue;
                }

                if let Some((i, j)) = new.get_next() {
                    self.stack.push((new, i, j, 0));
                    continue 'w;
                } else if new.is_solved() {
                    return Some(new.to_latin_squares());
                }
            }

            self.stack.pop();
        }

        None
    }
}
