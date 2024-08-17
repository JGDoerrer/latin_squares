use std::{cmp::Ordering, fmt::Debug};

use crate::{
    bitset::BitSet16, isotopy_class_generator::CYCLE_STRUCTURES, latin_square::LatinSquare,
    permutation::Permutation, tuple_iterator::TupleIterator,
};

#[derive(Clone)]
pub struct RowPartialLatinSquare<const N: usize> {
    rows: [[u8; 16]; N],
    col_masks: [BitSet16; N],
    full_rows: usize,
    first_cycle_index: usize,
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

    pub fn new_first_cycle(first_cycle_index: usize) -> Self {
        let row_cycle = CYCLE_STRUCTURES[N][first_cycle_index];

        let mut rows = [[0; 16]; N];

        rows[0] = Self::FIRST_ROW;

        let mut index = 0;
        for cycle in row_cycle {
            let start_index = index;
            index += cycle;
            for j in 0..*cycle {
                rows[1][start_index + j] = (start_index + (j + 1) % cycle) as u8;
            }
        }

        let mut col_masks = [BitSet16::all_less_than(N); N];

        for i in 0..N {
            col_masks[i].remove(i);
            col_masks[i].remove(rows[1][i] as usize);
        }

        Self {
            rows,
            full_rows: 2,
            first_cycle_index,
            col_masks,
        }
    }

    pub fn new_first_row() -> Self {
        let mut rows = [[0; 16]; N];

        rows[0] = Self::FIRST_ROW;

        let mut col_masks = [BitSet16::all_less_than(N); N];

        for i in 0..N {
            col_masks[i].remove(i);
        }

        Self {
            rows,
            full_rows: 1,
            first_cycle_index: 0,
            col_masks,
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

    pub fn add_row(&mut self, row: [u8; N]) {
        debug_assert!(self.is_valid_next_row(row));
        let row = Self::pad_row(row);

        for i in 0..N {
            self.col_masks[i].remove(row[i] as usize);
        }

        self.rows[self.full_rows] = row;

        if self.full_rows == 1 {
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = self.rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = self.rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let mut cycles = row_permutation.cycle_lengths();
            cycles.sort();

            let cycle_index = CYCLE_STRUCTURES[N]
                .iter()
                .position(|c| c == &cycles)
                .unwrap();

            self.first_cycle_index = cycle_index;
        }

        self.full_rows += 1;
    }

    pub fn first_cycle_index(&self) -> usize {
        self.first_cycle_index
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

    pub fn is_valid_next_row(&self, row: [u8; N]) -> bool {
        for i in 0..self.full_rows {
            for rows in [
                [Self::shrink_row(*self.get_row(i)), row],
                [row, Self::shrink_row(*self.get_row(i))],
            ] {
                let row_permutation = {
                    let mut permutation = [0; N];

                    for i in 0..N {
                        let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                        permutation[i] = rows[1][position].into();
                    }

                    Permutation::from_array(permutation)
                };

                let mut cycles = row_permutation.cycle_lengths();
                cycles.sort();

                if cycles.as_slice() < CYCLE_STRUCTURES[N][self.first_cycle_index] {
                    return false;
                }
            }
        }

        true
    }

    pub fn is_minimal(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> bool {
        let min_cycles = CYCLE_STRUCTURES[N][self.first_cycle_index];

        for rows in TupleIterator::<2>::new(self.full_rows)
            .flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
        {
            let rows = rows.map(|i| Self::shrink_row(self.rows[i]));
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let mut cycle_lens = row_permutation.cycle_lengths();
            cycle_lens.sort();

            if cycle_lens.as_slice() < min_cycles {
                unreachable!()
            }

            if cycle_lens == min_cycles {
                let mut cycles = row_permutation.cycles();
                cycles.sort();
                cycles.sort_by_key(|c| c.len());

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

                let column_permutation =
                    Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v.into())));

                let mut sq = self.clone();
                sq.permute_cols_vals_simd(&column_permutation, &symbol_permutation);

                let permutations = &lookup[self.first_cycle_index];

                for (s, c) in permutations {
                    let mut sq = sq.clone();
                    sq.permute_cols_vals_simd(c, s);

                    // dbg!(&sq);
                    sq.sort_rows();
                    // dbg!(&sq);

                    if sq.full_rows < self.full_rows {
                        continue;
                    }

                    if sq.cmp_rows(self).is_lt() {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// does not fix col_masks
    fn permute_cols_vals_simd(
        &mut self,
        col_permutation: &Permutation<N>,
        val_permutation: &Permutation<N>,
    ) {
        use std::simd::Simd;

        assert!(N <= 16);

        let mut col_permutation_simd = [0; 16];
        col_permutation_simd[0..N].copy_from_slice(
            &col_permutation
                .clone()
                .inverse()
                .into_array()
                .map(|v| v as u8),
        );
        let col_permutation = Simd::from_array(col_permutation_simd);

        let mut val_permutation_simd = [0; 16];
        val_permutation_simd[0..N]
            .copy_from_slice(&val_permutation.clone().into_array().map(|v| v as u8));
        let val_permutation = Simd::from_array(val_permutation_simd);

        for i in 0..self.full_rows {
            let simd = Simd::from_array(self.rows[i]);
            let new_row = val_permutation
                .swizzle_dyn(simd)
                .swizzle_dyn(col_permutation)
                .to_array();

            self.rows[i] = new_row;
        }
    }

    pub fn sort_rows(&mut self) {
        let mut new_rows = [[0; 16]; N];

        for i in 0..self.full_rows {
            let j = self.rows[i][0] as usize;
            if j >= self.full_rows {
                self.full_rows -= 1;
                continue;
            }
            new_rows[j] = self.rows[i];
        }

        self.rows = new_rows;
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
