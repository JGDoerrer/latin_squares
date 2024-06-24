use crate::{constraints::Constraints, latin_square::LatinSquare};

pub struct RandomLatinSquareGenerator<const N: usize> {
    random_state: [u64; 4],
}

impl<const N: usize> RandomLatinSquareGenerator<N> {
    pub fn new(seed: u64) -> Self {
        RandomLatinSquareGenerator {
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

impl<const N: usize> Iterator for RandomLatinSquareGenerator<N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut stack = vec![(Constraints::new(), 1, 1)];

        while let Some((constraints, i, j)) = stack.last() {
            let (constraints, i, j) = (constraints.clone(), *i, *j);

            let values = constraints.get(i, j);

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
                return Some(new.into());
            }

            if new.is_solvable() {
                let mut cell = (
                    self.next_random() as usize % N,
                    self.next_random() as usize % N,
                );

                while new.is_set(cell.0, cell.1) {
                    cell = (
                        self.next_random() as usize % N,
                        self.next_random() as usize % N,
                    );
                }

                stack.push((new, cell.0, cell.1));
                continue;
            }

            stack.clear();
            stack.push((Constraints::new_reduced(), 1, 1));
        }

        unreachable!()
    }
}
