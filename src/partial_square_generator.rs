use crate::{latin_square_dyn::LatinSquareDyn, partial_latin_square_dyn::PartialLatinSquareDyn};

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
        assert!(num_entries >= current_entries);
        let entries_left = num_entries - current_entries;

        let mut gen = if entries_left > 0 {
            Some(Box::new(PartialSquareGeneratorDyn::new_partial(
                sq.clone(),
                partial.clone(),
                num_entries - 1,
            )))
        } else {
            None
        };

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
