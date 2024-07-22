use std::{
    fmt::{Debug, Display},
    mem::MaybeUninit,
};

use crate::{
    bitset::BitSet128,
    latin_square::{self, Cell, LatinSquare},
    latin_square_oa_generator::LatinSquareOAGenerator,
    latin_square_trait::{LatinSquareTrait, MOLSTrait, PartialMOLSTrait},
    partial_orthogonal_array::PartialOrthogonalArray,
    permutation::Permutation,
    tuple_iterator::TupleIterator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValuePair(pub usize, pub usize);

impl ValuePair {
    pub fn from_index<const N: usize>(index: usize) -> Self {
        ValuePair(index % N, index / N)
    }

    pub fn to_index<const N: usize>(self) -> usize {
        self.0 + self.1 * N
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrthogonalArray<const N: usize, const MOLS: usize> {
    sqs: [LatinSquare<N>; MOLS],
}

impl<const N: usize, const MOLS: usize> PartialMOLSTrait for OrthogonalArray<N, MOLS> {
    fn n(&self) -> usize {
        N
    }

    fn mols(&self) -> usize {
        MOLS
    }

    fn partial_squares(&self) -> &[impl crate::latin_square_trait::PartialLatinSquareTrait] {
        self.sqs.as_slice()
    }
}

impl<const N: usize, const MOLS: usize> MOLSTrait for OrthogonalArray<N, MOLS> {
    fn squares(&self) -> &[impl crate::latin_square_trait::LatinSquareTrait] {
        self.sqs.as_slice()
    }
}

impl<const N: usize, const MOLS: usize> OrthogonalArray<N, MOLS> {
    pub fn new(sqs: [LatinSquare<N>; MOLS]) -> Self {
        OrthogonalArray {
            sqs: sqs.map(|sq| sq.into()),
        }
    }

    pub fn squares(&self) -> [LatinSquare<N>; MOLS] {
        self.sqs
    }

    pub fn unavoidable_sets(&self) -> Vec<Vec<BitSet128>> {
        vec![self.unavoidable_sets_order_1()]
    }

    pub fn unavoidable_sets_order_1(&self) -> Vec<BitSet128> {
        let mut sets = Vec::new();
        let max_size = N * 4 * MOLS;

        let triple_iter = TupleIterator::<4>::new(N);

        for triple in triple_iter {
            for partial in [self.without_rows(&triple), self.without_cols(&triple)]
                .into_iter()
                .chain((0..MOLS).map(|i| self.without_vals(i, &triple)))
            {
                let solutions = LatinSquareOAGenerator::<N, MOLS>::from_partial_oa(&partial);

                for solution in solutions {
                    let difference = self.difference_mask(&solution);

                    if !difference.is_empty()
                        && difference.len() <= max_size
                        && !sets.contains(&difference)
                    {
                        sets.push(difference);
                        // if sets.len() > 10000 {
                        //     max_size -= 1;
                        //     sets.retain(|s| s.len() <= max_size);
                        // }
                    }
                }
            }
        }

        sets
    }

    pub fn mask(&self, mask: BitSet128) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial_oa = PartialOrthogonalArray::empty();

        for i in mask {
            let col = i / (N * N);
            let Cell(i, j) = Cell::from_index::<N>(i % (N * N));

            partial_oa.set(col, i, j, Some(self.get(col, i, j)));
        }

        partial_oa
    }

    pub fn get(&self, column: usize, i: usize, j: usize) -> u8 {
        self.sqs[column].get(i, j) as u8
    }

    fn without_rows(&self, rows: &[usize]) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial = PartialOrthogonalArray::from(self.clone());

        for column in 0..MOLS {
            for i in rows {
                for j in 0..N {
                    partial.set(column, *i, j, None);
                }
            }
        }

        partial
    }

    fn without_cols(&self, cols: &[usize]) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial = PartialOrthogonalArray::from(self.clone());

        for column in 0..MOLS {
            for i in 0..N {
                for j in cols {
                    partial.set(column, i, *j, None);
                }
            }
        }

        partial
    }

    fn without_vals(&self, index: usize, values: &[usize]) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial = PartialOrthogonalArray::from(self.clone());

        for i in 0..N {
            for j in 0..N {
                if values.contains(&(self.get(index, i, j) as usize)) {
                    for column in 0..MOLS {
                        partial.set(column, i, j, None);
                    }
                }
            }
        }

        partial
    }

    fn difference_mask(&self, other: &OrthogonalArray<N, MOLS>) -> BitSet128 {
        let mut mask = BitSet128::empty();

        for col in 0..MOLS {
            for i in 0..N {
                for j in 0..N {
                    if self.get(col, i, j) != other.get(col, i, j) {
                        let index = col * N * N + Cell(i, j).to_index::<N>();
                        assert!(index < 128);
                        mask.insert(index);
                    }
                }
            }
        }

        mask
    }

    pub fn permute_rows(&self, permutation: &Permutation<N>) -> Self {
        let mut new = self.clone();

        for i in 0..MOLS {
            new.sqs[i] = new.sqs[i].permute_rows(permutation);
        }

        new
    }
}

impl<const N: usize, const MOLS: usize> Display for OrthogonalArray<N, MOLS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.squares().map(|sq| sq.to_string()).join("-"))
    }
}

pub const SEPARATOR: char = '-';

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
    InvalidLatinSquare {
        index: usize,
        error: latin_square::Error,
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
            Error::InvalidLatinSquare { index, error } => {
                write!(f, "Error in latin square {}: {error}", index + 1)
            }
        }
    }
}

impl<const N: usize, const MOLS: usize> TryFrom<&str> for OrthogonalArray<N, MOLS> {
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

        let sqs: Vec<Result<LatinSquare<N>, Error>> = value
            .split(SEPARATOR)
            .enumerate()
            .map(|(i, split)| {
                LatinSquare::try_from(split)
                    .map_err(|error| Error::InvalidLatinSquare { index: i, error })
            })
            .collect();

        let mut sqs_array = [MaybeUninit::uninit(); MOLS];
        for (i, sq) in sqs.into_iter().enumerate() {
            sqs_array[i].write(sq?);
        }

        let sqs = sqs_array.map(|sq| unsafe { sq.assume_init() });

        let oa = OrthogonalArray::new(sqs);
        Ok(oa)
    }
}
