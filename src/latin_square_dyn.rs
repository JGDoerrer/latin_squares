use std::fmt::{Display, Write};

use crate::{
    bitset::{BitSet128, BitSet16},
    latin_square::LatinSquare,
    latin_square_generator::LatinSquareGeneratorDyn,
    latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait},
    partial_latin_square_dyn::PartialLatinSquareDyn,
    tuple_iterator::TupleIterator,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LatinSquareDyn {
    n: usize,
    values: Box<[u8]>,
}

impl PartialLatinSquareTrait for LatinSquareDyn {
    fn n(&self) -> usize {
        self.n
    }

    fn get_partial(&self, row: usize, col: usize) -> Option<usize> {
        Some(self.values[row * self.n() + col].into())
    }
}

impl LatinSquareTrait for LatinSquareDyn {
    fn get(&self, row: usize, col: usize) -> usize {
        self.values[row * self.n() + col].into()
    }
}

impl LatinSquareDyn {
    pub fn from_boxed_slice(values: Box<[u8]>) -> Option<LatinSquareDyn> {
        if !Self::is_valid(&values) {
            return None;
        }

        let n = isqrt(values.len()).unwrap();

        Some(LatinSquareDyn { n, values })
    }

    pub fn values(&self) -> &[u8] {
        &self.values
    }

    fn is_valid(values: &[u8]) -> bool {
        let Some(n) = isqrt(values.len()) else {
            return false;
        };

        (0..n).all(|i| {
            (0..n)
                .map(|j| values[i * n + j] as usize)
                .collect::<BitSet16>()
                == BitSet16::all_less_than(n)
                && (0..n)
                    .map(|j| values[j * n + i] as usize)
                    .collect::<BitSet16>()
                    == BitSet16::all_less_than(n)
        })
    }

    pub fn differences(&self) -> Vec<BitSet128> {
        let mut sets: Vec<BitSet128> = Vec::new();

        for tuple in TupleIterator::<3>::new(self.n) {
            for partial in [
                self.without_rows(&tuple),
                self.without_cols(&tuple),
                self.without_vals(&tuple),
            ] {
                let solutions = LatinSquareGeneratorDyn::from_partial_sq(&partial);

                for solution in solutions {
                    let difference = self.difference_mask(&solution);

                    if !difference.is_empty() && !sets.iter().any(|s| s.is_subset_of(difference)) {
                        sets.retain(|s| !difference.is_subset_of(*s));
                        sets.push(difference);
                    }
                }
            }
        }

        sets.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        sets.dedup();

        sets
    }

    pub fn without_rows(&self, rows: &[usize]) -> PartialLatinSquareDyn {
        let mut sq = PartialLatinSquareDyn::from(self);
        for row in rows {
            for i in 0..self.n {
                sq.set(*row, i, None);
            }
        }
        sq
    }

    pub fn without_cols(&self, cols: &[usize]) -> PartialLatinSquareDyn {
        let mut sq = PartialLatinSquareDyn::from(self);
        for col in cols {
            for i in 0..self.n {
                sq.set(i, *col, None);
            }
        }
        sq
    }

    pub fn without_vals(&self, vals: &[usize]) -> PartialLatinSquareDyn {
        let mut sq = PartialLatinSquareDyn::from(self);
        for value in vals {
            for i in 0..self.n {
                for j in 0..self.n {
                    if self.get(i, j) == *value {
                        sq.set(i, j, None);
                    }
                }
            }
        }
        sq
    }

    pub fn difference_mask(&self, other: &Self) -> BitSet128 {
        let mut mask = BitSet128::empty();

        assert_eq!(self.n, other.n);
        let n = self.n;

        assert!(n * n <= 128);

        for i in 0..n {
            for j in 0..n {
                if self.get(i, j) != other.get(i, j) {
                    mask.insert(i * n + j);
                }
            }
        }

        mask
    }

    pub fn mask(&self, mask: BitSet128) -> PartialLatinSquareDyn {
        let mut partial_sq = PartialLatinSquareDyn::empty(self.n);

        for index in mask {
            let i = index / self.n;
            let j = index % self.n;

            partial_sq.set(i, j, Some(self.get(i, j)));
        }

        partial_sq
    }
}

impl<const N: usize> From<LatinSquare<N>> for LatinSquareDyn {
    fn from(sq: LatinSquare<N>) -> Self {
        let values = sq
            .to_values()
            .into_iter()
            .flat_map(|row| row.into_iter())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        LatinSquareDyn { n: N, values }
    }
}

impl Display for LatinSquareDyn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.n();
        for i in 0..n {
            for j in 0..n {
                f.write_char(char::from_digit(self.get(i, j) as u32, 10).unwrap())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidLength { len: usize },
    InvalidChar { index: usize, char: char },
    InvalidLatinSquare,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength { len } => {
                write!(f, "Invalid len: {len}, expected a square number")
            }
            Error::InvalidChar { index, char } => {
                write!(f, "Invalid char at index {index}: {char}")
            }
            Error::InvalidLatinSquare => write!(f, "The latin square property is not met"),
        }
    }
}

impl TryFrom<&str> for LatinSquareDyn {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Some(n) = isqrt(value.len()) else {
            return Err(Error::InvalidLength { len: value.len() });
        };

        let mut values = vec![0; value.len()].into_boxed_slice();
        for (i, c) in value.chars().enumerate() {
            let entry = c
                .to_digit(10)
                .ok_or(Error::InvalidChar { index: i, char: c })?;
            if entry >= n as u32 {
                return Err(Error::InvalidChar { index: i, char: c });
            }
            values[i] = entry as u8;
        }

        LatinSquareDyn::from_boxed_slice(values).ok_or(Error::InvalidLatinSquare)
    }
}

impl TryFrom<PartialLatinSquareDyn> for LatinSquareDyn {
    type Error = ();

    fn try_from(value: PartialLatinSquareDyn) -> Result<Self, ()> {
        let n = value.n();
        let mut sq = LatinSquareDyn {
            n,
            values: vec![0; n * n].into_boxed_slice(),
        };

        for i in 0..n {
            for j in 0..n {
                sq.values[i * n + j] = value.get_partial(i, j).unwrap() as u8;
            }
        }

        Ok(sq)
    }
}

pub fn isqrt(n: usize) -> Option<usize> {
    for i in 0.. {
        if i * i == n {
            return Some(i);
        }
        if i * i > n {
            return None;
        }
    }
    unreachable!()
}
