use std::fmt::Debug;

use crate::{
    bitset::{BitSet128, BitSet16},
    constraints::Constraints,
    latin_square_oa_generator::LatinSquareOAGenerator,
    pair_constraints::{PairConstraints, ValuePair},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LatinSquare<const N: usize> {
    values: [[u8; N]; N],
}

#[derive(Debug, Clone, Copy)]
pub struct Cell(pub usize, pub usize);

pub type LatinSquarePair<const N: usize> = (LatinSquare<N>, LatinSquare<N>);

impl<const N: usize> LatinSquare<N> {
    pub fn new(values: [[u8; N]; N]) -> Self {
        let sq = LatinSquare { values };
        debug_assert!(sq.is_valid());
        sq
    }

    pub fn get(&self, i: usize, j: usize) -> usize {
        self.values[i][j] as usize
    }

    pub fn get_row(&self, i: usize) -> [u8; N] {
        self.values[i]
    }

    pub fn is_valid(&self) -> bool {
        (0..N).all(|i| {
            (0..N).map(|j| self.get(i, j)).collect::<BitSet16>() == BitSet16::all_less_than(N)
                && (0..N).map(|j| self.get(j, i)).collect::<BitSet16>()
                    == BitSet16::all_less_than(N)
        })
    }

    pub fn is_orthogonal_to(&self, other: &Self) -> bool {
        for value in 0..N {
            let mut other_values = BitSet128::empty();

            for i in 0..N {
                for j in 0..N {
                    if self.get(i, j) == value {
                        other_values.insert(other.get(i, j));
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

    pub fn unavoidable_sets(&self) -> Vec<Vec<BitSet128>> {
        let mut order1 = self.unavoidable_sets_order_1();

        order1 = order1
            .iter()
            .filter(|set| {
                order1
                    .iter()
                    .all(|other| other == *set || !other.is_subset_of(**set))
            })
            .copied()
            .collect();

        order1.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        order1.dedup();

        let mut order2 = vec![];
        for (i, set1) in order1.iter().enumerate() {
            for set2 in order1.iter().skip(i + 1) {
                let new_set = set1.union(*set2);
                if set1.is_disjoint(*set2)
                    && order2.iter().all(|set| !new_set.is_subset_of(*set))
                    && new_set.len() <= 3 * N
                {
                    order2.push(new_set);
                }
            }
        }

        order2.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        order2.dedup();

        let mut all_orders = vec![order1.clone(), order2];

        while all_orders.last().is_some_and(|sets| !sets.is_empty()) {
            let last_order = all_orders.last().unwrap();
            let mut next_order = vec![];

            for set1 in &order1 {
                for set2 in last_order {
                    let new_set = set1.union(*set2);
                    if set1.is_disjoint(*set2) && new_set.len() <= all_orders.len() * N
                    // && last_order.iter().all(|set| !new_set.is_subset_of(*set))
                    {
                        next_order.push(new_set);
                    }
                }
            }

            next_order.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
            next_order.dedup();

            all_orders.push(next_order);
        }

        let all_orders = vec![order1];
        all_orders
    }

    pub fn unavoidable_sets_order_1(&self) -> Vec<BitSet128> {
        debug_assert!(self.is_reduced());

        let mut sets = Vec::new();

        let triple_iter = (0..N).flat_map(|first| {
            ((first + 1)..N)
                .flat_map(move |second| ((second + 1)..N).map(move |third| [first, second, third]))
        });

        for triple in triple_iter {
            for partial in [
                self.without_rows(triple),
                self.without_cols(triple),
                self.without_vals(triple),
            ] {
                let solutions = LatinSquareOAGenerator::from_partial(partial).map(|s| s[0]);

                for solution in solutions {
                    let difference = self.difference_mask(&solution);

                    if !difference.is_empty() {
                        sets.push(difference);
                    }
                }
            }
        }

        // for [row1, row2, col1, col2] in self.subsquares_order_2_iter() {
        //     let set = [
        //         Cell(row1, col1),
        //         Cell(row1, col2),
        //         Cell(row2, col1),
        //         Cell(row2, col2),
        //     ]
        //     .map(|cell| cell.to_index::<N>())
        //     .into_iter()
        //     .collect();
        //     sets.push(set);
        // }

        sets
    }

    pub fn subsquares_order_2_iter(&self) -> impl Iterator<Item = [usize; 4]> + '_ {
        let rows_iter = (0..N).flat_map(|row1| ((row1 + 1)..N).map(move |row2| (row1, row2)));

        rows_iter.flat_map(|(row1, row2)| {
            let cols_iter = (0..N).flat_map(|col1| ((col1 + 1)..N).map(move |col2| (col1, col2)));
            cols_iter
                .map(move |(col1, col2)| [row1, row2, col1, col2])
                .filter(|[row1, row2, col1, col2]| {
                    self.get(*row1, *col1) == self.get(*row2, *col2)
                        && self.get(*row1, *col2) == self.get(*row2, *col1)
                })
        })
    }

    pub fn without_rows(&self, rows: impl IntoIterator<Item = usize>) -> PartialLatinSquare<N> {
        let mut sq = PartialLatinSquare::from(*self);
        for row in rows {
            for i in 0..N {
                sq.set(row, i, None);
            }
        }
        sq
    }

    pub fn without_cols(&self, cols: impl IntoIterator<Item = usize>) -> PartialLatinSquare<N> {
        let mut sq = PartialLatinSquare::from(*self);
        for col in cols {
            for i in 0..N {
                sq.set(i, col, None);
            }
        }
        sq
    }

    pub fn without_vals(&self, vals: impl IntoIterator<Item = usize>) -> PartialLatinSquare<N> {
        let mut sq = PartialLatinSquare::from(*self);
        for value in vals {
            for i in 0..N {
                for j in 0..N {
                    if self.get(i, j) == value {
                        sq.set(i, j, None);
                    }
                }
            }
        }
        sq
    }

    pub fn difference_mask(&self, other: &Self) -> BitSet128 {
        let mut mask = BitSet128::empty();

        for i in 0..N {
            for j in 0..N {
                if self.get(i, j) != other.get(i, j) {
                    mask.insert(Cell(i, j).to_index::<N>());
                }
            }
        }

        mask
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

impl<const N: usize> Debug for LatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PartialLatinSquare<const N: usize> {
    values: [[Option<u8>; N]; N],
}

impl<const N: usize> Default for PartialLatinSquare<N> {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn set(&mut self, i: usize, j: usize, value: Option<usize>) {
        self.values[i][j] = value.map(|v| v as u8);
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

    pub fn num_entries(&self) -> usize {
        (0..N)
            .map(|row| {
                (0..N)
                    .filter(|col| self.get(Cell(row, *col)).is_some())
                    .count()
            })
            .sum()
    }

    pub fn num_empty_rows(&self) -> usize {
        (0..N)
            .filter(|row| (0..N).all(|col| self.get(Cell(*row, col)).is_none()))
            .count()
    }

    pub fn num_full_rows(&self) -> usize {
        (0..N)
            .filter(|row| (0..N).all(|col| self.get(Cell(*row, col)).is_some()))
            .count()
    }

    pub fn num_empty_cols(&self) -> usize {
        (0..N)
            .filter(|col| (0..N).all(|row| self.get(Cell(row, *col)).is_none()))
            .count()
    }

    pub fn num_full_cols(&self) -> usize {
        (0..N)
            .filter(|col| (0..N).all(|row| self.get(Cell(row, *col)).is_some()))
            .count()
    }

    pub fn num_unique_values(&self) -> usize {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(Cell(row, col))))
            .flatten()
            .collect::<BitSet16>()
            .len()
    }

    pub fn first_empty_index(&self) -> Option<usize> {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(Cell(row, col))))
            .position(|entry| entry.is_none())
    }

    pub fn next_empty_index(&self, start: usize) -> Option<usize> {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(Cell(row, col))))
            .skip(start)
            .position(|entry| entry.is_none())
            .map(|index| index + start)
    }

    pub fn num_next_empty_indices(&self, start: usize) -> usize {
        (0..N)
            .flat_map(|row| (0..N).map(move |col| self.get(Cell(row, col))))
            .skip(start)
            .filter(|entry| entry.is_none())
            .count()
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

impl<const N: usize> From<LatinSquare<N>> for PartialLatinSquare<N> {
    fn from(value: LatinSquare<N>) -> Self {
        let mut sq = PartialLatinSquare {
            values: [[None; N]; N],
        };

        for i in 0..N {
            for j in 0..N {
                sq.values[i][j] = Some(value.get(i, j) as u8);
            }
        }

        sq
    }
}

impl<const N: usize> Debug for PartialLatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
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
