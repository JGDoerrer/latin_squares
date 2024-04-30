use std::fmt::Debug;

use crate::{
    bitset::{BitSet128, BitSet16, BitSet192, BitSet64},
    latin_square::{Cell, PartialLatinSquare},
    pair_constraints::ValuePair,
};

pub const N: usize = 10;
pub const MOLS: usize = 8;

const COLUMNS: usize = MOLS + 2;

type BigBitSet = BitSet128;

#[derive(Clone)]
pub struct PartialOrthogonalArray {
    columns: [[Option<u8>; N * N]; COLUMNS],
}

impl PartialOrthogonalArray {
    pub fn new() -> Self {
        PartialOrthogonalArray {
            columns: [[None; N * N]; COLUMNS],
        }
    }

    pub fn squares(&self) -> [PartialLatinSquare<N>; MOLS] {
        let columns = self.columns.map(|col| {
            let mut new_col = [[None; N]; N];

            for i in 0..N {
                for j in 0..N {
                    new_col[i][j] = col[i * N + j];
                }
            }

            PartialLatinSquare::from_array(new_col)
        });

        let mut squares = [PartialLatinSquare::new(); MOLS];

        for i in 2..COLUMNS {
            squares[i - 2] = columns[i];
        }

        squares
    }
}

#[derive(Clone, Debug)]
pub struct OAConstraints {
    oa: PartialOrthogonalArray,
    column_pair_values: [BigBitSet; (COLUMNS * (COLUMNS - 1)) / 2],
    cell_values: [[BitSet16; N * N]; COLUMNS],
    empty_cells: [BigBitSet; COLUMNS],
}

impl OAConstraints {
    pub fn new() -> Self {
        let mut constraints = OAConstraints {
            oa: PartialOrthogonalArray::new(),
            column_pair_values: [BigBitSet::all_less_than(N * N); (COLUMNS * (COLUMNS - 1)) / 2],
            cell_values: [[BitSet16::all_less_than(N); N * N]; COLUMNS],
            empty_cells: [BigBitSet::all_less_than(N * N); COLUMNS],
        };

        let index = N;
        for col in 2..(COLUMNS - 1) {
            let next_col = col + 1;

            let min_val = constraints
                .values_for_cell(Cell(col, index))
                .into_iter()
                .next()
                .or(constraints.oa.columns[col][index].map(|val| val as usize))
                .unwrap();

            dbg!(min_val);

            let mask = BitSet16::all_less_than(min_val + 1).complement();

            let values = &mut constraints.cell_values[next_col][index];
            *values = values.intersect(mask);
        }

        for i in 0..N {
            for j in 0..N {
                constraints.set_and_propagate(Cell(0, i * N + j), i);
            }
        }

        for i in 0..N {
            for j in 0..N {
                constraints.set_and_propagate(Cell(1, i * N + j), j);
            }
        }

        for i in 2..COLUMNS {
            for j in 0..N {
                constraints.set_and_propagate(Cell(i, j), j);
            }
        }

        for j in 1..N {
            constraints.set_and_propagate(Cell(2, j * N), j);
        }

        constraints.find_and_set_singles();

        constraints
    }

    pub fn new_first_row() -> Self {
        let mut constraints = OAConstraints {
            oa: PartialOrthogonalArray::new(),
            column_pair_values: [BigBitSet::all_less_than(N * N); (COLUMNS * (COLUMNS - 1)) / 2],
            cell_values: [[BitSet16::all_less_than(N); N * N]; COLUMNS],
            empty_cells: [BigBitSet::all_less_than(N * N); COLUMNS],
        };

        let index = N;
        for col in 2..(COLUMNS - 1) {
            let next_col = col + 1;

            let min_val = constraints
                .values_for_cell(Cell(col, index))
                .into_iter()
                .next()
                .or(constraints.oa.columns[col][index].map(|val| val as usize))
                .unwrap();

            dbg!(min_val);

            let mask = BitSet16::all_less_than(min_val + 1).complement();

            let values = &mut constraints.cell_values[next_col][index];
            *values = values.intersect(mask);
        }

        for i in 0..N {
            for j in 0..N {
                constraints.set_and_propagate(Cell(0, i * N + j), i);
            }
        }

        for i in 0..N {
            for j in 0..N {
                constraints.set_and_propagate(Cell(1, i * N + j), j);
            }
        }

        for i in 2..COLUMNS {
            for j in 0..N {
                constraints.set_and_propagate(Cell(i, j), j);
            }
        }

        constraints.find_and_set_singles();

        constraints
    }

    pub fn squares(&self) -> [PartialLatinSquare<N>; MOLS] {
        self.oa.squares()
    }

    fn get_column_pair_values_mut(&mut self, column1: usize, column2: usize) -> &mut BigBitSet {
        let min = column1.min(column2);
        let max = column1.max(column2);

        assert!(min != max);

        let mut index = max - 1;
        for i in 0..min {
            index += COLUMNS - i - 2;
        }
        //  match min {
        //     0 => max - 1,
        //     1 => max + 2,
        //     2 => max + 4,
        //     3 => max + 5,
        // };

        &mut self.column_pair_values[index]
    }

    fn get_column_pair_values(&self, column1: usize, column2: usize) -> &BigBitSet {
        let min = column1.min(column2);
        let max = column1.max(column2);

        assert!(min != max);

        let mut index = max - 1;
        for i in 0..min {
            index += COLUMNS - i - 2;
        }
        // let index = match min {
        //     0 => max - 1,
        //     1 => max + 2,
        //     2 => max + 4,
        //     3 => max + 5,
        //     _ => unreachable!(),
        // };

        &self.column_pair_values[index]
    }

    fn get_value_pair(
        &self,
        column1: usize,
        column2: usize,
        index: usize,
    ) -> (Option<u8>, Option<u8>) {
        let min = column1.min(column2);
        let max = column1.max(column2);

        assert!(min != max);

        (self.oa.columns[min][index], self.oa.columns[max][index])
    }

    pub fn set_and_propagate(&mut self, cell: Cell, value: usize) {
        self.set(cell, value);
        self.propagate_constraints();
    }

    fn set(&mut self, cell: Cell, value: usize) {
        let Cell(column, index) = cell;

        assert!(
            self.cell_values[column][index].contains(value),
            "{:?}, {column}, {index}, {value}, {:?}",
            self.oa,
            self.cell_values[column][index]
        );
        assert!(
            self.empty_cells[column].contains(index),
            "{:?}, {column}, {index}, {value}, {:?}",
            self.oa,
            self.empty_cells[column]
        );

        self.oa.columns[column][index] = Some(value as u8);
        self.empty_cells[column].remove(index);
        self.cell_values[column][index] = BitSet16::empty();

        for i in 0..COLUMNS {
            if i == column {
                continue;
            }

            let value_pair = self.get_value_pair(column, i, index);
            let pair = self.get_column_pair_values_mut(column, i);

            if let (Some(v1), Some(v2)) = value_pair {
                pair.remove(ValuePair(v1 as usize, v2 as usize).to_index::<N>());
            }
        }
    }

    fn propagate_constraints(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            for first_column in 0..COLUMNS {
                for second_column in (first_column + 1)..COLUMNS {
                    let mut second_vals_for_first_val = [BitSet16::empty(); N];
                    let mut first_vals_for_second_val = [BitSet16::empty(); N];

                    let pair = self.get_column_pair_values_mut(first_column, second_column);

                    for index in *pair {
                        let ValuePair(first_val, second_val) = ValuePair::from_index::<N>(index);

                        second_vals_for_first_val[first_val].insert(second_val);
                        first_vals_for_second_val[second_val].insert(first_val);
                    }

                    for index in self.empty_cells[first_column]
                        .complement()
                        .intersect(BigBitSet::all_less_than(N * N))
                    {
                        let first_value = self.oa.columns[first_column][index].unwrap() as usize;

                        let second_values = &mut self.cell_values[second_column][index];
                        if !second_values.is_subset_of(second_vals_for_first_val[first_value]) {
                            *second_values =
                                second_values.intersect(second_vals_for_first_val[first_value]);
                            changed = true;
                        }
                    }

                    for index in self.empty_cells[second_column]
                        .complement()
                        .intersect(BigBitSet::all_less_than(N * N))
                    {
                        let second_value = self.oa.columns[second_column][index].unwrap() as usize;

                        let first_values = &mut self.cell_values[first_column][index];
                        if !first_values.is_subset_of(first_vals_for_second_val[second_value]) {
                            *first_values =
                                first_values.intersect(first_vals_for_second_val[second_value]);
                            changed = true;
                        }
                    }
                }
            }

            let index = N;
            for col in 2..(COLUMNS - 1) {
                let next_col = col + 1;

                let min_val = self
                    .values_for_cell(Cell(col, index))
                    .into_iter()
                    .next()
                    .or(self.oa.columns[col][index].map(|val| val as usize))
                    .unwrap();

                let mask = BitSet16::all_less_than(min_val + 1).complement();

                let values = &mut self.cell_values[next_col][index];
                *values = values.intersect(mask);
            }
        }
    }

    pub fn values_for_cell(&self, cell: Cell) -> BitSet16 {
        let Cell(column, index) = cell;
        self.cell_values[column][index]
    }

    pub fn cells_for_pair(
        &self,
        column1: usize,
        column2: usize,
        value_pair: ValuePair,
    ) -> BigBitSet {
        let pair = self.get_column_pair_values(column1, column2);

        let min = column1.min(column2);
        let max = column1.max(column2);

        let value_pair_index = value_pair.to_index::<N>();
        assert!(pair.contains(value_pair_index));

        let mut cells = BigBitSet::empty();

        let mut first_values = [false; N * N];
        for i in 0..N * N {
            first_values[i] = if let Some(val) = self.oa.columns[min][i] {
                val as usize == value_pair.0
            } else {
                self.values_for_cell(Cell(min, i)).contains(value_pair.0)
            };
        }

        let mut second_values = [false; N * N];
        for i in 0..N * N {
            second_values[i] = if let Some(val) = self.oa.columns[max][i] {
                val as usize == value_pair.1
            } else {
                self.values_for_cell(Cell(max, i)).contains(value_pair.1)
            };
        }

        let cells: BigBitSet = first_values
            .into_iter()
            .zip(second_values.into_iter())
            .map(|(a, b)| a && b)
            .enumerate()
            .filter(|(_, b)| *b)
            .map(|(i, _)| i)
            .collect();

        // for index in self.empty_cells[column1].union(self.empty_cells[column2]) {
        //     match (self.oa.columns[min][index], self.oa.columns[max][index]) {
        //         (None, None) => {
        //             if self
        //                 .values_for_cell(Cell(min, index))
        //                 .contains(value_pair.0)
        //                 && self
        //                     .values_for_cell(Cell(max, index))
        //                     .contains(value_pair.1)
        //             {
        //                 cells.insert(index);
        //             }
        //         }
        //         (None, Some(v2)) => {
        //             if v2 as usize == value_pair.1
        //                 && self
        //                     .values_for_cell(Cell(min, index))
        //                     .contains(value_pair.0)
        //             {
        //                 cells.insert(index);
        //             }
        //         }
        //         (Some(v1), None) => {
        //             if v1 as usize == value_pair.0
        //                 && self
        //                     .values_for_cell(Cell(max, index))
        //                     .contains(value_pair.1)
        //             {
        //                 cells.insert(index);
        //             }
        //         }
        //         _ => unreachable!(),
        //     }
        // }
        cells
    }

    pub fn pairs_for_cell(&self, column1: usize, column2: usize, index: usize) -> BigBitSet {
        let pair = self.get_column_pair_values(column1, column2);

        let min = column1.min(column2);
        let max = column1.max(column2);

        let values1 = if let Some(v1) = self.oa.columns[min][index] {
            BitSet16::single(v1 as usize)
        } else {
            self.values_for_cell(Cell(min, index))
        };
        let values2 = if let Some(v2) = self.oa.columns[max][index] {
            BitSet16::single(v2 as usize)
        } else {
            self.values_for_cell(Cell(max, index))
        };

        let mut pairs = BigBitSet::empty();
        for value1 in values1 {
            for value2 in values2 {
                let value_pair = ValuePair(value1, value2);
                let value_pair_index = value_pair.to_index::<N>();

                if pair.contains(value_pair_index) {
                    pairs.insert(value_pair_index);
                }
            }
        }

        pairs
    }

    pub fn most_constrained_cell(&self) -> Option<Cell> {
        let mut min = N + 1;
        let mut min_dist = N * N;
        let mut min_cell = Cell(0, 0);

        for column in 0..COLUMNS {
            for index in self.empty_cells[column] {
                let cell = Cell(column, index);

                let row = index / N;
                let col = index % N;
                let dist = (row * N + col).min(row + col * N);

                let len = self.values_for_cell(cell).len();
                if len.cmp(&min).then(dist.cmp(&min_dist)).is_le() {
                    min = len;
                    min_dist = dist;
                    min_cell = cell;
                }
            }
        }

        (min != N + 1).then(|| min_cell)
    }

    pub fn is_solvable(&self) -> bool {
        self.is_solvable_rec(0)
    }

    fn is_solvable_rec(&self, max_depth: usize) -> bool {
        for col in 0..COLUMNS {
            for index in self.empty_cells[col] {
                let cell = Cell(col, index);
                if self.values_for_cell(cell).is_empty() {
                    return false;
                }
            }

            for col2 in (col + 1)..COLUMNS {
                let pair = self.get_column_pair_values(col, col2);

                for value_pair in *pair {
                    let value_pair = ValuePair::from_index::<N>(value_pair);
                    let cells_for_pair = self.cells_for_pair(col, col2, value_pair);
                    if cells_for_pair.is_empty() {
                        return false;
                    }
                }

                for index in self.empty_cells[col].union(self.empty_cells[col2]) {
                    let pairs_for_cell = self.pairs_for_cell(col, col2, index);
                    if pairs_for_cell.is_empty() {
                        return false;
                    }
                }
            }
        }

        if max_depth > 0 {
            let mut cells = Vec::new();
            for col in 0..COLUMNS {
                for index in self.empty_cells[col] {
                    let cell = Cell(col, index);
                    let len = self.values_for_cell(cell).len();

                    cells.push((cell, len));
                }
            }

            cells.retain(|(_, len)| *len <= 3);
            cells.sort_by_key(|(_, len)| *len);

            for (cell, _) in cells {
                if self.values_for_cell(cell).into_iter().all(|value| {
                    let mut copy = self.clone();
                    copy.set_and_propagate(cell, value);
                    copy.find_and_set_singles();

                    !copy.is_solvable_rec(max_depth - 1)
                }) {
                    return false;
                }
            }

            for col1 in 0..COLUMNS {
                for col2 in (col1 + 1)..COLUMNS {
                    let pairs = *self.get_column_pair_values(col1, col2);

                    for pair in pairs {
                        let value_pair = ValuePair::from_index::<N>(pair);
                        let cells_for_pair = self.cells_for_pair(col1, col2, value_pair);
                        if cells_for_pair.len() <= 4
                            && cells_for_pair.into_iter().all(|cell| {
                                let mut copy = self.clone();
                                if !copy
                                    .values_for_cell(Cell(col1, cell))
                                    .contains(value_pair.0)
                                {
                                    return false;
                                }
                                copy.set_and_propagate(Cell(col1, cell), value_pair.0);
                                if !copy
                                    .values_for_cell(Cell(col2, cell))
                                    .contains(value_pair.1)
                                {
                                    return false;
                                }
                                copy.set_and_propagate(Cell(col2, cell), value_pair.1);
                                copy.find_and_set_singles();

                                !copy.is_solvable_rec(max_depth - 1)
                            })
                        {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    pub fn find_and_set_singles(&mut self) {
        let mut changed = true;

        while changed {
            changed = false;
            for col in 0..COLUMNS {
                for index in self.empty_cells[col] {
                    let cell = Cell(col, index);

                    if self.values_for_cell(cell).is_single() {
                        let value = self.values_for_cell(cell).into_iter().next().unwrap();
                        self.set_and_propagate(cell, value);
                        changed = true;
                    }
                }
            }

            for column1 in 0..COLUMNS {
                for column2 in (column1 + 1)..COLUMNS {
                    let pair = self.get_column_pair_values(column1, column2);

                    for value_pair in *pair {
                        let value_pair = ValuePair::from_index::<N>(value_pair);

                        let cells_for_pair = self.cells_for_pair(column1, column2, value_pair);
                        if cells_for_pair.is_single() {
                            let cell = cells_for_pair.into_iter().next().unwrap();

                            if self.empty_cells[column1].contains(cell) {
                                self.set_and_propagate(Cell(column1, cell), value_pair.0);
                                changed = true;
                            }
                            if self.empty_cells[column2].contains(cell) {
                                self.set_and_propagate(Cell(column2, cell), value_pair.1);
                                changed = true;
                            }
                            break;
                        }
                    }

                    for index in self.empty_cells[column1].union(self.empty_cells[column2]) {
                        let pairs_for_cell = self.pairs_for_cell(column1, column2, index);

                        if pairs_for_cell.is_single() {
                            let pair_index = pairs_for_cell.into_iter().next().unwrap();
                            let value_pair = ValuePair::from_index::<N>(pair_index);

                            if self.empty_cells[column1].contains(index) {
                                self.set_and_propagate(Cell(column1, index), value_pair.0);
                                changed = true;
                            }
                            if self.empty_cells[column2].contains(index) {
                                self.set_and_propagate(Cell(column2, index), value_pair.1);
                                changed = true;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn filled_cells(&self) -> usize {
        self.empty_cells
            .map(|bitset| {
                bitset
                    .complement()
                    .intersect(BigBitSet::all_less_than(N * N))
                    .len()
            })
            .into_iter()
            .sum()
    }

    pub fn sum_possible_values(&self) -> f64 {
        self.cell_values
            .iter()
            .map(|col| col.iter().map(|val| (val.len() as f64).log2()).sum::<f64>())
            .sum()
    }

    pub fn is_solved(&self) -> bool {
        self.empty_cells
            .map(|bitset| bitset.len())
            .into_iter()
            .sum::<usize>()
            == 0
    }
}

impl Debug for PartialOrthogonalArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n")?;
        for i in 0..N * N {
            write!(f, "    [")?;
            for j in 0..COLUMNS {
                if let Some(value) = self.columns[j][i] {
                    write!(f, "{:2}, ", value)?;
                } else {
                    write!(f, "??, ")?;
                }
            }
            write!(f, "]")?;
            if i != N * N - 1 {
                writeln!(f, ",")?;
            }
        }
        write!(f, "\n]")?;
        Ok(())
    }
}
