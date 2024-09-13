use std::{cmp::Ordering, fmt::Debug};

use crate::{
    bitset::BitSet16,
    cycles::CYCLE_STRUCTURES,
    latin_square::LatinSquare,
    permutation::{Permutation, PermutationIter},
    permutation_simd::PermutationSimd,
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

    #[inline]
    fn pad_row(row: [u8; N]) -> [u8; 16] {
        assert!(N <= 16);
        let mut new_row = [0; 16];
        new_row[..N].copy_from_slice(&row);
        new_row
    }

    #[inline]
    fn shrink_row(row: [u8; 16]) -> [u8; N] {
        assert!(N <= 16);
        let mut new_row = [0; N];
        new_row.copy_from_slice(&row[..N]);
        new_row
    }

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

    fn from_rows(
        in_rows: [[u8; 16]; N],
        lookup: &[Vec<(PermutationSimd, PermutationSimd)>],
    ) -> Self {
        let full_rows = N;

        let mut min_row_cycle_index = CYCLE_STRUCTURES[N].len() - 1;
        let mut min_rows = in_rows;
        let mut min_row_cycles = [[false; N]; N];

        for rows in TupleIterator::<2>::new(full_rows) {
            for row_indices in [[rows[0], rows[1]], [rows[1], rows[0]]] {
                let rows = row_indices.map(|i| Self::shrink_row(in_rows[i]));

                let row_permutation = {
                    let mut permutation = [0; N];

                    for i in 0..N {
                        let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                        permutation[i] = rows[1][position].into();
                    }

                    Permutation::from_array(permutation)
                };

                let cycle_index = row_permutation.cycle_lengths_index();
                match cycle_index.cmp(&min_row_cycle_index) {
                    Ordering::Less => {
                        min_row_cycle_index = cycle_index;
                        min_row_cycles = [[false; N]; N];
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => break,
                }
                min_row_cycles[row_indices[0]][row_indices[1]] = true;

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
                    Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v as usize)))
                        .inverse();

                let (rows, _) = Self::permuted_cols_vals_simd(
                    &in_rows[0..full_rows],
                    &inverse_column_permutation.into_simd(),
                    &symbol_permutation.into_simd(),
                    false,
                );

                let permutations = &lookup[min_row_cycle_index];

                for (s, inverse_c) in permutations {
                    let (rows, new_full_rows) =
                        Self::permuted_cols_vals_simd(&rows[0..full_rows], inverse_c, s, true);

                    if new_full_rows != full_rows {
                        continue;
                    }

                    for i in 0..full_rows {
                        match min_rows[i][0..N].cmp(&rows[i][0..N]) {
                            Ordering::Less => break,
                            Ordering::Equal => {}
                            Ordering::Greater => min_rows = rows,
                        }
                    }
                }
            }
        }

        let col_masks = [BitSet16::empty(); N];

        Self {
            rows: min_rows,
            min_row_cycles,
            min_row_cycle_index,
            col_masks,
            full_rows: N,
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
    pub fn add_row(&mut self, padded_row: [u8; 16]) -> bool {
        let row: [u8; N] = Self::shrink_row(padded_row);

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

    pub fn is_minimal(&self, lookup: &[Vec<(PermutationSimd, PermutationSimd)>]) -> bool {
        for rows in TupleIterator::<2>::new(self.full_rows) {
            if !self.min_row_cycles[rows[0]][rows[1]] {
                continue;
            }

            for rows in [[rows[0], rows[1]], [rows[1], rows[0]]] {
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
                    Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v as usize)))
                        .inverse();

                let (rows, _) = Self::permuted_cols_vals_simd(
                    &self.rows[0..self.full_rows],
                    &inverse_column_permutation.into_simd(),
                    &symbol_permutation.into_simd(),
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
                        match self.rows[i][0..N].cmp(&rows[i][0..N]) {
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
        inverse_column_permutation: &PermutationSimd,
        val_permutation: &PermutationSimd,
        sort_rows: bool,
    ) -> ([[u8; 16]; N], usize) {
        use std::simd::Simd;

        assert!(N <= 16);

        let col_permutation_simd = inverse_column_permutation.clone().into_array();
        let col_permutation = Simd::from_array(col_permutation_simd);

        let val_permutation_simd = val_permutation.clone().into_array();
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

    fn from_rcv(
        rows: &[[u8; 16]],
        cols: &[[u8; 16]],
        vals: &[[u8; 16]],
        lookup: &[Vec<(PermutationSimd, PermutationSimd)>],
    ) -> Self {
        let mut new_values = [[0; 16]; N];

        for i in 0..N {
            for j in 0..N {
                let row = rows[i][j] as usize;
                let col = cols[i][j] as usize;
                let val = vals[i][j];

                new_values[row][col] = val;
            }
        }

        Self::from_rows(new_values, lookup)
    }

    fn permuted_rcs(
        &self,
        permutation: &Permutation<3>,
        lookup: &[Vec<(PermutationSimd, PermutationSimd)>],
    ) -> Self {
        assert!(N <= 16);
        const ROWS: [[u8; 16]; 16] = {
            let mut rows = [[0; 16]; 16];
            let mut i = 0;
            while i < 16 {
                rows[i] = [i as u8; 16];
                i += 1;
            }
            rows
        };
        const COLS: [[u8; 16]; 16] = {
            let mut row = [0; 16];
            let mut i = 0;
            while i < 16 {
                row[i] = i as u8;
                i += 1;
            }
            [row; 16]
        };

        let rows = &ROWS[0..N];
        let cols = &COLS[0..N];
        let vals = &self.rows[0..N];

        let [rows, cols, vals] = permutation.apply_array([rows, cols, vals]);
        Self::from_rcv(rows, cols, vals, lookup)
    }

    pub fn is_minimal_main_class(
        &self,
        lookup: &[Vec<(PermutationSimd, PermutationSimd)>],
    ) -> bool {
        debug_assert!(self.is_complete());

        for conjugate in PermutationIter::new()
            .skip(1)
            .map(|perm| self.permuted_rcs(&perm, lookup))
        {
            'i: for i in 0..N {
                for j in 0..N {
                    match self.rows[i][j].cmp(&conjugate.rows[i][j]) {
                        Ordering::Less => break 'i,
                        Ordering::Equal => {}
                        Ordering::Greater => return false,
                    }
                }
            }
        }

        true
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
