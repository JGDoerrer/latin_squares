use std::fmt::Debug;

use crate::{
    bitset::BitSet,
    constants::MAX_N,
    constraints::{Constraint, Constraints},
    types::Value,
};

#[derive(Clone)]
pub struct LatinSquare<const N: usize> {
    values: [[Value; N]; N],
}

impl<const N: usize> LatinSquare<N> {
    pub fn n(&self) -> usize {
        N
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

impl<const N: usize> From<Constraints<N>> for LatinSquare<N> {
    fn from(constraints: Constraints<N>) -> Self {
        debug_assert!(constraints.is_solved());
        let mut values = [[0; N]; N];

        for i in 0..N {
            for j in 0..N {
                match constraints.get(i, j) {
                    Constraint::Value(value) => values[i][j] = value,
                    Constraint::Impossible | Constraint::PossibleValues(_) => unreachable!(),
                }
            }
        }

        LatinSquare { values }
    }
}

impl<const N: usize> Debug for LatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n[")?;
        for i in 0..self.n() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "[")?;
            for j in 0..self.n() {
                write!(f, "{:2}, ", self.get(i, j))?;
            }
            write!(f, "]")?;
            if i != self.n() - 1 {
                writeln!(f, ",")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}
