use crate::{
    latin_square::Cell, orthogonal_array::OrthogonalArray,
    partial_orthogonal_array::PartialOrthogonalArray,
};

#[derive(Debug)]
pub struct PartialOAGenerator<const N: usize, const MOLS: usize> {
    oa: OrthogonalArray<N, MOLS>,
    partial_oa: Option<PartialOrthogonalArray<N, MOLS>>,
    entries_left: usize,
    index: usize,
    gen: Option<Box<PartialOAGenerator<N, MOLS>>>,
}

impl<const N: usize, const MOLS: usize> PartialOAGenerator<N, MOLS> {
    pub fn new(oa: OrthogonalArray<N, MOLS>, num_entries: usize) -> Self {
        let mut gen = (num_entries != 0)
            .then(|| Box::new(PartialOAGenerator::new(oa.clone(), num_entries - 1)));

        let partial_oa = if num_entries == 0 {
            Some(PartialOrthogonalArray::empty())
        } else {
            gen.as_mut().unwrap().next()
        };

        PartialOAGenerator {
            entries_left: num_entries,
            index: gen.as_ref().map_or(0, |gen| gen.index),
            gen,
            partial_oa,
            oa,
        }
    }

    pub fn new_partial(
        oa: OrthogonalArray<N, MOLS>,
        partial: PartialOrthogonalArray<N, MOLS>,
        num_entries: usize,
    ) -> Self {
        let current_entries = partial.num_entries();
        let entries_left = num_entries - current_entries;

        let mut gen = (entries_left != 0).then(|| {
            Box::new(PartialOAGenerator::new_partial(
                oa.clone(),
                partial.clone(),
                num_entries - 1,
            ))
        });

        let partial_oa = if entries_left == 0 {
            Some(partial.clone())
        } else {
            gen.as_mut().unwrap().next()
        };

        PartialOAGenerator {
            entries_left,
            index: gen
                .as_ref()
                .map_or(partial.first_empty_index().unwrap(), |gen| {
                    partial.next_empty_index(gen.index).unwrap_or(N * N * MOLS)
                }),
            gen,
            partial_oa,
            oa,
        }
    }
}

impl<const N: usize, const MOLS: usize> Iterator for PartialOAGenerator<N, MOLS> {
    type Item = PartialOrthogonalArray<N, MOLS>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries_left == 0 {
            let value = self.partial_oa.take();
            return value;
        }

        while self.index >= N * N * MOLS {
            let gen = self.gen.as_mut().unwrap();

            let sq = gen.next()?;
            self.partial_oa = Some(sq);

            self.index = self
                .partial_oa
                .as_ref()
                .unwrap()
                .next_empty_index(gen.index + 1)
                .unwrap_or(N * N * MOLS);
        }

        let mut oa = self.partial_oa.clone().unwrap();

        let column = self.index / (N * N);
        let Cell(i, j) = Cell::from_index::<N>(self.index % (N * N));
        oa.set(column, i, j, Some(self.oa.get(column, i, j)));

        self.index = self
            .partial_oa
            .as_ref()
            .unwrap()
            .next_empty_index(self.index + 1)
            .unwrap_or(N * N);

        Some(oa)
    }
}
