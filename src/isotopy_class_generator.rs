use crate::{
    bitset::BitSet16, latin_square::LatinSquare, permutation::Permutation,
    row_partial_latin_square::RowPartialLatinSquare,
};

/// Generates latin squares by filling them one row at a time
pub struct IsotopyClassGenerator<'a, const N: usize> {
    row_generators: Vec<RowGenerator<'a, N>>,
    lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
}

impl<'a, const N: usize> IsotopyClassGenerator<'a, N> {
    pub fn new(lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>) -> Self {
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

        'r: while let Some(generator) = self.row_generators.last_mut() {
            let Some(sq) = generator.next() else {
                self.row_generators.pop();
                continue;
            };

            if sq.is_complete() {
                let sq: LatinSquare<N> = sq.try_into().unwrap();

                debug_assert_eq!(sq, sq.isotopy_class_lookup(&self.lookup));

                return Some(sq);
            }

            let next_row_index = sq.full_rows();

            if sq.first_cycle_index() != 0 {
                // check for rows with disallowed cycle structure
                for rows in (0..next_row_index - 1)
                    .flat_map(|i| [[next_row_index - 1, i], [i, next_row_index - 1]])
                {
                    let rows = rows.map(|i| sq.get_row(i));

                    let row_permutation = {
                        let mut permutation = [0; N];

                        for i in 0..N {
                            let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                            permutation[i] = rows[1][position].into();
                        }

                        Permutation::from_array(permutation)
                    };

                    let mut cycles: Vec<_> = row_permutation.cycle_lengths();
                    cycles.sort();

                    if !CYCLE_STRUCTURES[N][sq.first_cycle_index()..].contains(&cycles.as_slice()) {
                        continue 'r;
                    }
                }
            }

            self.row_generators.push(RowGenerator::new(sq, self.lookup));
        }

        None
    }
}

/// fills a row in all (minimal) possible ways
struct RowGenerator<'a, const N: usize> {
    indices: [usize; N],
    lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
    sq: RowPartialLatinSquare<N>,
}

impl<'a, const N: usize> RowGenerator<'a, N> {
    fn new(
        sq: RowPartialLatinSquare<N>,
        lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
    ) -> Self {
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
            let row_index = sq.full_rows();
            let mut new_row = [0; N];

            let mut values = BitSet16::all_less_than(N);

            new_row[0] = row_index as u8;
            values.remove(row_index);

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

            if !sq.is_valid_next_row(new_row) {
                continue;
            }

            sq.add_row(new_row);

            if row_index != N - 2 && !sq.is_minimal(self.lookup) {
                continue;
            }

            return Some(sq);
        }
    }
}

pub const CYCLE_STRUCTURES: [&[&[usize]]; 11] = [
    &[&[0]],
    &[&[1]],
    &[&[2]],
    &[&[3]],
    &[&[2, 2], &[4]],
    &[&[2, 3], &[5]],
    &[&[2, 2, 2], &[2, 4], &[3, 3], &[6]],
    &[&[2, 2, 3], &[2, 5], &[3, 4], &[7]],
    &[
        &[2, 2, 2, 2],
        &[2, 2, 4],
        &[2, 3, 3],
        &[2, 6],
        &[3, 5],
        &[4, 4],
        &[8],
    ],
    &[
        &[2, 2, 2, 3],
        &[2, 2, 5],
        &[2, 3, 4],
        &[2, 7],
        &[3, 3, 3],
        &[3, 6],
        &[4, 5],
        &[9],
    ],
    &[
        &[2, 2, 2, 2, 2],
        &[2, 2, 2, 4],
        &[2, 2, 3, 3],
        &[2, 2, 6],
        &[2, 3, 5],
        &[2, 4, 4],
        &[2, 8],
        &[3, 3, 4],
        &[3, 7],
        &[4, 6],
        &[5, 5],
        &[10],
    ],
];

#[cfg(test)]
mod test {

    use crate::latin_square::generate_minimize_rows_lookup;

    use super::*;

    #[test]
    fn isotopy_class_count() {
        let lookup4 = generate_minimize_rows_lookup::<4>();
        assert_eq!(IsotopyClassGenerator::new(&lookup4).count(), 2);

        let lookup5 = generate_minimize_rows_lookup::<5>();
        assert_eq!(IsotopyClassGenerator::new(&lookup5).count(), 2);

        let lookup6 = generate_minimize_rows_lookup::<6>();
        assert_eq!(IsotopyClassGenerator::new(&lookup6).count(), 22);

        let lookup7 = generate_minimize_rows_lookup::<7>();
        assert_eq!(IsotopyClassGenerator::new(&lookup7).count(), 564);
    }
}
