use std::fmt::Debug;

use crate::{bitset::BitSet, constraints::Constraints, pair_constraints::PairConstraints};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LatinSquare<const N: usize> {
    values: [[usize; N]; N],
}

pub type LatinSquarePair<const N: usize> = (LatinSquare<N>, LatinSquare<N>);

#[derive(Debug, Clone, Copy)]
pub struct Cell(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub struct ValuePair(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub enum CellOrValuePair {
    Cell(Cell),
    ValuePair(ValuePair),
}

impl<const N: usize> LatinSquare<N> {
    pub fn get(&self, i: usize, j: usize) -> usize {
        self.values[i][j]
    }

    pub fn is_orthogonal_to(&self, other: &Self) -> bool {
        for value in 0..N as usize {
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
                let value = constraints
                    .values_for_cell(i, j)
                    .into_iter()
                    .next()
                    .unwrap();

                let value_pair = ((value % N) as usize, (value / N) as usize);

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

                square.values[i][j] = value as usize;
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

#[derive(Clone)]
pub struct PartialLatinSquare<const N: usize> {
    values: [[Option<usize>; N]; N],
}

pub type PartialLatinSquarePair<const N: usize> = (PartialLatinSquare<N>, PartialLatinSquare<N>);

impl<const N: usize> PartialLatinSquare<N> {
    pub fn new() -> Self {
        PartialLatinSquare {
            values: [[None; N]; N],
        }
    }

    pub fn get(&self, i: usize, j: usize) -> Option<usize> {
        self.values[i][j]
    }

    pub fn set(&mut self, i: usize, j: usize, value: usize) {
        self.values[i][j] = Some(value);
    }

    pub fn next_unknown(&self) -> Option<(usize, usize)> {
        for j in 0..(N + 1) / 2 {
            for j in [j, N - j - 1] {
                for i in 0..N {
                    if self.get(j, i).is_none() {
                        return Some((j, i));
                    }
                }
                for i in 0..N {
                    if self.get(i, j).is_none() {
                        return Some((i, j));
                    }
                }
            }
        }
        None
    }
}

impl<const N: usize> From<PartialLatinSquare<N>> for LatinSquare<N> {
    fn from(value: PartialLatinSquare<N>) -> Self {
        let mut sq = LatinSquare {
            values: [[0; N]; N],
        };

        for i in 0..N {
            for j in 0..N {
                sq.values[i][j] = value.get(i, j).unwrap();
            }
        }

        sq
    }
}

impl<const N: usize> Debug for PartialLatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n")?;
        for i in 0..N {
            write!(f, "    [")?;
            for j in 0..N {
                if let Some(value) = self.get(i, j) {
                    write!(f, "{:2}, ", value)?;
                } else {
                    write!(f, "??, ")?;
                }
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
