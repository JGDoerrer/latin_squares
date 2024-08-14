use core::fmt::Debug;
use std::{
    cmp::{Ordering, Reverse},
    fmt::{Display, Write},
};

use crate::{
    bitset::BitSet16,
    latin_square::{minimize_rows_with_lookup, LatinSquare},
    latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait},
    permutation::{Permutation, PermutationIter},
    tuple_iterator::TupleIterator,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartialLatinSquare<const N: usize> {
    rows: [[Option<u8>; N]; N],
}

impl<const N: usize> Default for PartialLatinSquare<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const N: usize> PartialLatinSquareTrait for PartialLatinSquare<N> {
    fn n(&self) -> usize {
        N
    }

    fn get_partial(&self, row: usize, col: usize) -> Option<usize> {
        self.rows[row][col].map(|val| val.into())
    }
}

impl<const N: usize> PartialLatinSquare<N> {
    pub const fn empty() -> Self {
        PartialLatinSquare {
            rows: [[None; N]; N],
        }
    }

    pub fn from_array(values: [[Option<u8>; N]; N]) -> Self {
        PartialLatinSquare { rows: values }
    }

    pub fn from_rcv(
        rows: [[Option<usize>; N]; N],
        cols: [[Option<usize>; N]; N],
        vals: [[Option<usize>; N]; N],
    ) -> Self {
        let mut new_values = [[None; N]; N];

        for i in 0..N {
            for j in 0..N {
                let Some(row) = rows[i][j] else {
                    continue;
                };
                let Some(col) = cols[i][j] else {
                    continue;
                };
                let Some(val) = vals[i][j] else {
                    continue;
                };

                new_values[row][col] = Some(val as u8);
            }
        }

        Self::from_array(new_values)
    }

    pub fn values(self) -> [[Option<u8>; N]; N] {
        self.rows
    }

    pub fn get(&self, row: usize, col: usize) -> Option<usize> {
        self.rows[row][col].map(|val| val.into())
    }

    pub fn get_row(&self, i: usize) -> &[Option<u8>; N] {
        &self.rows[i]
    }

    pub fn get_col(&self, i: usize) -> [Option<u8>; N] {
        let mut col = [None; N];

        for (j, val) in col.iter_mut().enumerate() {
            *val = self.rows[j][i];
        }

        col
    }

    /// Returns for each row, in which column the value `i` appears
    pub fn get_val(&self, i: usize) -> [Option<u8>; N] {
        let mut val = [None; N];

        for j in 0..N {
            val[j] = self
                .get_row(j)
                .iter()
                .position(|v| v.is_some_and(|v| v as usize == i))
                .map(|v| v as u8);
        }

        val
    }

    pub fn set(&mut self, i: usize, j: usize, value: Option<usize>) {
        self.rows[i][j] = value.map(|v| v as u8);
    }

    pub fn is_complete(&self) -> bool {
        self.rows
            .iter()
            .all(|row| row.iter().all(|val| val.is_some()))
    }

    pub fn is_reduced(&self) -> bool {
        for i in 0..N {
            if self.rows[0][i].is_some_and(|j| j != i as u8)
                || self.rows[i][0].is_some_and(|j| j != i as u8)
            {
                return false;
            }
        }
        true
    }

    pub fn is_valid(&self) -> bool {
        (0..N).all(|i| {
            (0..N)
                .filter_map(|j| self.get_partial(i, j))
                .collect::<BitSet16>()
                .len()
                == (0..N).filter_map(|j| self.get_partial(i, j)).count()
                && (0..N)
                    .filter_map(|j| self.get_partial(j, i))
                    .collect::<BitSet16>()
                    .len()
                    == (0..N).filter_map(|j| self.get_partial(j, i)).count()
        })
    }

    pub fn transposed(&self) -> Self {
        let mut values = [[None; N]; N];

        for (i, row) in values.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.rows[j][i];
            }
        }

        Self { rows: values }
    }

    pub fn num_entries(&self) -> usize {
        (0..N)
            .map(|row| {
                (0..N)
                    .filter(|col| self.get_partial(row, *col).is_some())
                    .count()
            })
            .sum()
    }

    pub fn count_val(&self, value: usize) -> usize {
        (0..N)
            .flat_map(|col| {
                (0..N).filter(move |row| self.get_partial(*row, col).is_some_and(|i| i == value))
            })
            .count()
    }

    pub fn unique_entries(&self) -> BitSet16 {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get_partial(row, col)))
            .flatten()
            .collect::<BitSet16>()
    }

    pub fn first_empty_index(&self) -> Option<usize> {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get_partial(row, col)))
            .position(|entry| entry.is_none())
    }

    pub fn next_empty_index(&self, start: usize) -> Option<usize> {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get_partial(row, col)))
            .skip(start)
            .position(|entry| entry.is_none())
            .map(|index| index + start)
    }

    pub fn permute_rows(&self, permutation: &Permutation<N>) -> Self {
        let rows = permutation.apply_array(self.rows);

        Self { rows }
    }

    pub fn permute_cols(&self, permutation: &Permutation<N>) -> Self {
        let mut rows = self.rows;

        permutation.apply_arrays(&mut rows);

        Self { rows }
    }

    pub fn permute_rows_and_cols(&self, permutation: &Permutation<N>) -> Self {
        let mut values = [[None; N]; N];

        let permutation = permutation.as_array();

        for (i, new_row) in values.iter_mut().enumerate() {
            let row = self.rows[permutation[i]];

            for (j, new_val) in new_row.iter_mut().enumerate() {
                *new_val = row[permutation[j]];
            }
        }

        Self { rows: values }
    }

    pub fn permute_vals(&self, permutation: &Permutation<N>) -> Self {
        let mut values = self.rows;

        for row in &mut values {
            for val in row {
                if let Some(val) = val.as_mut() {
                    *val = permutation.apply(*val as usize) as u8;
                }
            }
        }

        Self { rows: values }
    }

    pub fn permute_cols_vals_simd(
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

        for i in 0..N {
            debug_assert!(self.rows[i]
                .iter()
                .all(|v| v.is_some() == self.rows[i][0].is_some()));

            if self.rows[i][0].is_none() {
                continue;
            }

            let mut simd = [0; 16];
            simd[0..N].copy_from_slice(&self.rows[i].map(|v| v.unwrap()));
            let simd = Simd::from_array(simd);
            let new_row = val_permutation
                .swizzle_dyn(simd)
                .swizzle_dyn(col_permutation);

            for j in 0..N {
                self.rows[i][j] = Some(new_row[j]);
            }
        }
    }

    pub fn sort_entries_top_left(&self) -> Self {
        let mut new = *self;

        let top_row_index = (0..N)
            .max_by_key(|i| new.get_row(*i).iter().flatten().count())
            .unwrap();
        let bottom_row_index = (0..N)
            .max_by_key(|i| {
                new.get_row(*i)
                    .iter()
                    .zip(*new.get_row(top_row_index))
                    .filter(|(a, b)| a.is_some() && b.is_none())
                    .count()
            })
            .unwrap();

        new.rows.swap(0, top_row_index);
        if bottom_row_index != 0 {
            new.rows.swap(N - 1, bottom_row_index);
        }

        let mut top_cols = Permutation::<N>::identity().into_array().map(|j| {
            (
                j,
                new.get_partial(0, j)
                    .is_some()
                    .then(|| new.get_col(j).iter().flatten().count()),
            )
        });

        top_cols.sort_by_key(|(_, c)| Reverse(c.unwrap_or(0)));

        let permutation: Permutation<N> = top_cols.map(|(i, _)| i).into();
        new = new.permute_cols(&permutation.inverse());

        let mut bottom_cols = Permutation::<N>::identity().into_array().map(|j| {
            (
                j,
                new.get_partial(N - 1, j)
                    .is_some()
                    .then(|| new.get_col(j).iter().flatten().count()),
            )
        });

        bottom_cols.sort_by_key(|(_, c)| (c.unwrap_or(0)));

        let permutation: Permutation<N> = bottom_cols.map(|(i, _)| i).into();
        new = new.permute_cols(&permutation.inverse());

        for i in 1..N - 1 {
            let (max_row, count) = (i..N - 1)
                .map(|j| {
                    (
                        j,
                        new.get_row(j)
                            .iter()
                            .zip(new.get_row(0))
                            .filter(|(a, b)| a.is_some() && b.is_some())
                            .count()
                            .saturating_sub(
                                new.get_row(j)
                                    .iter()
                                    .zip(new.get_row(N - 1))
                                    .filter(|(a, b)| a.is_some() && b.is_some())
                                    .count(),
                            ),
                    )
                })
                .max_by_key(|(_, count)| *count)
                .unwrap();

            if count == 0 {
                continue;
            }

            new.rows.swap(i, max_row);
        }

        for i in (1..N - 1).rev() {
            let (max_row, count) = (1..=i)
                .map(|j| {
                    (
                        j,
                        new.get_row(j)
                            .iter()
                            .zip(new.get_row(N - 1))
                            .filter(|(a, b)| a.is_some() && b.is_some())
                            .count()
                            .saturating_sub(
                                new.get_row(j)
                                    .iter()
                                    .zip(new.get_row(0))
                                    .filter(|(a, b)| a.is_some() && b.is_some())
                                    .count(),
                            ),
                    )
                })
                .max_by_key(|(_, count)| *count)
                .unwrap();

            if count == 0 {
                continue;
            }

            new.rows.swap(i, max_row);
        }

        new
    }

    pub fn has_entry_determined_by_row_col(&self) -> bool {
        let rows = self.rows.map(|row| {
            row.into_iter()
                .flatten()
                .map(|i| i as usize)
                .collect::<BitSet16>()
        });

        let cols = {
            let mut cols = [0; N];
            for (i, col) in cols.iter_mut().enumerate() {
                *col = i;
            }

            cols.map(|i| {
                self.get_col(i)
                    .into_iter()
                    .flatten()
                    .map(|i| i as usize)
                    .collect::<BitSet16>()
            })
        };

        for (i, row) in rows.iter().enumerate() {
            for (j, col) in cols.iter().enumerate() {
                if self.get_partial(i, j).is_none() && row.union(*col).len() == N - 1 {
                    return true;
                }
            }
        }

        false
    }

    pub fn minimize_rows(&self, lookup: &Vec<Vec<(Permutation<N>, Permutation<N>)>>) -> Self {
        let full_rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| row.iter().all(|v| v.is_some()))
            .map(|row| row.map(|v| v.unwrap()))
            .collect();

        let mut candidates = Vec::new();
        let mut min_cycles = vec![N];

        for [row0, row1] in TupleIterator::<2>::new(full_rows.len())
            .flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
        {
            let rows = [full_rows[row0], full_rows[row1]];
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

            if cycles < min_cycles {
                min_cycles = cycles.clone();
                candidates.clear();
            }
            if cycles == min_cycles {
                candidates.push(rows);
            }
        }

        let mut min = *self;

        for rows in candidates {
            let permutations = minimize_rows_with_lookup(&rows, lookup);

            for (s, c) in permutations {
                let mut new_sq = *self;
                new_sq.permute_cols_vals_simd(&c, &s);

                let mut new_rows = [[None; N]; N];
                for i in 0..N {
                    if let Some(j) = new_sq.rows[i][0] {
                        new_rows[j as usize] = new_sq.rows[i];
                    }
                }

                let new_sq = PartialLatinSquare::from_array(new_rows);

                if new_sq.cmp_rows(&min).is_lt() {
                    min = new_sq;
                }
            }
        }

        min
    }

    pub fn is_minimal(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> bool {
        let full_rows: Vec<_> = self
            .rows
            .iter()
            .take_while(|row| row.iter().all(|v| v.is_some()))
            .map(|row| row.map(|v| v.unwrap()))
            .collect();

        let mut candidates = Vec::new();
        let mut min_cycles = vec![N];

        for [row0, row1] in TupleIterator::<2>::new(full_rows.len())
            .flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
        {
            let rows = [full_rows[row0], full_rows[row1]];
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

            if cycles < min_cycles {
                min_cycles = cycles.clone();
                candidates.clear();
            }
            if cycles == min_cycles {
                candidates.push(rows);
            }
        }

        for rows in candidates {
            let permutations = minimize_rows_with_lookup(&rows, lookup);

            for (s, c) in permutations {
                let mut new_sq = *self;
                new_sq.permute_cols_vals_simd(&c, &s);

                let mut new_rows = [[None; N]; N];
                for i in 0..N {
                    if let Some(j) = new_sq.rows[i][0] {
                        new_rows[j as usize] = new_sq.rows[i];
                    }
                }

                let new_sq = PartialLatinSquare::from_array(new_rows);

                if new_sq.cmp_rows(self).is_lt() {
                    return false;
                }
            }
        }

        true
    }

    /// returns all permutations of rows, columns and values
    pub fn paratopic(&self) -> impl Iterator<Item = Self> + '_ {
        let mut rows = [[None; N]; N];
        for (i, row) in rows.iter_mut().enumerate() {
            *row = [Some(i); N];
        }

        let mut col = [None; N];

        for (i, val) in col.iter_mut().enumerate() {
            *val = Some(i);
        }

        let cols = [col; N];
        let vals = self.rows.map(|row| row.map(|val| val.map(|v| v as usize)));

        PermutationIter::new().map(move |perm| {
            let [rows, cols, vals] = perm.apply_array([rows, cols, vals]);
            Self::from_rcv(rows, cols, vals)
        })
    }

    pub fn minimize_main_class(&self, lookup: &Vec<Vec<(Permutation<N>, Permutation<N>)>>) -> Self {
        let mut min = *self;

        for sq in self.paratopic() {
            let sq = sq.minimize_rows(lookup);

            min = min.min(sq);
        }

        min
    }

    /// returns the smallest row cycle after completing the row cycles in the maximal way
    pub fn largest_min_row_cycle(&self) -> Vec<usize> {
        let mut min_cycle = vec![N];

        for rows in TupleIterator::<2>::new(N).map(|rows| rows.map(|row| self.get_row(row))) {
            let mut row_permutation = [None; N];

            for i in 0..N {
                let Some(position) = rows[0].iter().position(|v| *v == Some(i as u8)) else {
                    continue;
                };
                row_permutation[i] = rows[1][position].map(|v| v as usize);
            }

            let cycle = Self::largest_possible_cycle(row_permutation);

            min_cycle = min_cycle.min(cycle);
        }

        min_cycle
    }

    /// returns the smallest col cycle after completing the col cycles in the maximal way
    pub fn largest_min_col_cycle(&self) -> Vec<usize> {
        let mut min_cycle = vec![N];

        for cols in TupleIterator::<2>::new(N).map(|cols| cols.map(|row| self.get_col(row))) {
            let mut col_permutation = [None; N];

            for i in 0..N {
                let Some(position) = cols[0].iter().position(|v| *v == Some(i as u8)) else {
                    continue;
                };
                col_permutation[i] = cols[1][position].map(|v| v as usize);
            }

            let cycle = Self::largest_possible_cycle(col_permutation);

            min_cycle = min_cycle.min(cycle);
        }

        min_cycle
    }

    /// returns the smallest val cycle after completing the val cycles in the maximal way
    pub fn largest_min_val_cycle(&self) -> Vec<usize> {
        let mut min_cycle = vec![N];

        for vals in TupleIterator::<2>::new(N).map(|vals| vals.map(|val| self.get_val(val))) {
            let mut val_permutation = [None; N];

            for i in 0..N {
                let Some(position) = vals[0].iter().position(|v| *v == Some(i as u8)) else {
                    continue;
                };
                val_permutation[i] = vals[1][position].map(|v| v as usize);
            }

            let cycle = Self::largest_possible_cycle(val_permutation);

            min_cycle = min_cycle.min(cycle);
        }

        min_cycle
    }

    fn largest_possible_cycle(permutation: [Option<usize>; N]) -> Vec<usize> {
        let mut cycles = Vec::new();
        let mut unfinished_cycle_lens = [None; N];
        let mut used = [false; N];

        for start in permutation.into_iter().flatten() {
            if used[start] {
                continue;
            }

            used[start] = true;
            let mut cycle_len = 1;
            let mut last = start;
            let mut current = permutation[start];

            while current.is_some_and(|c| c != start) {
                let c = current.unwrap();

                used[c] = true;
                cycle_len += 1;
                last = c;
                current = permutation[c];
            }

            if current.is_some_and(|c| c == start) {
                cycles.push(cycle_len);
            } else {
                unfinished_cycle_lens[last] =
                    Some(cycle_len.max(unfinished_cycle_lens[last].unwrap_or(0)));
            }
        }

        let unused = used.into_iter().filter(|b| !b).count();

        if unfinished_cycle_lens.iter().flatten().next().is_none() {
            // just add all unused elements as one new cycle
            cycles.push(unused);
            cycles.sort();
            cycles
        } else {
            // we can join unfinished cycles
            cycles.push(unfinished_cycle_lens.into_iter().flatten().sum::<usize>() + unused);
            cycles.sort();
            cycles
        }
    }

    pub fn cmp_rows(&self, other: &Self) -> Ordering {
        for i in 0..N {
            for j in 0..N {
                match (self.rows[i][j], other.rows[i][j]) {
                    (None, None) => {}
                    (Some(_), None) => return Ordering::Less,
                    (None, Some(_)) => return Ordering::Greater,
                    (Some(i), Some(j)) => match i.cmp(&j) {
                        Ordering::Equal => {}
                        o => return o,
                    },
                }
            }
        }
        Ordering::Equal
    }

    pub fn cmp_diagonal(&self, other: &Self) -> Ordering {
        for i in 0..N {
            for j in (0..=i).rev() {
                match self.rows[j][i].cmp(&other.rows[j][i]) {
                    Ordering::Equal => {}
                    o => return o,
                }
                if i != j {
                    match self.rows[i][j].cmp(&other.rows[i][j]) {
                        Ordering::Equal => {}
                        o => return o,
                    }
                }
            }
        }

        Ordering::Equal
    }
}

impl<const N: usize> PartialOrd for PartialLatinSquare<N> {
    fn partial_cmp(&self, other: &PartialLatinSquare<N>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for PartialLatinSquare<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_rows(other)
    }
}

impl<const N: usize> From<LatinSquare<N>> for PartialLatinSquare<N> {
    fn from(value: LatinSquare<N>) -> Self {
        let mut sq = PartialLatinSquare {
            rows: [[None; N]; N],
        };

        for i in 0..N {
            for j in 0..N {
                sq.rows[i][j] = Some(value.get(i, j) as u8);
            }
        }

        sq
    }
}

impl<const N: usize> Display for PartialLatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..N {
            for j in 0..N {
                if let Some(entry) = self.get_partial(i, j) {
                    f.write_char(char::from_digit(entry as u32, 10).unwrap())?;
                } else {
                    f.write_char('.')?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidLength { len: usize, expected: usize },
    InvalidChar { index: usize, char: char },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength { len, expected } => {
                write!(f, "Invalid len: {len}, expected {expected}")
            }
            Error::InvalidChar { index, char } => {
                write!(f, "Invalid char at index {index}: {char}")
            }
        }
    }
}

impl<const N: usize> TryFrom<&str> for PartialLatinSquare<N> {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != N * N {
            return Err(Error::InvalidLength {
                len: value.len(),
                expected: N * N,
            });
        }

        let mut values = [[None; N]; N];
        for (i, c) in value.chars().enumerate() {
            if c != '.' {
                let entry = c
                    .to_digit(10)
                    .ok_or(Error::InvalidChar { index: i, char: c })?;
                if entry >= N as u32 {
                    return Err(Error::InvalidChar { index: i, char: c });
                }
                values[i / N][i % N] = Some(entry as u8);
            }
        }

        Ok(PartialLatinSquare { rows: values })
    }
}

impl<const N: usize> Debug for PartialLatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for i in 0..N {
            writeln!(f, "+{}", "---+".repeat(N))?;
            write!(f, "|")?;
            for j in 0..N {
                if let Some(value) = self.get_partial(i, j) {
                    write!(f, " {} |", value)?;
                } else {
                    write!(f, "   |")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "+{}", "---+".repeat(N))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn permute_rows_and_cols() {
        let sq = PartialLatinSquare::<3>::try_from("012120201").unwrap();

        let perm = Permutation::from_array([2, 1, 0]);

        assert_eq!(
            sq.permute_rows(&perm).permute_cols(&perm),
            sq.permute_rows_and_cols(&perm)
        );
    }

    // #[test]
    // fn minimize_rows() {
    //     assert_eq!(
    //         PartialLatinSquare::from_array([
    //             [Some(0), Some(1), Some(2), Some(3)],
    //             [Some(1), Some(2), Some(3), Some(0)],
    //             [Some(2), Some(3), Some(0), Some(1)],
    //             [None; 4]
    //         ])
    //         .minimize_rows(),
    //         PartialLatinSquare::from_array([
    //             [Some(0), Some(1), Some(2), Some(3)],
    //             [Some(1), Some(0), Some(3), Some(2)],
    //             [Some(2), Some(3), Some(1), Some(0)],
    //             [None; 4]
    //         ])
    //     )
    // }

    #[test]
    fn largest_possible_col_cycle() {
        assert_eq!(
            PartialLatinSquare::from_array([
                [Some(0), Some(1), Some(2), Some(3)],
                [Some(1), Some(2), Some(3), Some(0)],
                [Some(2), Some(3), Some(0), Some(1)],
                [None; 4]
            ])
            .largest_min_col_cycle(),
            vec![4]
        );
        assert_eq!(
            PartialLatinSquare::from_array([
                [Some(0), Some(1), None, None],
                [Some(1), Some(0), None, None],
                [None, None, Some(0), Some(1)],
                [None, None, Some(1), Some(0)],
            ])
            .largest_min_col_cycle(),
            vec![4]
        );
    }
}
