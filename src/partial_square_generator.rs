use crate::{
    latin_square::{Cell, LatinSquare},
    latin_square_dyn::LatinSquareDyn,
    latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait},
    partial_latin_square::PartialLatinSquare,
    partial_latin_square_dyn::PartialLatinSquareDyn,
};

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
            Some(PartialLatinSquare::empty())
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

        while self.index >= N * N {
            let gen = self.gen.as_mut().unwrap();

            let sq = gen.next()?;
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

        Some(sq)
    }
}

#[derive(Debug)]
pub struct PartialSquareGeneratorDyn {
    sq: LatinSquareDyn,
    partial_sq: Option<PartialLatinSquareDyn>,
    entries_left: usize,
    index: usize,
    gen: Option<Box<PartialSquareGeneratorDyn>>,
}

impl PartialSquareGeneratorDyn {
    pub fn new(sq: LatinSquareDyn, num_entries: usize) -> Self {
        let n = sq.n();
        let mut gen = (num_entries != 0)
            .then(|| Box::new(PartialSquareGeneratorDyn::new(sq.clone(), num_entries - 1)));

        let partial_sq = if num_entries == 0 {
            Some(PartialLatinSquareDyn::empty(n))
        } else {
            gen.as_mut().unwrap().next()
        };

        PartialSquareGeneratorDyn {
            entries_left: num_entries,
            index: gen.as_ref().map_or(0, |gen| gen.index),
            gen,
            partial_sq,
            sq,
        }
    }

    pub fn new_partial(
        sq: LatinSquareDyn,
        partial: PartialLatinSquareDyn,
        num_entries: usize,
    ) -> Self {
        assert_eq!(sq.n(), partial.n());
        let n = sq.n();

        let current_entries = partial.num_entries();
        let entries_left = num_entries - current_entries;

        let mut gen = (entries_left != 0).then(|| {
            Box::new(PartialSquareGeneratorDyn::new_partial(
                sq.clone(),
                partial.clone(),
                num_entries - 1,
            ))
        });

        let partial_sq = if entries_left == 0 {
            Some(partial.clone())
        } else {
            gen.as_mut().unwrap().next()
        };

        PartialSquareGeneratorDyn {
            entries_left,
            index: gen
                .as_ref()
                .map_or(partial.first_empty_index().unwrap(), |gen| {
                    partial.next_empty_index(gen.index).unwrap_or(n * n)
                }),
            gen,
            partial_sq,
            sq,
        }
    }
}

impl Iterator for PartialSquareGeneratorDyn {
    type Item = PartialLatinSquareDyn;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.sq.n();

        if self.entries_left == 0 {
            let value = self.partial_sq.take();
            return value;
        }

        while self.index >= n * n {
            let gen = self.gen.as_mut().unwrap();

            let sq = gen.next()?;
            self.partial_sq = Some(sq);

            self.index = self
                .partial_sq
                .as_ref()
                .unwrap()
                .next_empty_index(gen.index + 1)
                .unwrap_or(n * n);
        }

        let mut sq = self.partial_sq.clone().unwrap();

        let i = self.index / n;
        let j = self.index % n;
        sq.set(i, j, Some(self.sq.get(i, j)));

        self.index = self
            .partial_sq
            .as_ref()
            .unwrap()
            .next_empty_index(self.index + 1)
            .unwrap_or(n * n);

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
