use std::{cmp::Ordering, fmt::Display};

use crate::{
    latin_square::{self, LatinSquare},
    permutation::{Permutation, PermutationIter},
    tuple_iterator::TupleIterator,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mols<const N: usize> {
    sqs: Vec<LatinSquare<N>>,
}

impl<const N: usize> Mols<N> {
    pub fn new(sqs: Vec<LatinSquare<N>>) -> Result<Self, (usize, usize)> {
        for i in 0..sqs.len() {
            for j in (i + 1)..sqs.len() {
                if !sqs[i].is_orthogonal_to(&sqs[j]) {
                    return Err((i, j));
                }
            }
        }

        Ok(Mols { sqs: sqs.to_vec() })
    }

    pub fn new_unchecked(sqs: Vec<LatinSquare<N>>) -> Self {
        Mols { sqs }
    }

    const ROWS: [[u8; N]; N] = {
        let mut rows = [[0; N]; N];
        let mut i = 0;
        while i < N {
            rows[i] = [i as u8; N];
            i += 1;
        }
        rows
    };

    const COLS: [[u8; N]; N] = {
        let mut col = [0; N];

        let mut i = 0;
        while i < N {
            col[i] = i as u8;
            i += 1;
        }

        [col; N]
    };

    pub fn normalize_main_class_set_sq(
        &self,
        lookup: &[Vec<(Permutation<N>, Permutation<N>)>],
        in_sq: &LatinSquare<N>,
    ) -> Option<Self> {
        let values: Vec<_> = [Self::ROWS, Self::COLS]
            .into_iter()
            .chain(self.sqs.iter().map(|sq| (*sq).to_values()))
            .map(|v| v.map(|v| v.map(|v| v as usize)))
            .collect();

        let mut min_sq = self.sqs[0];

        let mut min_rcs = vec![[0, 1, 2]];

        for [r, c, s] in TupleIterator::<3>::new(values.len())
            .flat_map(|rcs| PermutationIter::new().map(move |p| p.apply_array(rcs)))
        {
            let sq = LatinSquare::from_rcs(values[r], values[c], values[s]);
            let isotopy_class = sq.isotopy_class_lookup(lookup);

            // let (isotopy_class, permutations) = sq.isotopy_class_permutations(lookup);

            match isotopy_class.cmp(&min_sq) {
                Ordering::Less => {
                    min_sq = sq;
                    if min_sq.cmp(in_sq).is_lt() {
                        return None;
                    }
                    min_rcs.push([r, c, s]);
                }
                Ordering::Equal => {
                    min_rcs.push([r, c, s]);
                }
                Ordering::Greater => {}
            }
        }

        if min_sq != *in_sq {
            return None;
        }
        debug_assert!(min_sq == min_sq.main_class_lookup(lookup));

        let mut min_perms = vec![];

        for [r, c, s] in min_rcs {
            let sq = LatinSquare::from_rcs(values[r], values[c], values[s]);
            let (_, permutations) = sq.isotopy_class_permutations(lookup);
            min_perms.push(([r, c, s], permutations));
        }

        let mut min_mols = self.clone();
        for (rcs, perms) in min_perms {
            let rows = values[rcs[0]];
            let cols = values[rcs[1]];

            let sqs: Vec<_> = values
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != rcs[0] && *i != rcs[1])
                .map(|(_, vals)| LatinSquare::from_rcs(rows, cols, *vals))
                .collect();

            for perm in perms {
                let mut mols = Mols { sqs: sqs.clone() };
                mols.permute_rows(&perm[0]);
                mols.permute_cols_and_reduce(&perm[1]);
                // mols.permute_cols(&perm[1]);
                // mols.reduce_all_sqs();
                mols.sqs.sort();

                if mols < min_mols {
                    min_mols = mols;
                }
            }
        }

        if min_mols.sqs[0] == *in_sq {
            Some(min_mols)
        } else {
            None
        }
    }

    pub fn normalize_main_class_set(
        &self,
        lookup: &[Vec<(Permutation<N>, Permutation<N>)>],
    ) -> Self {
        let values: Vec<_> = [Self::ROWS, Self::COLS]
            .into_iter()
            .chain(self.sqs.iter().map(|sq| (*sq).to_values()))
            .map(|v| v.map(|v| v.map(|v| v as usize)))
            .collect();

        let mut min_sq = self.sqs[0];
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
            let sq = LatinSquare::from_rcs(values[r], values[c], values[s]);
            let isotopy_class = sq.isotopy_class_lookup(lookup);

            // let (isotopy_class, permutations) = sq.isotopy_class_permutations(lookup);

            match isotopy_class.cmp(&min_sq) {
                Ordering::Less => {
                    min_sq = sq;
                    let (_, permutations) = sq.isotopy_class_permutations(lookup);
                    min_perms = vec![([r, c, s], permutations)];
                }
                Ordering::Equal => {
                    let (_, permutations) = sq.isotopy_class_permutations(lookup);
                    min_perms.push(([r, c, s], permutations))
                }
                Ordering::Greater => {}
            }
        }

        debug_assert!(min_sq == min_sq.main_class_lookup(lookup));

        let mut min_mols = self.clone();
        for (rcs, perms) in min_perms {
            let rows = values[rcs[0]];
            let cols = values[rcs[1]];

            let sqs: Vec<_> = values
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != rcs[0] && *i != rcs[1])
                .map(|(_, vals)| LatinSquare::from_rcs(rows, cols, *vals))
                .collect();

            for perm in perms {
                let mut mols = Mols { sqs: sqs.clone() };
                mols.permute_rows(&perm[0]);
                mols.permute_cols_and_reduce(&perm[1]);
                // mols.permute_cols(&perm[1]);
                // mols.reduce_all_sqs();
                mols.sqs.sort_by_key(|sq| sq.get(1, 0));

                if mols < min_mols {
                    min_mols = mols;
                }
            }
        }
        min_mols
    }

    pub fn permute_rows(&mut self, permutation: &Permutation<N>) {
        for sq in self.sqs.iter_mut() {
            sq.permute_rows(permutation);
        }
    }

    fn permute_cols(&mut self, permutation: &Permutation<N>) {
        let inverse = permutation.inverse();
        for sq in self.sqs.iter_mut() {
            sq.permute_cols_simd(&inverse);
        }
    }

    fn reduce_all_sqs(&mut self) {
        for sq in self.sqs.iter_mut() {
            let first_row = sq.get_row(0);

            let mut permutation = [0; N];
            for i in 0..N {
                permutation[first_row[i] as usize] = i;
            }

            let symbol_permutation = Permutation::from_array(permutation);
            sq.permute_vals_simd(&symbol_permutation);
        }
    }

    pub fn permute_cols_and_reduce(&mut self, col_permutation: &Permutation<N>) {
        let inverse = col_permutation.inverse();
        for sq in self.sqs.iter_mut() {
            let first_row = col_permutation.apply_array(*sq.get_row(0));

            let mut permutation = [0; N];
            for i in 0..N {
                permutation[first_row[i] as usize] = i;
            }

            sq.permute_cols_vals_simd(&inverse, &permutation.into());
        }
    }
}

impl<const N: usize> Display for Mols<N> {
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

impl<const N: usize> Ord for Mols<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.sqs.len(), other.sqs.len());

        for (a, b) in self.sqs.iter().zip(&other.sqs) {
            match a.cmp_rows(b) {
                Ordering::Equal => {}
                o => return o,
            }
        }

        Ordering::Equal
    }
}

impl<const N: usize> PartialOrd for Mols<N> {
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

impl<const N: usize> TryFrom<&str> for Mols<N> {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let separator_count = value.chars().filter(|c| *c == SEPARATOR).count();
        let expected_len = (separator_count + 1) * N * N + separator_count;

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

        let mols = Mols::new(sqs).map_err(|indices| Error::NotOrthogonal { indices })?;

        Ok(mols)
    }
}
