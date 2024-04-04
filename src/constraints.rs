use std::fmt::Debug;

use crate::{bitset::BitSet, constants::MAX_N, latin_square::LatinSquare, types::Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    Impossible,
    Value(Value),
    PossibleValues(BitSet),
}

impl Constraint {
    pub fn bitset(&self) -> BitSet {
        match self {
            Constraint::Impossible => BitSet::empty(),
            Constraint::Value(v) => BitSet::single(*v as usize),
            Constraint::PossibleValues(bitset) => *bitset,
        }
    }
}

#[derive(Clone)]
pub struct Constraints<const N: usize> {
    matrix: [[Constraint; N]; N],
}

impl<const N: usize> Constraints<N> {
    pub const fn new() -> Self {
        Constraints {
            matrix: [[Constraint::PossibleValues(BitSet::all_less_than(N)); N]; N],
        }
    }

    pub fn new_first_row() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            constraints.set(0, i, i as Value);
        }

        constraints
    }

    pub fn new_reduced() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            constraints.set(0, i, i as Value);
            constraints.set(i, 0, i as Value);
        }

        constraints
    }

    pub const fn n(&self) -> usize {
        N
    }

    pub const fn get(&self, i: usize, j: usize) -> Constraint {
        self.matrix[i][j]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut Constraint {
        &mut self.matrix[i][j]
    }

    pub fn set(&mut self, i: usize, j: usize, value: Value) {
        debug_assert!(
            matches!(self.get(i, j), Constraint::PossibleValues(b) if b.contains(value as usize))
        );

        self.propagate_value(i, j, value);
    }

    pub fn propagate_value(&mut self, i: usize, j: usize, value: Value) {
        let mut stack = Vec::with_capacity(N);
        stack.push((i, j, value));

        while let Some((i, j, value)) = stack.pop() {
            if Constraint::Value(value) == self.get(i, j) {
                continue;
            }

            *self.get_mut(i, j) = Constraint::Value(value);

            let mask = BitSet::single(value.into())
                .complement()
                .intersect(BitSet::all_less_than(N));

            for k in 0..N {
                if k == j {
                    continue;
                }
                let a = self.get_mut(i, k);

                match a {
                    Constraint::Impossible => {
                        return;
                    }
                    Constraint::Value(v) => {
                        if !mask.contains(*v as usize) {
                            *a = Constraint::Impossible;
                            return;
                        }
                    }
                    Constraint::PossibleValues(values) => {
                        *values = values.intersect(mask);

                        if values.is_single() {
                            let value = values.into_iter().next().unwrap() as Value;

                            stack.push((i, k, value));
                        }
                    }
                }
            }

            for k in 0..N {
                if k == i {
                    continue;
                }

                let a = self.get_mut(k, j);

                match a {
                    Constraint::Impossible => {
                        return;
                    }
                    Constraint::Value(v) => {
                        if !mask.contains(*v as usize) {
                            *a = Constraint::Impossible;
                            return;
                        }
                    }
                    Constraint::PossibleValues(values) => {
                        *values = values.intersect(mask);

                        if values.is_single() {
                            let value = values.into_iter().next().unwrap() as Value;

                            stack.push((k, j, value));
                        }
                    }
                }
            }
        }
    }

    pub fn find_singles(&mut self) {
        for i in 0..N {
            let mut counts = [0; N];
            for j in 0..N {
                if let Constraint::PossibleValues(v) = self.get(i, j) {
                    for value in v {
                        counts[value] += 1;
                    }
                }
            }

            for value in counts
                .into_iter()
                .enumerate()
                .filter(|(_, c)| *c == 1)
                .map(|(i, _)| i)
            {
                for j in 0..N {
                    if let Constraint::PossibleValues(v) = self.get(i, j) {
                        if v.contains(value) {
                            self.propagate_value(i, j, value as Value);
                        }
                    }
                }
            }

            let mut counts = [0; N];
            for j in 0..N {
                if let Constraint::PossibleValues(v) = self.get(j, i) {
                    for value in v {
                        counts[value] += 1;
                    }
                }
            }

            for value in counts
                .into_iter()
                .enumerate()
                .filter(|(_, c)| *c == 1)
                .map(|(i, _)| i)
            {
                for j in 0..N {
                    if let Constraint::PossibleValues(v) = self.get(j, i) {
                        if v.contains(value) {
                            self.propagate_value(j, i, value as Value);
                        }
                    }
                }
            }
        }
    }

    pub fn first_unsolved(&self) -> Option<(usize, usize)> {
        for i in 0..self.n() {
            for j in 0..self.n() {
                if matches!(self.get(i, j), Constraint::PossibleValues(_)) {
                    return Some((i, j));
                }
            }
        }
        None
    }

    pub fn is_solvable(&self) -> bool {
        for i in 0..N {
            for j in 0..N {
                if matches!(self.get(i, j), Constraint::Impossible) {
                    return false;
                }
            }
        }

        for i in 0..N {
            let mut values = BitSet::empty();

            for j in 0..N {
                values = values.union(self.get(i, j).bitset());
            }

            if values != BitSet::all_less_than(N) {
                return false;
            }
        }

        for i in 0..N {
            let mut values = BitSet::empty();

            for j in 0..N {
                values = values.union(self.get(j, i).bitset());
            }

            if values != BitSet::all_less_than(N) {
                return false;
            }
        }

        true
    }

    pub fn is_solved(&self) -> bool {
        for i in 0..self.n() {
            let mut values = BitSet::empty();

            for j in 0..self.n() {
                match self.get(i, j) {
                    Constraint::Impossible => return false,
                    Constraint::PossibleValues(_) => return false,
                    Constraint::Value(value) => {
                        values.insert(value as usize);
                    }
                }
            }

            if values != BitSet::all_less_than(self.n()) {
                return false;
            }
        }

        for i in 0..self.n() {
            let mut values = BitSet::empty();

            for j in 0..self.n() {
                match self.get(j, i) {
                    Constraint::Impossible => return false,
                    Constraint::PossibleValues(_) => return false,
                    Constraint::Value(value) => {
                        values.insert(value as usize);
                    }
                }
            }

            if values != BitSet::all_less_than(self.n()) {
                return false;
            }
        }

        true
    }

    pub fn is_orthogonal_to(&self, sq: &LatinSquare<N>) -> bool {
        debug_assert!(sq.n() == self.n());
        let n = self.n();

        for value in 0..n as Value {
            let mut other_values = BitSet::empty();

            for i in 0..n {
                for j in 0..n {
                    if sq.get(i, j) == value {
                        other_values = other_values.union(self.get(i, j).bitset());
                    }
                }
            }

            if other_values != BitSet::all_less_than(n) {
                return false;
            }
        }

        true
    }

    // pub fn make_orthogonal_to(&mut self, other: &Constraints<N>) {
    //     let mut known_values = [BitSet::empty(); MAX_N];
    //     for i in 0..N {
    //         for j in 0..N {
    //             let value = other.get(i, j);

    //             if value.is_single() && self.get(i, j).is_single() {
    //                 let value = value.into_iter().next().unwrap();
    //                 known_values[value] = known_values[value].union(self.get(i, j));
    //             }
    //         }
    //     }

    //     for i in 0..N {
    //         for j in 0..N {
    //             let value = other.get(i, j);
    //             if value.is_single() && !self.get(i, j).is_single() {
    //                 let value = value.into_iter().next().unwrap();

    //                 let new = self.get(i, j).intersect(known_values[value].complement());
    //                 *self.get_mut(i, j) = new;
    //                 if new.is_single() {
    //                     self.propagate_value(i, j, new.into_iter().next().unwrap() as Value);
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn make_orthogonal_to_sq(&mut self, sq: &LatinSquare<N>) {
        let mut known_values = [BitSet::empty(); MAX_N];
        for i in 0..N {
            for j in 0..N {
                if let Constraint::Value(v) = self.get(i, j) {
                    let value = sq.get(i, j) as usize;
                    known_values[value].insert(v as usize);
                }
            }
        }

        for i in 0..N {
            for j in 0..N {
                let value = sq.get(i, j) as usize;
                match self.get_mut(i, j) {
                    Constraint::Impossible | Constraint::Value(_) => {}
                    Constraint::PossibleValues(bitset) => {
                        let new = bitset.intersect(known_values[value].complement());
                        *bitset = new;
                        if new.is_empty() {
                            *self.get_mut(i, j) = Constraint::Impossible;
                        } else if new.is_single() {
                            let value = new.into_iter().next().unwrap() as Value;
                            self.propagate_value(i, j, value);
                        }
                    }
                }
            }
        }
    }

    // pub fn try_solve(&mut self) {
    //     let n = self.n();

    //     for value in 0..n {
    //         for i in 0..n {
    //             let mut index = None;
    //             let mut single = true;
    //             for j in 0..n {
    //                 if self.get(i, j).contains(value) {
    //                     if index.is_none() {
    //                         index = Some(j);
    //                         single = true;
    //                     } else {
    //                         single = false;
    //                     }
    //                 }
    //             }

    //             if let Some(j) = index {
    //                 if single && !self.get(i, j).is_single() {
    //                     *self.get_mut(i, j) = BitSet::single(value);
    //                     self.propagate_value(i, j, value as Value);
    //                 }
    //             }
    //         }

    //         for j in 0..n {
    //             let mut index = None;
    //             let mut single = true;
    //             for i in 0..n {
    //                 if self.get(i, j).contains(value) {
    //                     if index.is_none() {
    //                         index = Some(i);
    //                         single = true;
    //                     } else {
    //                         single = false;
    //                     }
    //                 }
    //             }

    //             if let Some(i) = index {
    //                 if single && !self.get(i, j).is_single() {
    //                     *self.get_mut(i, j) = BitSet::single(value);
    //                     self.propagate_value(i, j, value as Value);
    //                 }
    //             }
    //         }
    //     }
    // }
}

impl<const N: usize> Debug for Constraints<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n[")?;
        for i in 0..self.n() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "[")?;
            for j in 0..self.n() {
                match self.get(i, j) {
                    Constraint::Impossible => todo!(),
                    Constraint::Value(v) => write!(f, "{:3?}, ", v)?,
                    Constraint::PossibleValues(v) => write!(f, "{:03X}, ", v.bits())?,
                }
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
