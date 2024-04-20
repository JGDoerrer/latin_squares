use std::fmt::Debug;

use crate::{
    bitset::{BitSet128, BitSet16},
    latin_square::{Cell, PartialLatinSquare},
};

pub const N: usize = 10;

#[derive(Clone)]
pub struct PartialOrthogonalArray {
    columns: [[Option<u8>; N * N]; 5],
}

impl PartialOrthogonalArray {
    pub fn new() -> Self {
        PartialOrthogonalArray {
            columns: [[None; N * N]; 5],
        }
    }

    pub fn squares(&self) -> [PartialLatinSquare<N>; 3] {
        let arrays = self.columns.map(|col| {
            let mut new_col = [[None; N]; N];

            for i in 0..N {
                for j in 0..N {
                    new_col[i][j] = col[i * N + j];
                }
            }

            new_col
        });

        [
            PartialLatinSquare::from_array(arrays[2]),
            PartialLatinSquare::from_array(arrays[3]),
            PartialLatinSquare::from_array(arrays[4]),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct OAConstraints {
    oa: PartialOrthogonalArray,
    column_pair_values: [BitSet128; 10],
    cell_values: [[BitSet16; N * N]; 5],
    empty_cells: [BitSet128; 5],
}

impl OAConstraints {
    pub fn new() -> Self {
        let mut constraints = OAConstraints {
            oa: PartialOrthogonalArray::new(),
            column_pair_values: [BitSet128::all_less_than(N * N); 10],
            cell_values: [[BitSet16::all_less_than(N); N * N]; 5],
            empty_cells: [BitSet128::all_less_than(N * N); 5],
        };

        for i in 0..N {
            for j in 0..N {
                constraints.set(Cell(0, i * N + j), i);
            }
        }

        for i in 0..N {
            for j in 0..N {
                constraints.set(Cell(1, i * N + j), j);
            }
        }

        for i in 2..5 {
            for j in 0..N {
                constraints.set(Cell(i, j), j);
            }
        }

        for j in 1..N {
            constraints.set(Cell(2, j * N), j);
        }

        constraints
    }

    pub fn squares(&self) -> [PartialLatinSquare<N>; 3] {
        self.oa.squares()
    }

    fn get_column_pair_values(&mut self, column1: usize, column2: usize) -> &mut BitSet128 {
        let min = column1.min(column2);
        let max = column1.max(column2);

        assert!(min != max);

        let index = match min {
            0 => max - 1,
            1 => max + 2,
            2 => max + 4,
            3 => max + 5,
            _ => unreachable!(),
        };

        &mut self.column_pair_values[index]
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

    pub fn set(&mut self, cell: Cell, value: usize) {
        let Cell(column, index) = cell;

        assert!(
            self.cell_values[column][index].contains(value),
            "{:?}",
            self.oa
        );

        self.oa.columns[column][index] = Some(value as u8);
        self.empty_cells[column].remove(index);
        self.cell_values[column][index] = BitSet16::empty();

        for i in 0..5 {
            if i == column {
                continue;
            }

            let value_pair = self.get_value_pair(column, i, index);
            let pair = self.get_column_pair_values(column, i);
            let first_column = column.min(i);
            let second_column = column.max(i);

            if let (Some(v1), Some(v2)) = value_pair {
                pair.remove(v1 as usize * N + v2 as usize);
            }

            let mut second_vals_for_first_val = [BitSet16::empty(); N];
            let mut first_vals_for_second_val = [BitSet16::empty(); N];

            for index in *pair {
                let first_val = index % N;
                let second_val = index / N;

                first_vals_for_second_val[first_val].insert(second_val);
                second_vals_for_first_val[second_val].insert(first_val);
            }

            for index in self.empty_cells[first_column]
                .complement()
                .intersect(BitSet128::all_less_than(N * N))
            {
                let first_value = self.oa.columns[first_column][index].unwrap() as usize;

                let second_values = &mut self.cell_values[second_column][index];
                *second_values = second_values.intersect(second_vals_for_first_val[first_value]);
            }

            for index in self.empty_cells[second_column]
                .complement()
                .intersect(BitSet128::all_less_than(N * N))
            {
                let second_value = self.oa.columns[second_column][index].unwrap() as usize;

                let first_values = &mut self.cell_values[first_column][index];
                *first_values = first_values.intersect(first_vals_for_second_val[second_value]);
            }
        }
    }

    pub fn values_for_cell(&self, cell: Cell) -> BitSet16 {
        let Cell(column, index) = cell;
        self.cell_values[column][index]
    }

    pub fn most_constrained_cell(&mut self) -> Option<Cell> {
        let mut min = N + 1;
        let mut min_cell = Cell(0, 0);

        for column in 0..5 {
            for index in self.empty_cells[column] {
                let cell = Cell(column, index);

                let len = self.values_for_cell(cell).len();
                if len <= min {
                    min = len;
                    min_cell = cell;
                }
            }
        }

        (min != N + 1).then(|| min_cell)
    }

    pub fn is_solvable(&self) -> bool {
        self.is_solvable_rec(7)
    }

    fn is_solvable_rec(&self, max_depth: usize) -> bool {
        for col in 0..5 {
            for index in self.empty_cells[col] {
                let cell = Cell(col, index);
                if self.values_for_cell(cell).is_empty() {
                    return false;
                }
            }
        }

        if max_depth > 0 {
            for col in 0..5 {
                for index in self.empty_cells[col] {
                    let cell = Cell(col, index);
                    let len = self.values_for_cell(cell).len();
                    if len < 4
                        && self.values_for_cell(cell).into_iter().all(|value| {
                            let mut copy = self.clone();
                            copy.set(cell, value);
                            copy.find_and_set_singles();

                            !copy.is_solvable_rec(max_depth - 1)
                        })
                    {
                        return false;
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
            for col in 0..5 {
                for index in self.empty_cells[col] {
                    let cell = Cell(col, index);

                    if self.values_for_cell(cell).is_single() {
                        let value = self.values_for_cell(cell).into_iter().next().unwrap();
                        self.set(cell, value);
                        changed = true;
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
                    .intersect(BitSet128::all_less_than(N * N))
                    .len()
            })
            .into_iter()
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
            for j in 0..5 {
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
