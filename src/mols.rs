use std::{cmp::Ordering, fmt::Display};

use crate::{
    latin_square::{self, LatinSquare},
    permutation::{Permutation, PermutationIter},
    permutation_dyn::{PermutationDyn, PermutationDynIter},
    tuple_iterator::TupleIterator,
    Mode,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MOLS<const N: usize> {
    sqs: Vec<LatinSquare<N>>,
}

impl<const N: usize> MOLS<N> {
    pub fn new(sqs: &[LatinSquare<N>]) -> Result<Self, (usize, usize)> {
        for i in 0..sqs.len() {
            for j in (i + 1)..sqs.len() {
                if !sqs[i].is_orthogonal_to(&sqs[j]) {
                    return Err((i, j));
                }
            }
        }

        Ok(MOLS { sqs: sqs.to_vec() })
    }

    pub fn new_unchecked(sqs: &[LatinSquare<N>]) -> Self {
        MOLS { sqs: sqs.to_vec() }
    }

    pub fn sqs(&self) -> &[LatinSquare<N>] {
        &self.sqs
    }

    pub fn normalize_main_class_set(
        &self,
        lookup: &[Vec<(Permutation<N>, Permutation<N>)>],
    ) -> Self {
        let mut rows = [[0; N]; N];
        for (i, row) in rows.iter_mut().enumerate() {
            *row = [i as u8; N];
        }

        let mut col = [0; N];

        for (i, val) in col.iter_mut().enumerate() {
            *val = i as u8;
        }

        let cols = [col; N];

        let values: Vec<_> = [rows, cols]
            .into_iter()
            .chain(self.sqs.iter().map(|sq| sq.clone().to_values()))
            .map(|v| v.map(|v| v.map(|v| v as usize)))
            .collect();

        let mut min_sq = LatinSquare::z();
        let mut min_perms = vec![(
            [0, 1, 2],
            vec![[
                Permutation::identity(),
                Permutation::identity(),
                Permutation::identity(),
            ]],
        )];

        for [r, c, s] in TupleIterator::<3>::new(values.len())
            .flat_map(|rcs| PermutationIter::new().map(move |p| p.apply_array(rcs)))
        {
            let (sq, permutations) = LatinSquare::from_rcs(values[r], values[c], values[s])
                .isotopy_class_permutations(lookup);

            match sq.cmp(&min_sq) {
                Ordering::Less => {
                    min_sq = sq;
                    min_perms = vec![([r, c, s], permutations)];
                }
                Ordering::Equal => min_perms.push(([r, c, s], permutations)),
                Ordering::Greater => {}
            }
        }

        debug_assert!(min_sq == min_sq.main_class());

        let mut min_mols = self.clone();
        for (rcs, perms) in min_perms {
            let rows = values[rcs[0]];
            let cols = values[rcs[1]];

            for perm in perms {
                let mut sqs = Vec::new();

                for (i, vals) in values.iter().enumerate() {
                    if i == rcs[0] || i == rcs[1] {
                        continue;
                    }

                    let sq = LatinSquare::from_rcs(rows, cols, *vals)
                        .permuted_rows(&perm[0])
                        .permuted_cols(&perm[1]);

                    sqs.push(sq);
                }

                let mut mols = MOLS { sqs };
                mols.reduce_all_sqs();
                mols.sqs.sort();

                if mols < min_mols {
                    min_mols = mols;
                }
            }
        }

        min_mols
    }

    /// No swapping squares
    pub fn normalize_isotopy_class(&mut self) {
        let (_, permutation) = self.sqs[0].isotopy_class_permutation();

        self.permute_rows(&permutation[0]);
        self.permute_cols(&permutation[1]);
        self.reduce_all_sqs();
    }

    pub fn permute_rows(&mut self, permutation: &Permutation<N>) {
        for sq in self.sqs.iter_mut() {
            sq.permuted_rows(&permutation);
        }
    }

    pub fn permute_cols(&mut self, permutation: &Permutation<N>) {
        for sq in self.sqs.iter_mut() {
            sq.permuted_cols(&permutation);
        }
    }

    pub fn permute_sqs(&mut self, permutation: &PermutationDyn) {
        self.sqs = permutation.apply_vec(self.sqs.clone());
    }

    fn reduce_all_sqs(&mut self) {
        for sq in self.sqs.iter_mut() {
            let first_row = sq.get_row(0);
            let symbol_permutation =
                Permutation::from_array(first_row.map(|s| s as usize)).inverse();
            *sq = sq.permuted_vals(&symbol_permutation);
        }
    }
}

impl<const N: usize> Display for MOLS<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.sqs
                .iter()
                .map(|sq| sq.to_string())
                .collect::<Vec<_>>()
                .join("-")
        )
    }
}

impl<const N: usize> Ord for MOLS<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.sqs.len(), other.sqs.len());

        for (a, b) in self.sqs.iter().zip(&other.sqs) {
            match a.cmp_rows(&b) {
                Ordering::Equal => {}
                o => return o,
            }
        }

        Ordering::Equal
    }
}

impl<const N: usize> PartialOrd for MOLS<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    NotOrthogonal {
        indices: (usize, usize),
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
            Error::NotOrthogonal { indices } => {
                write!(
                    f,
                    "Squares {} and {} are not orthogonal",
                    indices.0, indices.1
                )
            }
        }
    }
}

impl<const N: usize> TryFrom<&str> for MOLS<N> {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let separator_count = value.chars().filter(|c| *c == SEPARATOR).count();
        let expected_len = (separator_count - 1) * N * N + separator_count;

        if value.len() != expected_len {
            return Err(Error::InvalidLength {
                len: value.len(),
                expected: expected_len,
            });
        }

        if separator_count >= N {
            return Err(Error::InvalidSeparators {
                count: separator_count,
                expected: N - 1,
            });
        }

        let sqs_with_error: Vec<Result<LatinSquare<N>, Error>> = value
            .split(SEPARATOR)
            .enumerate()
            .map(|(i, split)| {
                LatinSquare::try_from(split)
                    .map_err(|error| Error::InvalidLatinSquare { index: i, error })
            })
            .collect();

        let mut sqs = Vec::with_capacity(sqs_with_error.len());

        for sq in sqs_with_error {
            sqs.push(sq?);
        }

        let mols = MOLS::new(&sqs).map_err(|indices| Error::NotOrthogonal { indices })?;

        Ok(mols)
    }
}
