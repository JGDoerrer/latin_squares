use crate::{constraints::Constraints, latin_square::LatinSquare};

pub struct LatinSquareGenerator<const N: usize> {
    stack: Vec<(Constraints<N>, usize, usize, usize)>,
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

            let values = constraints.get(i, j);

            for value in values {
                if value < (*start_value).into() {
                    continue;
                }
                *start_value = value as usize + 1;

                let mut new = constraints.clone();
                new.set(i, j, value);
                new.find_singles();

                if let Some((i, j)) = new.get_next() {
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
