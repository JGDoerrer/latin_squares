use std::fmt::Debug;

use crate::{bitset::BitSet, constants::MAX_N, constraints::Constraints, types::Value};

#[derive(Clone)]
pub struct LatinSquare {
    n: usize,
    values: [[Value; MAX_N]; MAX_N],
}

impl LatinSquare {
    pub fn n(&self) -> usize {
        self.n
    }

    pub fn get(&self, i: usize, j: usize) -> Value {
        self.values[i][j]
    }

    pub fn is_orthogonal_to(&self, other: &Self) -> bool {
        debug_assert!(self.n() == other.n());

        let n = self.n();

        for value in 0..n as Value {
            let mut other_values = BitSet::empty();

            for i in 0..n {
                for j in 0..n {
                    if self.get(i, j) == value {
                        other_values.insert(other.get(i, j).into());
                    }
                }
            }

            if other_values != BitSet::all_less_than(n) {
                return false;
            }
        }

        true
    }
}

impl From<Constraints> for LatinSquare {
    fn from(constraints: Constraints) -> Self {
        debug_assert!(constraints.is_solved());
        let n = constraints.n();

        let mut values = [[0; MAX_N]; MAX_N];

        for i in 0..n {
            for j in 0..n {
                values[i][j] = constraints.get(i, j).into_iter().next().unwrap() as Value;
            }
        }

        LatinSquare { n, values }
    }
}

impl Debug for LatinSquare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for i in 0..self.n() {
            for j in 0..self.n() {
                write!(f, "{:2} ", self.get(i, j))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
