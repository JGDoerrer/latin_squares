use crate::{bitset::BitSet, latin_square::LatinSquarePair, types::Value};

#[derive(Debug, Clone)]
pub struct PairConstraints<const N: usize> {
    constraints: [[BitSet; N]; N],
}

impl<const N: usize> PairConstraints<N> {
    pub fn new() -> Self {
        PairConstraints {
            constraints: [[BitSet::all_less_than(N * N); N]; N],
        }
    }

    pub fn new_first_row() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            let value = constraints.get(0, i).into_iter().next().unwrap();
            let value_pair = ((value % N) as Value, (value / N) as Value);
            constraints.set(0, i, value_pair);
        }

        constraints
    }

    pub fn set(&mut self, i: usize, j: usize, value: (Value, Value)) {
        self.propagate_value(i, j, value);
    }

    pub fn get(&self, i: usize, j: usize) -> BitSet {
        self.constraints[i][j]
    }

    fn propagate_value(&mut self, i: usize, j: usize, value: (Value, Value)) {
        let value_index = value.0 as usize + value.1 as usize * N;
        assert!(self.constraints[i][j].contains(value_index));
        self.constraints[i][j] = BitSet::single(value_index);

        let every_nth = {
            let mut num = 1u128;
            for _ in 0..N {
                num |= num << N;
            }
            num
        };

        let mask1 = BitSet::from_bits(BitSet::all_less_than(N).bits() << (value.1 as usize * N))
            .complement()
            .intersect(BitSet::all_less_than(N * N));

        let mask2 = BitSet::from_bits(BitSet::single(value.0 as usize).bits() * every_nth)
            .complement()
            .intersect(BitSet::all_less_than(N * N));

        let mask = mask1.intersect(mask2);

        for k in 0..N {
            if k != j {
                if self.constraints[i][k].intersect(mask) != self.constraints[i][k] {
                    self.constraints[i][k] = self.constraints[i][k].intersect(mask);
                    if self.constraints[i][k].is_single() {
                        let value = self.constraints[i][k].into_iter().next().unwrap();
                        let value_pair = ((value % N) as Value, (value / N) as Value);
                        self.propagate_value(i, k, value_pair);
                    }
                }
            }
            if k != i {
                if self.constraints[k][j].intersect(mask) != self.constraints[k][j] {
                    self.constraints[k][j] = self.constraints[k][j].intersect(mask);
                    if self.constraints[k][j].is_single() {
                        let value = self.constraints[k][j].into_iter().next().unwrap();
                        let value_pair = ((value % N) as Value, (value / N) as Value);
                        self.propagate_value(k, j, value_pair);
                    }
                }
            }
        }

        // remove value pair from all cells
        for k in 0..N {
            for l in 0..N {
                if k == i && l == j {
                    continue;
                }

                if self.constraints[k][l].contains(value_index) {
                    self.constraints[k][l].remove(value_index);
                    if self.constraints[k][l].is_single() {
                        let value = self.constraints[k][l].into_iter().next().unwrap();
                        let value_pair = ((value % N) as Value, (value / N) as Value);
                        self.propagate_value(k, l, value_pair);
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

    pub fn to_latin_squares(self) -> LatinSquarePair<N> {
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
        let mut counts = vec![0; N * N];
        for i in 0..N {
            for j in 0..N {
                if !self.get(i, j).is_single() {
                    for value in self.get(i, j) {
                        counts[value] += 1;
                    }
                }
            }
        }

        for value in counts
            .into_iter()
            .enumerate()
            .filter(|(_, c)| *c == 1)
            .map(|(i, _)| i)
        {
            for i in 0..N {
                for j in 0..N {
                    if self.get(i, j).contains(value) {
                        let value_pair = ((value % N) as Value, (value / N) as Value);
                        self.propagate_value(i, j, value_pair);
                    }
                }
            }
        }
    }
}
