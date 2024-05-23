use crate::latin_square::{Cell, LatinSquare, PartialLatinSquare};

pub struct PartialSquareGenerator<const N: usize> {
    sq: LatinSquare<N>,
    partial_sq: Option<PartialLatinSquare<N>>,
    num_entries: usize,
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
            num_entries,
            index: gen.as_ref().map_or(0, |gen| gen.index),
            gen,
            partial_sq,
            sq,
        }
    }
}

impl<const N: usize> Iterator for PartialSquareGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_entries == 0 {
            let value = self.partial_sq.take();
            return value;
        }

        while self.index >= N * N {
            let gen = self.gen.as_mut().unwrap();

            let Some(sq) = gen.next() else {
                return None;
            };
            self.partial_sq = Some(sq);

            self.index = gen.index + 1;
        }

        let mut sq = self.partial_sq.unwrap();

        let Cell(i, j) = Cell::from_index::<N>(self.index);
        sq.set(i, j, Some(self.sq.get(i, j)));

        self.index += 1;

        while sq.num_full_cols() > 0 || sq.num_full_rows() > 0 {
            sq = self.partial_sq.unwrap();
            let Cell(i, j) = Cell::from_index::<N>(self.index);
            sq.set(i, j, Some(self.sq.get(i, j)));

            self.index += 1;
            if self.index >= N * N {
                todo!()
            }
        }

        assert!(
            sq.num_entries() == self.num_entries,
            "{sq:?}, {}",
            self.num_entries
        );

        Some(sq)
    }
}
