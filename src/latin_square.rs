use std::fmt::Debug;

use crate::{
    bitset::BitSet, constraints::Constraints, pair_constraints::PairConstraints, types::Value,
};

#[derive(Clone)]
pub struct LatinSquare<const N: usize> {
    values: [[Value; N]; N],
}

pub type LatinSquarePair<const N: usize> = (LatinSquare<N>, LatinSquare<N>);

impl<const N: usize> LatinSquare<N> {
    pub fn get(&self, i: usize, j: usize) -> Value {
        self.values[i][j]
    }

    pub fn is_orthogonal_to(&self, other: &Self) -> bool {
        for value in 0..N as Value {
            let mut other_values = BitSet::empty();

            for i in 0..N {
                for j in 0..N {
                    if self.get(i, j) == value {
                        other_values.insert(other.get(i, j).into());
                    }
                }
            }

            if other_values != BitSet::all_less_than(N) {
                return false;
            }
        }

        true
    }

    pub fn max_disjoint_transversals2(&self) -> usize {
        let mut used = [[false; N]; N];

        let mut count = 0;
        'a: for _ in 0..N {
            let mut transversal = vec![];

            let mut values_left = BitSet::all_less_than(N);
            let mut columns = BitSet::all_less_than(N);

            'row: for row in 0..N {
                for col in columns {
                    let value = self.get(row, col);
                    if !used[row][col] && values_left.contains(value as usize) {
                        columns.remove(col);
                        values_left.remove(value as usize);
                        transversal.push((row, col));
                        used[row][col] = true;
                        continue 'row;
                    }
                }

                break 'a;
            }

            count += 1;
        }

        count
    }

    pub fn max_disjoint_transversals(&self) -> usize {
        let mut stack = vec![(
            0,
            [[false; N]; N],
            0,
            BitSet::all_less_than(N),
            BitSet::all_less_than(N),
            0,
        )];
        let mut max = 0;

        'w: while let Some(state) = stack.last_mut() {
            let (count, mut used, row, mut values, mut columns, col_start) = state.clone();

            for col in columns.intersect(BitSet::all_less_than(col_start).complement()) {
                state.5 = col;
                let value = self.get(row, col);
                if !used[row][col] && values.contains(value as usize) {
                    columns.remove(col);
                    values.remove(value as usize);
                    used[row][col] = true;

                    if row == N - 1 {
                        max = max.max(count + 1);

                        if max == N {
                            return N;
                        }

                        stack.push((
                            count + 1,
                            used,
                            0,
                            BitSet::all_less_than(N),
                            BitSet::all_less_than(N),
                            0,
                        ));
                    } else {
                        stack.push((count, used, row + 1, values, columns, 0));
                    }

                    continue 'w;
                }
            }

            stack.pop();
        }

        max
    }
}

impl<const N: usize> From<PairConstraints<N>> for LatinSquarePair<N> {
    fn from(constraints: PairConstraints<N>) -> Self {
        assert!(constraints.is_solved());

        let mut pair = (
            LatinSquare {
                values: [[0; N]; N],
            },
            LatinSquare {
                values: [[0; N]; N],
            },
        );

        for i in 0..N {
            for j in 0..N {
                let value = constraints.get(i, j).into_iter().next().unwrap();

                let value_pair = ((value % N) as Value, (value / N) as Value);

                pair.0.values[i][j] = value_pair.0;
                pair.1.values[i][j] = value_pair.1;
            }
        }

        pair
    }
}

impl<const N: usize> From<Constraints<N>> for LatinSquare<N> {
    fn from(constraints: Constraints<N>) -> Self {
        assert!(constraints.is_solved());

        let mut square = LatinSquare {
            values: [[0; N]; N],
        };

        for i in 0..N {
            for j in 0..N {
                let value = constraints.get(i, j).into_iter().next().unwrap();

                square.values[i][j] = value as Value;
            }
        }

        square
    }
}

impl<const N: usize> Debug for LatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n")?;
        for i in 0..N {
            write!(f, "    [")?;
            for j in 0..N {
                write!(f, "{:2}, ", self.get(i, j))?;
            }
            write!(f, "]")?;
            if i != N - 1 {
                writeln!(f, ",")?;
            }
        }
        write!(f, "\n]")?;
        Ok(())
    }
}
