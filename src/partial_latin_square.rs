use core::fmt::Debug;
use std::{
    cmp::Ordering,
    fmt::{Display, Write},
};

use crate::{bitset::BitSet16, latin_square::LatinSquare};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartialLatinSquare<const N: usize> {
    rows: [[Option<u8>; N]; N],
}

impl<const N: usize> Default for PartialLatinSquare<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const N: usize> PartialLatinSquare<N> {
    pub const fn empty() -> Self {
        PartialLatinSquare {
            rows: [[None; N]; N],
        }
    }

    pub fn get_partial(&self, row: usize, col: usize) -> Option<usize> {
        self.rows[row][col].map(|val| val.into())
    }

    pub fn from_array(values: [[Option<u8>; N]; N]) -> Self {
        PartialLatinSquare { rows: values }
    }

    pub fn values(self) -> [[Option<u8>; N]; N] {
        self.rows
    }

    pub fn get(&self, row: usize, col: usize) -> Option<usize> {
        self.rows[row][col].map(|val| val.into())
    }

    pub fn set(&mut self, i: usize, j: usize, value: Option<usize>) {
        self.rows[i][j] = value.map(|v| v as u8);
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

    pub fn num_entries(&self) -> usize {
        self.rows
            .iter()
            .map(|row| row.iter().flatten().count())
            .sum()
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
                    f.write_char(char::from_digit(entry as u32, 16).unwrap())?;
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
                    .to_digit(16)
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
mod test {}
