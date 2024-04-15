use crate::{
    constraints::Constraints,
    latin_square::{Cell, LatinSquare},
};

pub struct OrthogonalLatinSquareGenerator<const N: usize> {
    stack: Vec<(Constraints<N>, usize, usize, usize)>,
    sqs: Vec<LatinSquare<N>>,
}

impl<const N: usize> OrthogonalLatinSquareGenerator<N> {
    pub fn new(sqs: Vec<LatinSquare<N>>) -> Self {
        let mut constraints = Constraints::new_first_row();
        for sq in &sqs {
            constraints.make_orthogonal_to_sq(sq)
        }

        OrthogonalLatinSquareGenerator {
            sqs,
            stack: vec![(constraints, 0, 0, 0)],
        }
    }
}

impl<const N: usize> Iterator for OrthogonalLatinSquareGenerator<N> {
    type Item = LatinSquare<N>;

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
                *start_value = value as usize + 1;

                let mut new = constraints.clone();
                new.set(i, j, value);
                for sq in &self.sqs {
                    new.make_orthogonal_to_sq(sq)
                }
                new.find_singles();

                if !new.is_solvable() {
                    continue;
                }

                if let Some(Cell(i, j)) = new.most_constrained_cell() {
                    self.stack.push((new, i, j, 0));
                    continue 'w;
                }
                if new.is_solved() {
                    let new_sq = new.into();

                    if self.sqs.iter().all(|sq| sq.is_orthogonal_to(&new_sq)) {
                        return Some(new_sq);
                    }
                    // dbg!(new_sq, self.stack.len());
                    continue 'w;
                }
            }

            self.stack.pop();
        }

        None
    }
}
