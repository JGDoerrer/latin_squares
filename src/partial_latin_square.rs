use core::fmt::Debug;
use std::{cmp::Ordering, ffi::FromVecWithNulError};

use crate::{bitset::BitSet16, latin_square::LatinSquare, permutation::Permutation};

#[derive(Clone, Copy, PartialEq, Eq, Ord)]
pub struct PartialLatinSquare<const N: usize> {
    values: [[Option<u8>; N]; N],
}

impl<const N: usize> Default for PartialLatinSquare<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PartialLatinSquare<N> {
    pub fn new() -> Self {
        PartialLatinSquare {
            values: [[None; N]; N],
        }
    }

    pub fn from_array(values: [[Option<u8>; N]; N]) -> Self {
        PartialLatinSquare { values }
    }

    pub fn get(&self, i: usize, j: usize) -> Option<usize> {
        self.values[i][j].map(|val| val.into())
    }

    pub fn get_row(&self, i: usize) -> &[Option<u8>; N] {
        &self.values[i]
    }

    pub fn get_col(&self, i: usize) -> [Option<u8>; N] {
        let mut col = [None; N];

        for j in 0..N {
            col[j] = self.values[j][i];
        }

        col
    }

    pub fn set(&mut self, i: usize, j: usize, value: Option<usize>) {
        self.values[i][j] = value.map(|v| v as u8);
    }

    pub fn is_reduced(&self) -> bool {
        for i in 0..N {
            if self.values[0][i].is_some_and(|j| j != i as u8)
                || self.values[i][0].is_some_and(|j| j != i as u8)
            {
                return false;
            }
        }
        true
    }

    pub fn is_valid(&self) -> bool {
        (0..N).all(|i| {
            (0..N)
                .map(|j| self.get(i, j))
                .flatten()
                .collect::<BitSet16>()
                .len()
                == (0..N).map(|j| self.get(i, j)).flatten().count()
                && (0..N)
                    .map(|j| self.get(j, i))
                    .flatten()
                    .collect::<BitSet16>()
                    .len()
                    == (0..N).map(|j| self.get(j, i)).flatten().count()
        })
    }

    pub fn transposed(&self) -> Self {
        let mut values = [[None; N]; N];

        for i in 0..N {
            for j in 0..N {
                values[i][j] = self.values[j][i];
            }
        }

        Self { values }
    }

    pub fn next_unknown(&self) -> Option<(usize, usize)> {
        for j in 0..(N + 1) / 2 {
            for j in [j, N - j - 1] {
                for i in 0..N {
                    if self.get(j, i).is_none() {
                        return Some((j, i));
                    }
                }
                for i in 0..N {
                    if self.get(i, j).is_none() {
                        return Some((i, j));
                    }
                }
            }
        }
        None
    }

    pub fn num_entries(&self) -> usize {
        (0..N)
            .map(|row| (0..N).filter(|col| self.get(row, *col).is_some()).count())
            .sum()
    }

    pub fn num_empty_rows(&self) -> usize {
        (0..N)
            .filter(|row| (0..N).all(|col| self.get(*row, col).is_none()))
            .count()
    }

    pub fn num_full_rows(&self) -> usize {
        (0..N)
            .filter(|row| (0..N).all(|col| self.get(*row, col).is_some()))
            .count()
    }

    pub fn num_empty_cols(&self) -> usize {
        (0..N)
            .filter(|col| (0..N).all(|row| self.get(row, *col).is_none()))
            .count()
    }

    pub fn num_full_cols(&self) -> usize {
        (0..N)
            .filter(|col| (0..N).all(|row| self.get(row, *col).is_some()))
            .count()
    }

    pub fn num_unique_values(&self) -> usize {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(row, col)))
            .flatten()
            .collect::<BitSet16>()
            .len()
    }

    pub fn count_val(&self, value: usize) -> usize {
        (0..N)
            .flat_map(|col| {
                (0..N).filter(move |row| self.get(*row, col).is_some_and(|i| i == value))
            })
            .count()
    }

    pub fn unique_entries(&self) -> BitSet16 {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(row, col)))
            .flatten()
            .collect::<BitSet16>()
    }

    pub fn first_empty_index(&self) -> Option<usize> {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(row, col)))
            .position(|entry| entry.is_none())
    }

    pub fn next_empty_index(&self, start: usize) -> Option<usize> {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(row, col)))
            .skip(start)
            .position(|entry| entry.is_none())
            .map(|index| index + start)
    }

    pub fn num_next_empty_indices(&self, start: usize) -> usize {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(row, col)))
            .skip(start)
            .filter(|entry| entry.is_none())
            .count()
    }

    fn reduced_subsquare(&self, k: usize) -> Self {
        let first_row = *self.get_row(0);
        let max_value = self.num_unique_values() - 1;
        let min_value = *first_row.iter().flatten().min().unwrap();

        let mut permutation = Permutation::identity().to_array();
        for i in 0..=max_value {
            permutation[i] = (i + max_value + 1 - min_value as usize) % (max_value + 1);
        }
        let permutation: Permutation<N> = permutation.into();

        let mut new = self.permute_vals(permutation);
        let mut first_row = *new.get_row(0);

        for i in k..N {
            first_row[i] = (0..N)
                .find(|i| !first_row.contains(&Some(*i as u8)))
                .map(|i| i as u8);
        }

        for i in 0..k {
            let a = first_row
                .iter()
                .skip(i)
                .position(|e| e.is_some_and(|e| e as usize == i));

            if let Some(a) = a {
                first_row.swap(i, a);
            }
        }

        let mut permutation = Permutation::<N>::identity().to_array();

        for i in 0..N {
            permutation[i] = first_row[i].unwrap() as usize;
        }

        let permutation = Permutation::from_array(permutation).inverse();

        new = new.permute_vals(permutation);

        let mut first_row = *new.get_row(0);

        for i in k..N {
            first_row[i] = (0..N)
                .find(|i| !first_row.contains(&Some(*i as u8)))
                .map(|i| i as u8);
        }

        let mut permutation = Permutation::<N>::identity().to_array();

        for i in 0..N {
            permutation[i] = first_row[i].unwrap() as usize;
        }

        new = new.permute_cols(permutation.into());

        new.values.sort_by_key(|i| i.map(|j| j.unwrap_or(u8::MAX)));

        debug_assert!(new.is_valid());

        new
    }

    fn all_reduced_subsquares(&self, k: usize) -> impl Iterator<Item = Self> + '_ {
        (0..k).map(move |i| {
            let mut new_values = self.values;

            new_values.swap(0, i);

            let new = Self { values: new_values }.reduced_subsquare(k);
            new
        })
    }

    pub fn is_minimal_subsquare(&self, k: usize) -> bool {
        let unique_entries = self.unique_entries();

        if unique_entries.into_iter().last().unwrap() != unique_entries.len() - 1 {
            return false;
        }

        let mut min = *self;

        // for sq in self.all_reduced_subsquares(k) {
        //     min = min.min(sq);
        //     min = min.min(sq.transposed());
        // }

        min == *self
    }

    pub fn is_minimal_diagonal(&self, k: usize) -> bool {
        let unique_entries = self.unique_entries();

        if unique_entries.into_iter().last().unwrap() != unique_entries.len() - 1 {
            return false;
        }

        let mut min = *self;

        // for sq in self.all_reduced_subsquares(k) {
        //     min = min.min(sq);
        //     min = min.min(sq.transposed());
        // }

        min == *self
    }

    pub fn permute_rows(&self, permutation: Permutation<N>) -> Self {
        let values = permutation.apply_array(self.values);

        Self { values }
    }

    pub fn permute_cols(&self, permutation: Permutation<N>) -> Self {
        let mut values = self.values;

        values.iter_mut().for_each(|row| {
            *row = permutation.apply_array(*row);
        });

        Self { values }
    }

    pub fn permute_vals(&self, permutation: Permutation<N>) -> Self {
        let mut values = self.values;

        for row in &mut values {
            for val in row {
                if let Some(val) = val.as_mut() {
                    *val = permutation.apply(*val as usize) as u8;
                }
            }
        }

        Self { values }
    }
}

impl<const N: usize> PartialOrd for PartialLatinSquare<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for i in 0..N {
            match self.values[i][0].cmp(&other.values[i][0]) {
                Ordering::Less => return Some(Ordering::Less),
                Ordering::Greater => return Some(Ordering::Greater),
                Ordering::Equal => {}
            }
            match self.values[0][i].cmp(&other.values[0][i]) {
                Ordering::Less => return Some(Ordering::Less),
                Ordering::Greater => return Some(Ordering::Greater),
                Ordering::Equal => {}
            }
        }

        for i in 1..N {
            for j in 1..=i {
                match self.values[j][i].cmp(&other.values[j][i]) {
                    Ordering::Less => return Some(Ordering::Less),
                    Ordering::Greater => return Some(Ordering::Greater),
                    Ordering::Equal => {}
                }
            }
            for j in 1..i {
                match self.values[i][j].cmp(&other.values[i][j]) {
                    Ordering::Less => return Some(Ordering::Less),
                    Ordering::Greater => return Some(Ordering::Greater),
                    Ordering::Equal => {}
                }
            }
        }

        Some(Ordering::Equal)
    }
}

impl<const N: usize> From<LatinSquare<N>> for PartialLatinSquare<N> {
    fn from(value: LatinSquare<N>) -> Self {
        let mut sq = PartialLatinSquare {
            values: [[None; N]; N],
        };

        for i in 0..N {
            for j in 0..N {
                sq.values[i][j] = Some(value.get(i, j) as u8);
            }
        }

        sq
    }
}

impl<const N: usize> ToString for PartialLatinSquare<N> {
    fn to_string(&self) -> String {
        let mut string = String::with_capacity(N * N);
        for i in 0..N {
            for j in 0..N {
                if let Some(entry) = self.get(i, j) {
                    string.push(char::from_digit(entry as u32, 10).unwrap());
                } else {
                    string.push('.');
                }
            }
        }
        string
    }
}

impl<const N: usize> TryFrom<&str> for PartialLatinSquare<N> {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != N * N {
            return Err("Invalid length");
        }

        let mut values = [[None; N]; N];
        for (i, c) in value.chars().enumerate() {
            if c != '.' {
                let entry = c.to_digit(10).ok_or("Invalid digit")?;
                if entry >= N as u32 {
                    return Err("Invalid digit");
                }
                values[i / N][i % N] = Some(entry as u8);
            }
        }

        Ok(PartialLatinSquare { values })
    }
}

impl<const N: usize> Debug for PartialLatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for i in 0..N {
            write!(f, "    [")?;
            for j in 0..N {
                if let Some(value) = self.get(i, j) {
                    write!(f, "{:2}, ", value)?;
                } else {
                    write!(f, "??, ")?;
                }
            }
            write!(f, "]")?;
            if i != N - 1 {
                writeln!(f, ",")?;
            }
        }
        write!(f, "\n]")?;
        Ok(())
    }
}
