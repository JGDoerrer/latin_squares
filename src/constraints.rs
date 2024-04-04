use std::fmt::Debug;

use crate::{bitset::BitSet, constants::MAX_N, latin_square::LatinSquare, types::Value};

#[derive(Clone)]
pub struct Constraints {
    n: usize,
    matrix: [[BitSet; MAX_N]; MAX_N],
}

impl Constraints {
    pub const fn new(n: usize) -> Self {
        Constraints {
            n,
            matrix: [[BitSet::all_less_than(n); MAX_N]; MAX_N],
        }
    }

    pub fn new_first_row(n: usize) -> Self {
        let mut constraints = Self::new(n);

        for i in 0..n {
            constraints.set(0, i, i as Value);
        }

        constraints
    }

    pub fn new_reduced(n: usize) -> Self {
        let mut constraints = Self::new(n);

        for i in 0..n {
            constraints.set(0, i, i as Value);
            constraints.set(i, 0, i as Value);
        }

        constraints
    }

    pub const fn n(&self) -> usize {
        self.n
    }

    pub const fn get(&self, i: usize, j: usize) -> BitSet {
        self.matrix[i][j]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut BitSet {
        &mut self.matrix[i][j]
    }

    pub fn set(&mut self, i: usize, j: usize, value: Value) {
        debug_assert!(self.get(i, j).contains(value as usize));

        *self.get_mut(i, j) = BitSet::single(value.into());
        self.propagate_value(i, j, value);
    }

    pub fn propagate_value(&mut self, i: usize, j: usize, value: Value) {
        let n = self.n();

        let mask = BitSet::single(value.into())
            .complement()
            .intersect(BitSet::all_less_than(n));

        for k in 0..n {
            if k == j {
                continue;
            }
            let a = self.get_mut(i, k);

            if a.is_single() {
                *a = a.intersect(mask);
                continue;
            }

            *a = a.intersect(mask);

            if a.is_single() {
                let value = a.into_iter().next().unwrap() as Value;
                self.propagate_value(i, k, value);
            }
        }

        for k in 0..n {
            if k == i {
                continue;
            }

            let a = self.get_mut(k, j);

            if a.is_single() {
                *a = a.intersect(mask);
                continue;
            }

            *a = a.intersect(mask);

            if a.is_single() {
                let value = a.into_iter().next().unwrap() as Value;
                self.propagate_value(k, j, value);
            }
        }
    }

    pub fn first_unsolved(&self) -> Option<(usize, usize)> {
        for i in 0..self.n() {
            for j in 0..self.n() {
                if !self.get(i, j).is_single() {
                    return Some((i, j));
                }
            }
        }
        None
    }

    pub fn is_solvable(&self) -> bool {
        for i in 0..self.n() {
            for j in 0..self.n() {
                if self.get(i, j).is_empty() {
                    return false;
                }
            }
        }

        for i in 0..self.n() {
            let mut values = BitSet::empty();

            for j in 0..self.n() {
                values = values.union(self.get(i, j));
            }

            if values != BitSet::all_less_than(self.n()) {
                return false;
            }
        }

        for i in 0..self.n() {
            let mut values = BitSet::empty();

            for j in 0..self.n() {
                values = values.union(self.get(j, i));
            }

            if values != BitSet::all_less_than(self.n()) {
                return false;
            }
        }

        true
    }

    pub fn is_solved(&self) -> bool {
        for i in 0..self.n() {
            let mut values = BitSet::empty();

            for j in 0..self.n() {
                if !self.get(i, j).is_single() {
                    return false;
                }

                values = values.union(self.get(i, j));
            }

            if values != BitSet::all_less_than(self.n()) {
                return false;
            }
        }

        for i in 0..self.n() {
            let mut values = BitSet::empty();

            for j in 0..self.n() {
                if !self.get(j, i).is_single() {
                    return false;
                }

                values = values.union(self.get(j, i));
            }

            if values != BitSet::all_less_than(self.n()) {
                return false;
            }
        }

        true
    }

    pub fn is_orthogonal_to(&self, sq: &LatinSquare) -> bool {
        debug_assert!(sq.n() == self.n());
        let n = self.n();

        for value in 0..n as Value {
            let mut other_values = BitSet::empty();

            for i in 0..n {
                for j in 0..n {
                    if sq.get(i, j) == value {
                        other_values = other_values.union(self.get(i, j));
                    }
                }
            }

            if other_values != BitSet::all_less_than(n) {
                return false;
            }
        }

        true
    }

    pub fn make_orthogonal_to(&mut self, sq: &LatinSquare) {
        debug_assert!(sq.n() == self.n());
        let n = self.n();

        let mut known_values = [BitSet::empty(); MAX_N];
        for i in 0..n {
            for j in 0..n {
                let value = sq.get(i, j) as usize;
                if self.get(i, j).is_single() {
                    known_values[value] = known_values[value].union(self.get(i, j));
                }
            }
        }

        for i in 0..n {
            for j in 0..n {
                let value = sq.get(i, j) as usize;
                if !self.get(i, j).is_single() {
                    let new = self.get(i, j).intersect(known_values[value].complement());
                    *self.get_mut(i, j) = new;
                    if new.is_single() {
                        self.propagate_value(i, j, new.into_iter().next().unwrap() as Value);
                    }
                }
            }
        }
    }

    pub fn try_solve(&mut self) {
        let n = self.n();

        for value in 0..n {
            for i in 0..n {
                let mut index = None;
                let mut single = true;
                for j in 0..n {
                    if self.get(i, j).contains(value) {
                        if index.is_none() {
                            index = Some(j);
                            single = true;
                        } else {
                            single = false;
                        }
                    }
                }

                if let Some(j) = index {
                    if single && !self.get(i, j).is_single() {
                        *self.get_mut(i, j) = BitSet::single(value);
                        self.propagate_value(i, j, value as Value);
                    }
                }
            }

            for j in 0..n {
                let mut index = None;
                let mut single = true;
                for i in 0..n {
                    if self.get(i, j).contains(value) {
                        if index.is_none() {
                            index = Some(i);
                            single = true;
                        } else {
                            single = false;
                        }
                    }
                }

                if let Some(i) = index {
                    if single && !self.get(i, j).is_single() {
                        *self.get_mut(i, j) = BitSet::single(value);
                        self.propagate_value(i, j, value as Value);
                    }
                }
            }
        }
    }
}

impl Debug for Constraints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.matrix[0..self.n()].iter().map(|row| &row[0..self.n()]))
            .finish()
    }
}
