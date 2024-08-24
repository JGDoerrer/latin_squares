use std::{cmp::Ordering, fmt::Debug};

use crate::{
    bitset::BitSet16,
    latin_square::LatinSquare,
    permutation::{Permutation, CYCLE_STRUCTURES},
    tuple_iterator::TupleIterator,
};

#[derive(Clone)]
pub struct RowPartialLatinSquare<const N: usize> {
    rows: [[u8; 16]; N],
    col_masks: [BitSet16; N],
    full_rows: usize,
    min_row_cycles: [[bool; N]; N],
    min_row_cycle_index: usize,
}

impl<const N: usize> RowPartialLatinSquare<N> {
    const FIRST_ROW: [u8; 16] = {
        let mut row = [0; 16];

        let mut i = 0;
        while i < N {
            row[i] = i as u8;
            i += 1;
        }

        row
    };

    fn pad_row(row: [u8; N]) -> [u8; 16] {
        assert!(N <= 16);
        let mut new_row = [0; 16];
        new_row[..N].copy_from_slice(&row);
        new_row
    }

    fn shrink_row(row: [u8; 16]) -> [u8; N] {
        assert!(N <= 16);
        let mut new_row = [0; N];
        new_row.copy_from_slice(&row[..N]);
        new_row
    }

    // pub fn new_first_cycle(first_cycle_index: usize) -> Self {
    //     let row_cycle = CYCLE_STRUCTURES[N][first_cycle_index];

    //     let mut rows = [[0; 16]; N];

    //     rows[0] = Self::FIRST_ROW;

    //     let mut index = 0;
    //     for cycle in row_cycle {
    //         let start_index = index;
    //         index += cycle;
    //         for j in 0..*cycle {
    //             rows[1][start_index + j] = (start_index + (j + 1) % cycle) as u8;
    //         }
    //     }

    //     let mut col_masks = [BitSet16::all_less_than(N); N];

    //     for i in 0..N {
    //         col_masks[i].remove(i);
    //         col_masks[i].remove(rows[1][i] as usize);
    //     }

    //     let mut min_row_cycles = [[false; N]; N];
    //     min_row_cycles[0][1] = true;

    //     let mut permutations = array::from_fn(|_| {
    //         array::from_fn(|_| (Permutation::<N>::identity(), Permutation::<N>::identity()))
    //     });

    //     Self {
    //         rows,
    //         full_rows: 2,
    //         col_masks,
    //         min_row_cycles,
    //         min_row_cycle_index: first_cycle_index,
    //         permutations,
    //     }
    // }

    pub fn new_first_row() -> Self {
        let mut rows = [[0; 16]; N];

        rows[0] = Self::FIRST_ROW;

        let mut col_masks = [BitSet16::all_less_than(N); N];

        for i in 0..N {
            col_masks[i].remove(i);
        }

        let min_row_cycles = [[false; N]; N];

        Self {
            rows,
            full_rows: 1,
            col_masks,
            min_row_cycles,
            min_row_cycle_index: 0,
        }
    }

    pub fn get_col_mask(&self, col_index: usize) -> BitSet16 {
        self.col_masks[col_index]
    }

    pub fn get_row(&self, row_index: usize) -> &[u8; 16] {
        &self.rows[row_index]
    }

    pub fn full_rows(&self) -> usize {
        self.full_rows
    }

    pub fn is_complete(&self) -> bool {
        self.full_rows == N
    }

    /// returns true if minimal, false if not
    pub fn add_row(&mut self, row: [u8; N]) -> bool {
        let padded_row = Self::pad_row(row);

        for i in 0..N {
            self.col_masks[i].remove(padded_row[i] as usize);
        }

        self.rows[self.full_rows] = padded_row;
        let new_row_index = self.full_rows;

        if self.full_rows == 1 {
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = self.rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = self.rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let cycle_index = row_permutation.cycle_lengths_index();

            self.min_row_cycle_index = cycle_index;
            self.min_row_cycles[0][1] = true;
            self.full_rows += 1;
            true
        } else {
            for i in 0..self.full_rows {
                for rows in [
                    [Self::shrink_row(self.rows[i]), row],
                    [row, Self::shrink_row(self.rows[i])],
                ] {
                    let row_permutation = {
                        let mut permutation = [0; N];

                        for i in 0..N {
                            let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                            permutation[i] = rows[1][position].into();
                        }

                        Permutation::from_array(permutation)
                    };

                    let cycle_index = row_permutation.cycle_lengths_index();

                    if cycle_index < self.min_row_cycle_index {
                        return false;
                    }
                    if cycle_index == self.min_row_cycle_index {
                        self.min_row_cycles[i][new_row_index] = true;
                    }
                }
            }
            self.full_rows += 1;
            true
        }
    }

    pub fn min_cycle_index(&self) -> usize {
        self.min_row_cycle_index
    }

    // fn from_rows(rows: [[u8; 16]; N], full_rows: usize, col_masks: [BitSet16; N]) -> Self {
    //     debug_assert_eq!(rows[0], Self::FIRST_ROW);

    //     let first_cycle_index = CYCLE_STRUCTURES[N]
    //         .iter()
    //         .position(|c| c == &Permutation::<N>::from_slice(&rows[1][0..N]).cycle_lengths())
    //         .unwrap();

    //     Self {
    //         rows,
    //         full_rows,
    //         first_cycle_index,
    //         col_masks,
    //     }
    // }

    // pub fn is_valid_next_row(&self, row: [u8; N]) -> bool {
    //     for i in 0..self.full_rows {
    //         for rows in [
    //             [Self::shrink_row(*self.get_row(i)), row],
    //             [row, Self::shrink_row(*self.get_row(i))],
    //         ] {
    //             let row_permutation = {
    //                 let mut permutation = [0; N];

    //                 for i in 0..N {
    //                     let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
    //                     permutation[i] = rows[1][position].into();
    //                 }

    //                 Permutation::from_array(permutation)
    //             };

    //             let mut cycles = row_permutation.cycle_lengths();
    //             cycles.sort();

    //             if cycles.as_slice() < CYCLE_STRUCTURES[N][self.min_row_cycle_index] {
    //                 return false;
    //             }
    //         }
    //     }

    //     true
    // }

    pub fn is_minimal(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> bool {
        for rows in TupleIterator::<2>::new(self.full_rows) {
            if !self.min_row_cycles[rows[0]][rows[1]] {
                continue;
            }

            for rows in [[rows[0], rows[1]], [rows[1], rows[0]]] {
                // let (inverse_column_permutation, symbol_permutation) =
                //     &self.permutations[rows[0]][rows[1]];

                let rows = rows.map(|i| Self::shrink_row(self.rows[i]));

                let row_permutation = {
                    let mut permutation = [0; N];

                    for i in 0..N {
                        let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                        permutation[i] = rows[1][position].into();
                    }

                    Permutation::from_array(permutation)
                };

                let mut cycles = row_permutation.cycles();
                cycles.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));

                let symbol_permutation = {
                    let mut permutation = [0; N];

                    let mut index = 0;
                    for cycle in cycles {
                        let cycle_len = cycle.len();
                        let start_index = index;
                        index += cycle_len;
                        for (i, j) in cycle.into_iter().enumerate() {
                            permutation[j] = start_index + (i + 1) % cycle_len;
                        }
                    }

                    Permutation::from_array(permutation)
                };

                let inverse_column_permutation =
                    Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v.into())))
                        .inverse();

                let (rows, _) = Self::permuted_cols_vals_simd(
                    &self.rows[0..self.full_rows],
                    &inverse_column_permutation,
                    &symbol_permutation,
                    false,
                );

                let permutations = &lookup[self.min_row_cycle_index];

                for (s, inverse_c) in permutations {
                    let (rows, full_rows) =
                        Self::permuted_cols_vals_simd(&rows[0..self.full_rows], inverse_c, s, true);

                    if full_rows != self.full_rows {
                        continue;
                    }

                    for i in 0..self.full_rows {
                        match self.rows[i].cmp(&rows[i]) {
                            Ordering::Less => break,
                            Ordering::Equal => {}
                            Ordering::Greater => return false,
                        }
                    }
                }
            }
        }

        true
    }

    /// does not fix col_masks
    fn permuted_cols_vals_simd(
        rows: &[[u8; 16]],
        inverse_column_permutation: &Permutation<N>,
        val_permutation: &Permutation<N>,
        sort_rows: bool,
    ) -> ([[u8; 16]; N], usize) {
        use std::simd::Simd;

        assert!(N <= 16);

        let mut col_permutation_simd = [0xff; 16];
        col_permutation_simd[0..N]
            .copy_from_slice(&inverse_column_permutation.as_array().map(|v| v as u8));
        let col_permutation = Simd::from_array(col_permutation_simd);

        let mut val_permutation_simd = [0xff; 16];
        val_permutation_simd[0..N].copy_from_slice(&val_permutation.as_array().map(|v| v as u8));
        let val_permutation = Simd::from_array(val_permutation_simd);

        let mut new_rows = [[0; 16]; N];
        let mut full_rows = rows.len();

        for i in 0..rows.len() {
            let simd = Simd::from_array(rows[i]);
            let new_row = val_permutation
                .swizzle_dyn(simd)
                .swizzle_dyn(col_permutation)
                .to_array();

            if sort_rows {
                if full_rows > new_row[0] as usize {
                    new_rows[new_row[0] as usize] = new_row;
                } else {
                    full_rows -= 1;
                }
            } else {
                new_rows[i] = new_row;
            }
        }

        (new_rows, full_rows)
    }

    pub fn cmp_rows(&self, other: &Self) -> Ordering {
        for i in 0..self.full_rows.min(other.full_rows) {
            match self.rows[i][0..N].cmp(&other.rows[i][0..N]) {
                Ordering::Equal => {}
                o => return o,
            }
        }

        Ordering::Equal
    }
}

impl<const N: usize> TryFrom<RowPartialLatinSquare<N>> for LatinSquare<N> {
    type Error = ();

    fn try_from(sq: RowPartialLatinSquare<N>) -> Result<Self, Self::Error> {
        if !sq.is_complete() {
            return Err(());
        }

        let rows = sq.rows.map(RowPartialLatinSquare::shrink_row);

        Ok(LatinSquare::new(rows))
    }
}

impl<const N: usize> Debug for RowPartialLatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for i in 0..self.full_rows {
            writeln!(f, "+{}", "---+".repeat(N))?;
            write!(f, "|")?;
            for j in 0..N {
                let value = self.get_row(i)[j];
                write!(f, " {} |", value)?;
            }
            writeln!(f)?;
        }
        for _ in self.full_rows..N {
            writeln!(f, "+{}", "---+".repeat(N))?;
            write!(f, "|")?;
            write!(f, "{}", "   |".repeat(N))?;
            writeln!(f)?;
        }
        write!(f, "+{}", "---+".repeat(N))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {}
}
