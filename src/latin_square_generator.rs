use crate::{
    constraints::{Constraints, ConstraintsDyn},
    latin_square::LatinSquare,
    latin_square_dyn::LatinSquareDyn,
    partial_latin_square::PartialLatinSquare,
    partial_latin_square_dyn::PartialLatinSquareDyn,
};

pub struct LatinSquareGenerator<const N: usize> {
    stack: Vec<(Constraints<N>, usize, usize, usize)>,
}

impl<const N: usize> LatinSquareGenerator<N> {
    pub fn new() -> Self {
        LatinSquareGenerator {
            stack: vec![(Constraints::new_reduced(), 1, 1, 0)],
        }
    }

    pub fn from_partial_sq(sq: &PartialLatinSquare<N>) -> Self {
        let constraints = Constraints::<N>::new_partial(sq);
        let index = constraints.first_empty().unwrap_or((0, 0));
        LatinSquareGenerator {
            stack: vec![(constraints, index.0, index.1, 0)],
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

            let values = constraints.get_possibilities(i, j);

            for value in values {
                if value < (*start_value) {
                    continue;
                }
                *start_value = value + 1;

                let mut new = constraints.clone();
                new.set(i, j, value);
                new.find_singles();

                if let Some((i, j)) = new.first_empty() {
                    if new.is_solvable() {
                        self.stack.push((new, i, j, 0));
                    }
                    continue 'w;
                }
                if new.is_solved() {
                    return Some((*new.partial_sq()).try_into().unwrap());
                }
            }

            self.stack.pop();
        }

        None
    }
}

#[derive(Debug)]
pub struct LatinSquareGeneratorDyn {
    stack: Vec<(ConstraintsDyn, usize, usize, usize)>,
}

impl LatinSquareGeneratorDyn {
    pub fn new(n: usize) -> Self {
        LatinSquareGeneratorDyn {
            stack: vec![(ConstraintsDyn::new(n), 1, 1, 0)],
        }
    }

    pub fn from_partial_sq(sq: &PartialLatinSquareDyn) -> Self {
        let mut constraints = ConstraintsDyn::new_partial(sq);
        constraints.find_singles();
        let index = constraints.first_empty().unwrap_or((0, 0));
        LatinSquareGeneratorDyn {
            stack: vec![(constraints, index.0, index.1, 0)],
        }
    }
}

impl Iterator for LatinSquareGeneratorDyn {
    type Item = LatinSquareDyn;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        if self.stack.first().unwrap().0.is_solved() {
            return Some(
                self.stack
                    .pop()
                    .unwrap()
                    .0
                    .partial_sq()
                    .clone()
                    .try_into()
                    .unwrap(),
            );
        }

        'w: while let Some((constraints, i, j, start_value)) = self.stack.last_mut() {
            let (constraints, i, j) = (constraints.clone(), *i, *j);
            let values = constraints.get_possibilities(i, j);

            for value in values {
                if value < (*start_value) {
                    continue;
                }
                *start_value = value + 1;

                let mut new = constraints.clone();
                new.set(i, j, value);
                new.find_singles();

                if new.is_solved() {
                    return Some(new.partial_sq().clone().try_into().unwrap());
                }
                if let Some((i, j)) = new.first_empty() {
                    if new.is_solvable() {
                        self.stack.push((new, i, j, 0));
                    }
                    continue 'w;
                }
            }

            self.stack.pop();
        }

        None
    }
}
