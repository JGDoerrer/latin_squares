use crate::{
    bitset::BitSet,
    constraints::{self, Constraints},
    latin_square::LatinSquare,
    types::Value,
};

pub struct LatinSquareGenerator {
    stack: Vec<(Constraints, usize, usize, Value)>,
}

impl LatinSquareGenerator {
    pub fn new(n: usize) -> Self {
        LatinSquareGenerator {
            stack: vec![(Constraints::new_reduced(n), 1, 1, 0)],
        }
    }
}

impl Iterator for LatinSquareGenerator {
    type Item = LatinSquare;

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

pub struct OrthogonalGenerator {
    sq: LatinSquare,
    stack: Vec<(Constraints, usize, usize, Value)>,
}

impl OrthogonalGenerator {
    pub fn new(sq: LatinSquare) -> Self {
        let mut constraints = Constraints::new_first_row(sq.n());
        constraints.make_orthogonal_to(&sq);

        OrthogonalGenerator {
            stack: vec![(constraints, 1, 1, 0)],
            sq,
        }
    }
}

impl Iterator for OrthogonalGenerator {
    type Item = LatinSquare;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        'w: while let Some((constraints, i, j, start_value)) = self.stack.last_mut() {
            let (constraints, i, j) = (constraints.clone(), *i, *j);

            let values = constraints.get(i, j);

            for value in values.intersect(BitSet::all_less_than(*start_value as usize).complement())
            {
                *start_value = value as Value + 1;

                let mut new = constraints.clone();
                new.set(i, j, value as Value);
                new.make_orthogonal_to(&self.sq);

                if let Some((i, j)) = new.first_unsolved() {
                    if new.is_solvable() && new.is_orthogonal_to(&self.sq) {
                        self.stack.push((new, i, j, 0));
                    }
                    continue 'w;
                }
                if new.is_solved() {
                    let sq = new.into();
                    if self.sq.is_orthogonal_to(&sq) {
                        return Some(sq);
                    }
                }
            }

            self.stack.pop();
        }

        None
    }
}
