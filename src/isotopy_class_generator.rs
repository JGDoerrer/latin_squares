use crate::{
    bitset::BitSet16, cycles::PermutationSimdLookup, latin_square::LatinSquare,
    row_partial_latin_square::RowPartialLatinSquare,
};

/// Generates latin squares by filling them one row at a time
pub struct IsotopyClassGenerator<'a, const N: usize> {
    row_generators: Vec<RowGenerator<'a, N>>,
    lookup: &'a PermutationSimdLookup,
}

impl<'a, const N: usize> IsotopyClassGenerator<'a, N> {
    pub fn new(lookup: &'a PermutationSimdLookup) -> Self {
        IsotopyClassGenerator {
            row_generators: vec![RowGenerator::new(
                RowPartialLatinSquare::new_first_row(),
                lookup,
            )],
            lookup,
        }
    }
}

impl<'a, const N: usize> Iterator for IsotopyClassGenerator<'a, N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row_generators.is_empty() {
            return None;
        }

        while let Some(generator) = self.row_generators.last_mut() {
            let Some(sq) = generator.next() else {
                self.row_generators.pop();
                continue;
            };

            if sq.is_complete() {
                let sq: LatinSquare<N> = sq.try_into().unwrap();

                debug_assert_eq!(sq, sq.isotopy_class());

                return Some(sq);
            }

            self.row_generators.push(RowGenerator::new(sq, self.lookup));
        }

        None
    }
}

/// fills a row in all (minimal) possible ways
pub struct RowGenerator<'a, const N: usize> {
    indices: [usize; N],
    lookup: &'a PermutationSimdLookup,
    sq: RowPartialLatinSquare<N>,
}

impl<'a, const N: usize> RowGenerator<'a, N> {
    pub fn new(sq: RowPartialLatinSquare<N>, lookup: &'a PermutationSimdLookup) -> Self {
        RowGenerator {
            sq,
            indices: [0; N],
            lookup,
        }
    }
}

impl<'a, const N: usize> Iterator for RowGenerator<'a, N> {
    type Item = RowPartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        'l: loop {
            let mut sq = self.sq.clone();
            let mut new_row = [0; 16];

            let mut values = BitSet16::all_less_than(N);

            let first_value = values
                .intersect(sq.get_col_mask(0))
                .into_iter()
                .next()
                .unwrap();
            new_row[0] = first_value as u8;
            values.remove(first_value);

            for i in 1..N {
                let index = self.indices[i];

                let possible_values = values.intersect(sq.get_col_mask(i));

                let Some(value) = possible_values.into_iter().nth(index) else {
                    if i == 1 {
                        return None;
                    } else {
                        self.indices[i - 1] += 1;
                        for i in i..N {
                            self.indices[i] = 0;
                        }
                        continue 'l;
                    }
                };

                values.remove(value);
                new_row[i] = value as u8;
            }
            self.indices[N - 1] += 1;

            if !sq.add_row(new_row) {
                continue;
            }

            if sq.full_rows() != N - 1 && !sq.is_minimal(self.lookup) {
                continue;
            }

            return Some(sq);
        }
    }
}

#[cfg(test)]
mod test {

    use crate::cycles::generate_minimize_rows_lookup_simd;

    use super::*;

    #[test]
    fn isotopy_class_count() {
        let lookup4 = generate_minimize_rows_lookup_simd::<4>();
        assert_eq!(IsotopyClassGenerator::<4>::new(&lookup4).count(), 2);

        let lookup5 = generate_minimize_rows_lookup_simd::<5>();
        assert_eq!(IsotopyClassGenerator::<5>::new(&lookup5).count(), 2);

        let lookup6 = generate_minimize_rows_lookup_simd::<6>();
        assert_eq!(IsotopyClassGenerator::<6>::new(&lookup6).count(), 22);

        let lookup7 = generate_minimize_rows_lookup_simd::<7>();
        assert_eq!(IsotopyClassGenerator::<7>::new(&lookup7).count(), 564);
    }
}
