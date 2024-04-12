use crate::{
    bitset::BitSet,
    latin_square::{Cell, LatinSquare},
};

#[derive(Debug, Clone)]
pub struct Constraints<const N: usize> {
    constraints: [[BitSet; N]; N],
}

impl<const N: usize> Constraints<N> {
    pub fn new() -> Self {
        Constraints {
            constraints: [[BitSet::all_less_than(N); N]; N],
        }
    }

    pub fn new_first_row() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            let value = constraints.get(0, i).into_iter().next().unwrap();
            constraints.set(0, i, value);
        }

        constraints
    }

    pub fn new_reduced() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            constraints.set(0, i, i);
            constraints.set(i, 0, i);
        }

        constraints
    }

    pub fn set(&mut self, i: usize, j: usize, value: usize) {
        self.propagate_value(i, j, value);
    }

    pub fn get(&self, i: usize, j: usize) -> BitSet {
        self.constraints[i][j]
    }

    fn propagate_value(&mut self, i: usize, j: usize, value: usize) {
        let value_index = value;
        assert!(self.constraints[i][j].contains(value_index));
        self.constraints[i][j] = BitSet::single(value_index);

        let mask = BitSet::single(value)
            .complement()
            .intersect(BitSet::all_less_than(N * N));

        for k in 0..N {
            if k != j {
                if self.constraints[i][k].intersect(mask) != self.constraints[i][k] {
                    self.constraints[i][k] = self.constraints[i][k].intersect(mask);
                    if self.constraints[i][k].is_single() {
                        let value = self.constraints[i][k].into_iter().next().unwrap();
                        self.propagate_value(i, k, value);
                    }
                }
            }
            if k != i {
                if self.constraints[k][j].intersect(mask) != self.constraints[k][j] {
                    self.constraints[k][j] = self.constraints[k][j].intersect(mask);
                    if self.constraints[k][j].is_single() {
                        let value = self.constraints[k][j].into_iter().next().unwrap();
                        self.propagate_value(k, j, value);
                    }
                }
            }
        }
    }

    pub fn get_next(&self) -> Option<(usize, usize)> {
        for i in 0..N {
            if !self.get(0, i).is_single() {
                return Some((0, i));
            }
        }
        for i in 0..N {
            if !self.get(i, 0).is_single() {
                return Some((i, 0));
            }
        }

        let mut min_values = N * N + 1;
        let mut index = (0, 0);

        for i in 0..N {
            for j in 0..N {
                if !self.get(i, j).is_single() {
                    let len = self.get(i, j).len();

                    if len < min_values {
                        min_values = len;
                        index = (i, j);
                    }
                }
            }
        }

        (min_values < N * N + 1).then(|| index)
    }

    pub fn most_constrained_cell(&self) -> Option<Cell> {
        (0..N * N)
            .map(|index| Cell(index / N, index % N))
            .filter(|cell| self.get(cell.0, cell.1).len() >= 2)
            .min_by_key(|cell| self.get(cell.0, cell.1).len() >= 2)
    }

    pub fn to_latin_square(self) -> LatinSquare<N> {
        self.into()
    }

    pub fn is_solved(&self) -> bool {
        for i in 0..N {
            for j in 0..N {
                if !self.get(i, j).is_single() {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_solvable(&self) -> bool {
        for i in 0..N {
            for j in 0..N {
                if self.get(i, j).is_empty() {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_singles(&mut self) {
        for i in 0..N {
            let mut counts = [0; N];
            for j in 0..N {
                if !self.get(i, j).is_single() {
                    for value in self.get(i, j) {
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
                    if !self.get(i, j).is_single() {
                        if self.get(i, j).contains(value) {
                            self.propagate_value(i, j, value);
                        }
                    }
                }
            }

            let mut counts = [0; N];
            for j in 0..N {
                if !self.get(j, i).is_single() {
                    for value in self.get(j, i) {
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
                    if !self.get(j, i).is_single() {
                        if self.get(j, i).contains(value) {
                            self.propagate_value(j, i, value);
                        }
                    }
                }
            }
        }
    }

    pub fn make_orthogonal_to_sq(&mut self, sq: &LatinSquare<N>) {
        let mut known_values = [BitSet::empty(); N];
        for i in 0..N {
            for j in 0..N {
                if self.get(i, j).is_single() {
                    let value = sq.get(i, j) as usize;
                    known_values[value].insert(self.get(i, j).into_iter().next().unwrap());
                }
            }
        }

        for i in 0..N {
            for j in 0..N {
                let value = sq.get(i, j) as usize;
                if !self.get(i, j).is_single() {
                    let new = self.get(i, j).intersect(known_values[value].complement());
                    self.constraints[i][j] = new;

                    if new.is_single() {
                        let value = new.into_iter().next().unwrap();
                        self.propagate_value(i, j, value);
                    }
                }
            }
        }
    }
}
