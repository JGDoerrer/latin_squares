use crate::{
    bitset::BitSet16,
    latin_square::{Cell, LatinSquare},
    latin_square_trait::PartialLatinSquareTrait,
    partial_latin_square::PartialLatinSquare,
    partial_latin_square_dyn::PartialLatinSquareDyn,
};

#[derive(Debug, Clone)]
pub struct Constraints<const N: usize> {
    sq: PartialLatinSquare<N>,
    rows: [BitSet16; N],
    cols: [BitSet16; N],
}

impl<const N: usize> Default for Constraints<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Constraints<N> {
    pub fn new() -> Self {
        Constraints {
            sq: PartialLatinSquare::empty(),
            rows: [BitSet16::all_less_than(N); N],
            cols: [BitSet16::all_less_than(N); N],
        }
    }

    pub fn new_first_row() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            let value = constraints
                .get_possibilities(0, i)
                .into_iter()
                .next()
                .unwrap();
            constraints.set(0, i, value);
        }

        constraints
    }

    pub fn new_reduced() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            constraints.set(0, i, i);
            if i != 0 {
                constraints.set(i, 0, i);
            }
        }

        constraints
    }

    pub fn new_partial(sq: &PartialLatinSquare<N>) -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            for j in 0..N {
                if let Some(value) = sq.get_partial(i, j) {
                    constraints.set(i, j, value);
                }
            }
        }

        constraints
    }

    pub fn partial_sq(&self) -> &PartialLatinSquare<N> {
        &self.sq
    }

    pub fn set(&mut self, i: usize, j: usize, value: usize) {
        debug_assert!(self.sq.get_partial(i, j).is_none());
        debug_assert!(self.rows[i].contains(value));
        debug_assert!(self.cols[j].contains(value));

        self.sq.set(i, j, Some(value));
        self.rows[i].remove(value);
        self.cols[j].remove(value);
    }

    pub fn get_possibilities(&self, i: usize, j: usize) -> BitSet16 {
        self.rows[i].intersect(self.cols[j])
    }

    pub fn is_set(&self, i: usize, j: usize) -> bool {
        self.sq.get_partial(i, j).is_some()
    }

    pub fn first_empty(&self) -> Option<(usize, usize)> {
        for i in 0..N {
            if !self.is_set(0, i) {
                return Some((0, i));
            }
        }
        for i in 0..N {
            if !self.is_set(i, 0) {
                return Some((i, 0));
            }
        }

        let mut min_values = N * N + 1;
        let mut index = (0, 0);

        for i in 0..N {
            for j in 0..N {
                if !self.is_set(i, j) {
                    let len = self.get_possibilities(i, j).len();

                    if len < min_values {
                        min_values = len;
                        index = (i, j);
                    }
                }
            }
        }

        (min_values < N * N + 1).then_some(index)
    }

    pub fn most_constrained_cell(&self) -> Option<Cell> {
        (0..N * N)
            .map(|index| Cell(index / N, index % N))
            .filter(|cell| self.get_possibilities(cell.0, cell.1).len() >= 2)
            .min_by_key(|cell| self.get_possibilities(cell.0, cell.1).len() >= 2)
    }

    pub fn into_latin_square(self) -> LatinSquare<N> {
        self.sq.try_into().unwrap()
    }

    pub fn is_solved(&self) -> bool {
        self.sq.num_entries() == N * N
    }

    pub fn is_solvable(&self) -> bool {
        for i in 0..N {
            for j in 0..N {
                if self.sq.get_partial(i, j).is_none() && self.get_possibilities(i, j).is_empty() {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_singles(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            for i in 0..N {
                for j in 0..N {
                    if self.sq.get_partial(i, j).is_none()
                        && self.get_possibilities(i, j).is_single()
                    {
                        self.set(
                            i,
                            j,
                            self.get_possibilities(i, j).into_iter().next().unwrap(),
                        );
                        changed = true;
                    }
                }
            }
        }
    }
}

impl<const N: usize> From<Constraints<N>> for LatinSquare<N> {
    fn from(constraints: Constraints<N>) -> Self {
        assert!(constraints.is_solved());

        constraints.sq.try_into().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ConstraintsDyn {
    sq: PartialLatinSquareDyn,
    rows: Box<[BitSet16]>,
    cols: Box<[BitSet16]>,
}

impl ConstraintsDyn {
    pub fn new(n: usize) -> Self {
        ConstraintsDyn {
            sq: PartialLatinSquareDyn::empty(n),
            rows: vec![BitSet16::all_less_than(n); n].into_boxed_slice(),
            cols: vec![BitSet16::all_less_than(n); n].into_boxed_slice(),
        }
    }

    pub fn new_partial(sq: &PartialLatinSquareDyn) -> Self {
        let n = sq.n();
        let mut constraints = Self::new(n);

        for i in 0..n {
            for j in 0..n {
                if let Some(value) = sq.get_partial(i, j) {
                    constraints.set(i, j, value);
                }
            }
        }

        constraints
    }

    pub fn partial_sq(&self) -> &PartialLatinSquareDyn {
        &self.sq
    }

    pub fn set(&mut self, i: usize, j: usize, value: usize) {
        debug_assert!(self.sq.get_partial(i, j).is_none());
        debug_assert!(self.rows[i].contains(value));
        debug_assert!(self.cols[j].contains(value));

        self.sq.set(i, j, Some(value));
        self.rows[i].remove(value);
        self.cols[j].remove(value);
        // self.propagate_value(i, j, value);
    }

    pub fn get_possibilities(&self, i: usize, j: usize) -> BitSet16 {
        self.rows[i].intersect(self.cols[j])
    }

    pub fn is_set(&self, i: usize, j: usize) -> bool {
        self.sq.get_partial(i, j).is_some()
    }

    pub fn is_solved(&self) -> bool {
        self.sq.num_entries() == self.sq.n() * self.sq.n()
    }

    pub fn is_solvable(&self) -> bool {
        let n = self.sq.n();
        for i in 0..n {
            for j in 0..n {
                if self.sq.get_partial(i, j).is_none() && self.get_possibilities(i, j).is_empty() {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_singles(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            let n = self.sq.n();
            for i in 0..n {
                for j in 0..n {
                    if !self.is_set(i, j) && self.get_possibilities(i, j).is_single() {
                        self.set(
                            i,
                            j,
                            self.get_possibilities(i, j).into_iter().next().unwrap(),
                        );
                        changed = true;
                    }
                }
            }

            // find values with ouly one valid place
            for i in 0..n {
                let mut candidates = self.rows[i];
                let mut found = BitSet16::empty();

                if candidates.is_empty() {
                    continue;
                }

                for j in 0..n {
                    let col = self.cols[j];

                    candidates = candidates.intersect(found.intersect(col).complement());
                    found = found.union(col);
                }

                for value in candidates {
                    for j in 0..n {
                        if !self.is_set(i, j) && self.cols[j].intersect(found).contains(value) {
                            self.set(i, j, value);
                            changed = true;
                            break;
                        }
                    }
                }

                let mut candidates = self.cols[i];
                let mut found = BitSet16::empty();

                if candidates.is_empty() {
                    continue;
                }

                for j in 0..n {
                    let row = self.rows[j];

                    candidates = candidates.intersect(found.intersect(row).complement());
                    found = found.union(row);
                }

                for value in candidates {
                    for j in 0..n {
                        if !self.is_set(j, i) && self.rows[j].intersect(found).contains(value) {
                            self.set(j, i, value);
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn first_empty(&self) -> Option<(usize, usize)> {
        let n = self.sq.n();
        for i in 0..n {
            if !self.is_set(0, i) {
                return Some((0, i));
            }
        }
        for i in 0..n {
            if !self.is_set(i, 0) {
                return Some((i, 0));
            }
        }

        let mut min_values = n * n + 1;
        let mut index = (0, 0);

        for i in 0..n {
            for j in 0..n {
                if !self.is_set(i, j) {
                    let len = self.get_possibilities(i, j).len();

                    if len < min_values {
                        min_values = len;
                        index = (i, j);
                    }
                }
            }
        }

        (min_values < n * n + 1).then_some(index)
    }
}
