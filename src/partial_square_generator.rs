use crate::latin_square::{Cell, LatinSquare, PartialLatinSquare};

#[derive(Debug)]
pub struct PartialSquareGenerator<const N: usize> {
    sq: LatinSquare<N>,
    partial_sq: Option<PartialLatinSquare<N>>,
    entries_left: usize,
    index: usize,
    gen: Option<Box<PartialSquareGenerator<N>>>,
}

impl<const N: usize> PartialSquareGenerator<N> {
    pub fn new(sq: LatinSquare<N>, num_entries: usize) -> Self {
        let mut gen =
            (num_entries != 0).then(|| Box::new(PartialSquareGenerator::new(sq, num_entries - 1)));

        let partial_sq = if num_entries == 0 {
            Some(PartialLatinSquare::new())
        } else {
            gen.as_mut().unwrap().next()
        };

        PartialSquareGenerator {
            entries_left: num_entries,
            index: gen.as_ref().map_or(0, |gen| gen.index),
            gen,
            partial_sq,
            sq,
        }
    }

    pub fn new_partial(
        sq: LatinSquare<N>,
        partial: PartialLatinSquare<N>,
        num_entries: usize,
    ) -> Self {
        let current_entries = partial.num_entries();
        let entries_left = num_entries - current_entries;

        let mut gen = (entries_left != 0).then(|| {
            Box::new(PartialSquareGenerator::new_partial(
                sq,
                partial,
                num_entries - 1,
            ))
        });

        let partial_sq = if entries_left == 0 {
            Some(partial)
        } else {
            gen.as_mut().unwrap().next()
        };

        PartialSquareGenerator {
            entries_left,
            index: gen
                .as_ref()
                .map_or(partial.first_empty_index().unwrap(), |gen| {
                    partial.next_empty_index(gen.index).unwrap_or(N * N)
                }),
            gen,
            partial_sq,
            sq,
        }
    }
}

impl<const N: usize> Iterator for PartialSquareGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries_left == 0 {
            let value = self.partial_sq.take();
            return value;
        }

        while self.index >= N * N
        // || self.partial_sq.is_none()
        // || self
        //     .partial_sq
        //     .is_some_and(|sq| sq.num_next_empty_indices(self.index) < self.entries_left)
        {
            let gen = self.gen.as_mut().unwrap();

            let Some(sq) = gen.next() else {
                return None;
            };
            self.partial_sq = Some(sq);

            self.index = self
                .partial_sq
                .as_ref()
                .unwrap()
                .next_empty_index(gen.index + 1)
                .unwrap_or(N * N);
        }

        let mut sq = self.partial_sq.unwrap();

        let Cell(i, j) = Cell::from_index::<N>(self.index);
        sq.set(i, j, Some(self.sq.get(i, j)));

        self.index = self
            .partial_sq
            .as_ref()
            .unwrap()
            .next_empty_index(self.index + 1)
            .unwrap_or(N * N);

        // while sq.num_full_cols() > 0 || sq.num_full_rows() > 0 {
        //     sq = self.partial_sq.unwrap();
        //     let Cell(i, j) = Cell::from_index::<N>(self.index);
        //     sq.set(i, j, Some(self.sq.get(i, j)));

        //     self.index += 1;
        //     if self.index >= N * N {
        //         todo!()
        //     }
        // }

        // assert!(
        //     sq.num_entries() == self.num_entries,
        //     "{sq:?}, {}",
        //     self.num_entries
        // );

        Some(sq)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn partial_test() {
        let sq = LatinSquare::new([[0, 1, 2], [1, 2, 0], [2, 0, 1]]);
        let partial_sq =
            PartialLatinSquare::from_array([[Some(0), None, None], [None; 3], [None; 3]]);

        let mut generator = PartialSquareGenerator::new_partial(sq, partial_sq, 9);

        assert_eq!(generator.next(), Some(sq.into()));
        assert_eq!(generator.next(), None);
    }
}
