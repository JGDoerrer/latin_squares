use std::fmt::{Debug, Display, Write};

use crate::{
    latin_square_dyn::{isqrt, LatinSquareDyn},
    latin_square_generator::LatinSquareGeneratorDyn,
    latin_square_trait::PartialLatinSquareTrait,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PartialLatinSquareDyn {
    n: usize,
    values: Box<[Option<u8>]>,
}

impl PartialLatinSquareTrait for PartialLatinSquareDyn {
    fn n(&self) -> usize {
        self.n
    }

    fn get_partial(&self, row: usize, col: usize) -> Option<usize> {
        self.values[row * self.n + col].map(|i| i.into())
    }
}

impl PartialLatinSquareDyn {
    pub fn empty(n: usize) -> Self {
        PartialLatinSquareDyn {
            n,
            values: vec![None; n * n].into_boxed_slice(),
        }
    }

    pub fn set(&mut self, row: usize, col: usize, val: Option<usize>) {
        self.values[row * self.n + col] = val.map(|i| i as u8);
    }

    pub fn num_entries(&self) -> usize {
        self.values.iter().filter(|v| v.is_some()).count()
    }

    pub fn first_empty_index(&self) -> Option<usize> {
        (0..self.n)
            .flat_map(|row| (0..self.n).map(move |col| self.get_partial(row, col)))
            .position(|entry| entry.is_none())
    }

    pub fn next_empty_index(&self, start: usize) -> Option<usize> {
        (0..self.n)
            .flat_map(|row| (0..self.n).map(move |col| self.get_partial(row, col)))
            .skip(start)
            .position(|entry| entry.is_none())
            .map(|index| index + start)
    }

    pub fn is_uniquely_completable(&self) -> bool {
        let mut generator = LatinSquareGeneratorDyn::from_partial_sq(self);
        let first_solution = generator.next();
        let second_solution = generator.next();

        first_solution.is_some() && second_solution.is_none()
    }

    pub fn is_uniquely_completable_to(&self, sq: &LatinSquareDyn) -> bool {
        debug_assert_eq!(self.n(), sq.n());

        let mut generator = LatinSquareGeneratorDyn::from_partial_sq(self);
        let first_solution = generator.next();
        let second_solution = generator.next();

        second_solution.is_none() && first_solution.is_some_and(|s| s == *sq)
    }

    pub fn is_critical_set_of(&self, sq: &LatinSquareDyn) -> bool {
        if !self.is_uniquely_completable_to(sq) {
            return false;
        }

        for i in 0..self.n() {
            for j in 0..self.n() {
                if self.get_partial(i, j).is_none() {
                    continue;
                }

                let mut copy = self.clone();
                copy.set(i, j, None);

                if copy.is_uniquely_completable() {
                    return false;
                }
            }
        }

        true
    }
}

impl Display for PartialLatinSquareDyn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.n {
            for j in 0..self.n {
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

impl Debug for PartialLatinSquareDyn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.n();

        writeln!(f)?;
        for i in 0..n {
            writeln!(f, "+{}", "---+".repeat(n))?;
            write!(f, "|")?;
            for j in 0..n {
                if let Some(value) = self.get_partial(i, j) {
                    write!(f, " {} |", value)?;
                } else {
                    write!(f, "   |")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "+{}", "---+".repeat(n))?;

        Ok(())
    }
}

impl From<&LatinSquareDyn> for PartialLatinSquareDyn {
    fn from(value: &LatinSquareDyn) -> Self {
        PartialLatinSquareDyn {
            n: value.n(),
            values: value.values().iter().map(|i| Some(*i)).collect(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidLength { len: usize },
    InvalidChar { index: usize, char: char },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength { len } => {
                write!(f, "Invalid len: {len}")
            }
            Error::InvalidChar { index, char } => {
                write!(f, "Invalid char at index {index}: {char}")
            }
        }
    }
}
impl TryFrom<&str> for PartialLatinSquareDyn {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Some(n) = isqrt(value.len()) else {
            return Err(Error::InvalidLength { len: value.len() });
        };

        let mut values = vec![None; n * n].into_boxed_slice();
        for (i, c) in value.chars().enumerate() {
            if c != '.' {
                let entry = c
                    .to_digit(10)
                    .ok_or(Error::InvalidChar { index: i, char: c })?;
                if entry >= n as u32 {
                    return Err(Error::InvalidChar { index: i, char: c });
                }
                values[i] = Some(entry as u8);
            }
        }

        Ok(PartialLatinSquareDyn { n, values })
    }
}
