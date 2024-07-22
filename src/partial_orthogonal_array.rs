use std::{fmt::Display, mem::MaybeUninit};

use crate::{
    orthogonal_array::{OrthogonalArray, SEPARATOR},
    partial_latin_square::{self, PartialLatinSquare},
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PartialOrthogonalArray<const N: usize, const MOLS: usize> {
    columns: [[[Option<u8>; N]; N]; MOLS],
}

impl<const N: usize, const MOLS: usize> PartialOrthogonalArray<N, MOLS> {
    pub fn empty() -> Self {
        PartialOrthogonalArray {
            columns: [[[None; N]; N]; MOLS],
        }
    }

    pub fn new(sqs: [PartialLatinSquare<N>; MOLS]) -> Self {
        PartialOrthogonalArray {
            columns: sqs.map(|sq| sq.values()),
        }
    }

    pub fn squares(&self) -> [PartialLatinSquare<N>; MOLS] {
        self.columns.map(|col| {
            let mut new_col = [[None; N]; N];

            for i in 0..N {
                for j in 0..N {
                    new_col[i][j] = col[i][j];
                }
            }

            PartialLatinSquare::from_array(new_col)
        })
    }

    pub fn get(&self, column: usize, i: usize, j: usize) -> Option<u8> {
        self.columns[column][i][j]
    }

    pub fn num_entries(&self) -> usize {
        self.columns
            .iter()
            .map(|col| {
                col.iter()
                    .map(|row| row.iter().flatten().count())
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn next_empty_index(&self, start: usize) -> Option<usize> {
        self.columns
            .iter()
            .flat_map(|col| col.iter().flat_map(|row| row.iter()))
            .skip(start)
            .position(|i| i.is_none())
            .map(|i| i + start)
    }

    pub fn first_empty_index(&self) -> Option<usize> {
        self.next_empty_index(0)
    }

    pub fn set(&mut self, column: usize, i: usize, j: usize, value: Option<u8>) {
        self.columns[column][i][j] = value
    }
}

impl<const N: usize, const MOLS: usize> From<OrthogonalArray<N, MOLS>>
    for PartialOrthogonalArray<N, MOLS>
{
    fn from(value: OrthogonalArray<N, MOLS>) -> Self {
        let sqs = value.squares().map(|sq| sq.into());
        PartialOrthogonalArray::new(sqs)
    }
}

impl<const N: usize, const MOLS: usize> Display for PartialOrthogonalArray<N, MOLS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.squares().map(|sq| sq.to_string()).join("-"))
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidLength {
        len: usize,
        expected: usize,
    },
    InvalidSeparators {
        count: usize,
        expected: usize,
    },
    InvalidPartialLatinSquare {
        index: usize,
        error: partial_latin_square::Error,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength { len, expected } => {
                write!(f, "Invalid len: {len}, expected {expected}")
            }
            Error::InvalidSeparators { count, expected } => write!(
                f,
                "Invalid number of separators ({SEPARATOR}): {count}, expected {expected}"
            ),
            Error::InvalidPartialLatinSquare { index, error } => {
                write!(f, "Error in latin square {}: {error}", index + 1)
            }
        }
    }
}

impl<const N: usize, const MOLS: usize> TryFrom<&str> for PartialOrthogonalArray<N, MOLS> {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != N * N * MOLS + MOLS - 1 {
            return Err(Error::InvalidLength {
                len: value.len(),
                expected: N * N * MOLS + MOLS - 1,
            });
        }

        let separator_count = value.chars().filter(|c| *c == SEPARATOR).count();
        if separator_count != MOLS - 1 {
            return Err(Error::InvalidSeparators {
                count: separator_count,
                expected: MOLS - 1,
            });
        }

        let sqs: Vec<Result<PartialLatinSquare<N>, Error>> = value
            .split(SEPARATOR)
            .enumerate()
            .map(|(i, split)| {
                PartialLatinSquare::try_from(split)
                    .map_err(|error| Error::InvalidPartialLatinSquare { index: i, error })
            })
            .collect();

        let mut sqs_array = [MaybeUninit::uninit(); MOLS];
        for (i, sq) in sqs.into_iter().enumerate() {
            sqs_array[i].write(sq?);
        }

        let sqs = sqs_array.map(|sq| unsafe { sq.assume_init() });

        let oa = PartialOrthogonalArray::new(sqs);
        Ok(oa)
    }
}
