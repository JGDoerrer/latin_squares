use crate::{constraints::ConstraintsDyn, latin_square_dyn::LatinSquareDyn};

pub struct RandomLatinSquareGeneratorDyn {
    n: usize,
    random_state: [u64; 4],
}

impl RandomLatinSquareGeneratorDyn {
    pub fn new(n: usize, seed: u64) -> Self {
        RandomLatinSquareGeneratorDyn {
            n,
            random_state: [seed, 1, 2, 3],
        }
    }

    /// https://en.wikipedia.org/wiki/Xorshift#xoshiro256**
    fn xoshiro(state: [u64; 4]) -> (u64, [u64; 4]) {
        let result = state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);

        let new_state = [
            state[0] ^ state[1] ^ state[3],
            state[0] ^ state[1] ^ state[2],
            state[2] ^ state[0] ^ (state[1] << 17),
            (state[3] ^ state[1]).rotate_left(45),
        ];

        (result, new_state)
    }

    fn next_random(&mut self) -> u64 {
        let (result, next_state) = Self::xoshiro(self.random_state);
        self.random_state = next_state;
        result
    }
}

impl Iterator for RandomLatinSquareGeneratorDyn {
    type Item = LatinSquareDyn;

    fn next(&mut self) -> Option<Self::Item> {
        let mut stack = vec![(
            ConstraintsDyn::new(self.n),
            self.next_random() as usize % self.n,
            self.next_random() as usize % self.n,
        )];

        while let Some((constraints, i, j)) = stack.last() {
            let (constraints, i, j) = (constraints.clone(), *i, *j);

            let values = constraints.get_possibilities(i, j);

            if values.is_empty() {
                dbg!(constraints, i, j);
                unreachable!()
            }

            let value = values
                .into_iter()
                .nth(self.next_random() as usize % values.len())
                .unwrap();

            let mut new = constraints.clone();
            new.set(i, j, value);
            new.find_singles();

            if new.is_solved() {
                return Some(new.partial_sq().clone().try_into().unwrap());
            }

            if new.is_solvable() {
                let mut cell = (
                    self.next_random() as usize % self.n,
                    self.next_random() as usize % self.n,
                );

                while new.is_set(cell.0, cell.1) {
                    cell = (
                        self.next_random() as usize % self.n,
                        self.next_random() as usize % self.n,
                    );
                }

                stack.push((new, cell.0, cell.1));
                continue;
            }

            stack.clear();
            stack.push((
                ConstraintsDyn::new(self.n),
                self.next_random() as usize % self.n,
                self.next_random() as usize % self.n,
            ));
        }

        unreachable!()
    }
}
