use std::fmt::Debug;

use crate::{
    bitset::BitSet128,
    constraints::Constraints,
    pair_constraints::{PairConstraints, ValuePair},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LatinSquare<const N: usize> {
    pub values: [[u8; N]; N],
}

#[derive(Debug, Clone, Copy)]
pub struct Cell(pub usize, pub usize);

pub type LatinSquarePair<const N: usize> = (LatinSquare<N>, LatinSquare<N>);

impl<const N: usize> LatinSquare<N> {
    pub fn new(values: [[u8; N]; N]) -> Self {
        LatinSquare { values }
    }

    pub fn get(&self, i: usize, j: usize) -> usize {
        self.values[i][j] as usize
    }

    pub fn is_orthogonal_to(&self, other: &Self) -> bool {
        for value in 0..N as usize {
            let mut other_values = BitSet128::empty();

            for i in 0..N {
                for j in 0..N {
                    if self.get(i, j) == value {
                        other_values.insert(other.get(i, j).into());
                    }
                }
            }

            if other_values != BitSet128::all_less_than(N) {
                return false;
            }
        }

        true
    }

    pub fn is_reduced(&self) -> bool {
        for i in 0..N {
            if self.values[0][i] != i as u8 || self.values[i][0] != i as u8 {
                return false;
            }
        }
        true
    }

    pub fn is_isotopic_to(&self, other: &Self) -> bool {
        if self.values[0] == other.values[0] {
            // is it enough to check the rows?
            for row in 1..N {
                let first_value = self.values[row][0];

                let other_row = other
                    .values
                    .iter()
                    .find(|row| row[0] == first_value)
                    .unwrap();

                if self.values[row] != *other_row {
                    return false;
                }
            }
            true
        } else {
            todo!()
        }
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

                let value_pair = ValuePair::from_index::<N>(value);

                pair.0.values[i][j] = value_pair.0 as u8;
                pair.1.values[i][j] = value_pair.1 as u8;
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

                square.values[i][j] = value as u8;
            }
        }

        square
    }
}

// impl<const N: usize> Debug for LatinSquare<N> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[\n")?;
//         for i in 0..N {
//             write!(f, "    [")?;
//             for j in 0..N {
//                 write!(f, "{:2}, ", self.get(i, j))?;
//             }
//             write!(f, "]")?;
//             if i != N - 1 {
//                 writeln!(f, ",")?;
//             }
//         }
//         write!(f, "\n]")?;
//         Ok(())
//     }
// }

#[derive(Clone, Copy)]
pub struct PartialLatinSquare<const N: usize> {
    values: [[Option<u8>; N]; N],
}

impl<const N: usize> PartialLatinSquare<N> {
    pub fn new() -> Self {
        PartialLatinSquare {
            values: [[None; N]; N],
        }
    }

    pub fn from_array(values: [[Option<u8>; N]; N]) -> Self {
        PartialLatinSquare { values }
    }

    pub fn get(&self, cell: Cell) -> Option<usize> {
        self.values[cell.0][cell.1].map(|val| val.into())
    }

    pub fn set(&mut self, i: usize, j: usize, value: usize) {
        self.values[i][j] = Some(value as u8);
    }

    pub fn next_unknown(&self) -> Option<(usize, usize)> {
        for j in 0..(N + 1) / 2 {
            for j in [j, N - j - 1] {
                for i in 0..N {
                    if self.get(Cell(j, i)).is_none() {
                        return Some((j, i));
                    }
                }
                for i in 0..N {
                    if self.get(Cell(i, j)).is_none() {
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
                sq.values[i][j] = value.get(Cell(i, j)).unwrap() as u8;
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
                if let Some(value) = self.get(Cell(i, j)) {
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

impl Cell {
    pub fn to_index<const N: usize>(self) -> usize {
        self.0 * N + self.1
    }
    pub fn from_index<const N: usize>(value: usize) -> Self {
        Cell(value / N, value % N)
    }
}
