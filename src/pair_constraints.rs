use crate::{
    bitset::BitSet128,
    latin_square::{Cell, LatinSquarePair, PartialLatinSquare},
    latin_square_pair_generator::PartialLatinSquarePair,
};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct PairConstraints<const N: usize> {
    sq_pair: PartialLatinSquarePair<N>,
    values_left: BitSet128,
    empty_cells: BitSet128,
    rows: [BitSet128; N],
    cols: [BitSet128; N],
    first_values: [BitSet128; N],
    second_values: [BitSet128; N],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValuePair(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub enum CellOrValuePair {
    Cell(Cell),
    ValuePair(ValuePair),
}

impl<const N: usize> PairConstraints<N> {
    const VALUE_PAIRS_WITHOUT_FIRST: [BitSet128; N] = {
        let mut table = [BitSet128::empty(); N];

        let every_nth = {
            let mut num = 1u128;
            let mut i = 0;
            while i < N {
                num |= num << N;
                i += 1;
            }
            num
        };

        let mut i = 0;
        while i < N {
            table[i] = BitSet128::from_bits(BitSet128::single(i).bits() * every_nth).complement();
            i += 1;
        }

        table
    };
    const VALUE_PAIRS_WITHOUT_SECOND: [BitSet128; N] = {
        let mut table = [BitSet128::empty(); N];

        let mut i = 0;
        while i < N {
            table[i] =
                BitSet128::from_bits(BitSet128::all_less_than(N).bits() << (i * N)).complement();
            i += 1;
        }

        table
    };
    const CELLS_WITHOUT_ROW: [BitSet128; N] = Self::VALUE_PAIRS_WITHOUT_SECOND;
    const CELLS_WITHOUT_COLUMN: [BitSet128; N] = Self::VALUE_PAIRS_WITHOUT_FIRST;

    const RELATIONS: [[BitSet128; 4]; 3] = [
        [
            BitSet128::from_slice(&[
                00, 11, 12, 13, 21, 22, 23, 31, 32, 33, 44, 45, 46, 54, 55, 56, 64, 65, 66, 77, 78,
                79, 87, 88, 89, 97, 98, 99,
            ]),
            BitSet128::from_slice(&[
                01, 02, 03, 10, 20, 30, 47, 48, 49, 57, 58, 59, 67, 68, 69, 74, 75, 76, 84, 85, 86,
                94, 95, 96,
            ]),
            BitSet128::from_slice(&[
                04, 05, 06, 17, 18, 19, 27, 28, 29, 37, 38, 39, 40, 50, 60, 71, 72, 73, 81, 82, 83,
                91, 92, 93,
            ]),
            BitSet128::from_slice(&[
                07, 08, 09, 14, 15, 16, 24, 25, 26, 34, 35, 36, 41, 42, 43, 51, 52, 53, 61, 62, 63,
                70, 80, 90,
            ]),
        ],
        [
            BitSet128::from_slice(&[
                00, 01, 10, 11, 22, 23, 32, 33, 44, 45, 54, 55, 66, 67, 68, 69, 76, 77, 78, 79, 86,
                87, 88, 89, 96, 97, 98, 99,
            ]),
            BitSet128::from_slice(&[
                02, 03, 12, 13, 20, 21, 30, 31, 46, 47, 48, 49, 56, 57, 58, 59, 64, 65, 74, 75, 84,
                85, 94, 95,
            ]),
            BitSet128::from_slice(&[
                04, 05, 14, 15, 26, 27, 28, 29, 36, 37, 38, 39, 40, 41, 50, 51, 62, 63, 72, 73, 82,
                83, 92, 93,
            ]),
            BitSet128::from_slice(&[
                06, 07, 08, 09, 16, 17, 18, 19, 24, 25, 34, 35, 42, 43, 52, 53, 60, 61, 70, 71, 80,
                81, 90, 91,
            ]),
        ],
        [
            BitSet128::from_slice(&[
                00, 01, 10, 11, 22, 23, 32, 33, 44, 45, 54, 55, 66, 67, 68, 69, 76, 77, 78, 79, 86,
                87, 88, 89, 96, 97, 98, 99,
            ]),
            BitSet128::from_slice(&[
                02, 03, 12, 13, 20, 21, 30, 31, 46, 47, 48, 49, 56, 57, 58, 59, 64, 65, 74, 75, 84,
                85, 94, 95,
            ]),
            BitSet128::from_slice(&[
                04, 05, 14, 15, 26, 27, 28, 29, 36, 37, 38, 39, 40, 41, 50, 51, 62, 63, 72, 73, 82,
                83, 92, 93,
            ]),
            BitSet128::from_slice(&[
                06, 07, 08, 09, 16, 17, 18, 19, 24, 25, 34, 35, 42, 43, 52, 53, 60, 61, 70, 71, 80,
                81, 90, 91,
            ]),
        ],
    ];

    fn get_class(i: usize) -> BitSet128 {
        *Self::RELATIONS[2]
            .iter()
            .find(|bitset| bitset.contains(i))
            .unwrap()
    }

    pub fn new() -> Self {
        PairConstraints {
            sq_pair: (PartialLatinSquare::new(), PartialLatinSquare::new()),
            values_left: BitSet128::all_less_than(N * N),
            empty_cells: BitSet128::all_less_than(N * N),
            rows: [(BitSet128::all_less_than(N * N)); N],
            cols: [(BitSet128::all_less_than(N * N)); N],
            first_values: [(BitSet128::all_less_than(N * N)); N],
            second_values: [(BitSet128::all_less_than(N * N)); N],
        }
    }

    pub fn new_first_row() -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            let value = constraints
                .values_for_cell(0, i)
                .into_iter()
                .next()
                .unwrap();
            let value_pair = ValuePair::from_index::<N>(value);
            constraints.set(0, i, value_pair);
        }

        constraints
    }

    pub fn sq_pair(&self) -> PartialLatinSquarePair<N> {
        self.sq_pair
    }

    pub fn set(&mut self, i: usize, j: usize, value: ValuePair) {
        let value_index = value.0 + value.1 * N;

        self.rows[i] = self.rows[i]
            .intersect(Self::VALUE_PAIRS_WITHOUT_FIRST[value.0])
            .intersect(Self::VALUE_PAIRS_WITHOUT_SECOND[value.1]);
        self.cols[j] = self.cols[j]
            .intersect(Self::VALUE_PAIRS_WITHOUT_FIRST[value.0])
            .intersect(Self::VALUE_PAIRS_WITHOUT_SECOND[value.1]);

        self.first_values[value.0] = self.first_values[value.0]
            .intersect(Self::CELLS_WITHOUT_ROW[i])
            .intersect(Self::CELLS_WITHOUT_COLUMN[j]);
        self.second_values[value.1] = self.second_values[value.1]
            .intersect(Self::CELLS_WITHOUT_ROW[i])
            .intersect(Self::CELLS_WITHOUT_COLUMN[j]);

        self.empty_cells.remove(i * N + j);
        self.values_left.remove(value_index);

        self.sq_pair.0.set(i, j, value.0);
        self.sq_pair.1.set(i, j, value.1);
    }

    pub fn set_first_value(&mut self, cell: Cell, value: usize) {
        assert!(
            self.first_values[value].contains(cell.to_index::<N>())
                || self.sq_pair().0.get(cell).is_some_and(|v| v == value),
            "{:?}, {:?}, {:?}",
            self.sq_pair(),
            cell,
            value
        );

        // self.first_values[value] = self.first_values[value]
        //     .intersect(Self::CELLS_WITHOUT_ROW[cell.0])
        //     .intersect(Self::CELLS_WITHOUT_COLUMN[cell.1]);

        self.sq_pair.0.set(cell.0, cell.1, value);
    }

    pub fn set_second_value(&mut self, cell: Cell, value: usize) {
        assert!(
            self.second_values[value].contains(cell.to_index::<N>())
                || self.sq_pair().1.get(cell).is_some_and(|v| v == value)
        );

        // self.second_values[value] = self.second_values[value]
        //     .intersect(Self::CELLS_WITHOUT_ROW[cell.0])
        //     .intersect(Self::CELLS_WITHOUT_COLUMN[cell.1]);

        self.sq_pair.1.set(cell.0, cell.1, value);
    }

    pub fn values_for_cell(&self, i: usize, j: usize) -> BitSet128 {
        if let Some((i, j)) = self
            .sq_pair
            .0
            .get(Cell(i, j))
            .map(|s| self.sq_pair.1.get(Cell(i, j)).map(|t| (s, t)))
            .flatten()
        {
            return BitSet128::single(ValuePair(i, j).to_index::<N>());
        }

        if let Some(first) = self.sq_pair.0.get(Cell(i, j)) {
            return self.rows[i]
                .intersect(self.cols[j])
                .intersect(self.values_left)
                .intersect(Self::VALUE_PAIRS_WITHOUT_FIRST[first].complement());
        }
        if let Some(second) = self.sq_pair.1.get(Cell(i, j)) {
            return self.rows[i]
                .intersect(self.cols[j])
                .intersect(self.values_left)
                .intersect(Self::VALUE_PAIRS_WITHOUT_SECOND[second].complement());
        }

        self.rows[i]
            .intersect(self.cols[j])
            .intersect(self.values_left)
        // .intersect(Self::get_class(i * N + j))
    }

    pub fn first_values_for_cell(&self, cell: Cell) -> BitSet128 {
        BitSet128::from_iter(
            self.values_for_cell(cell.0, cell.1)
                .into_iter()
                .map(|index| index % N),
        )
    }

    pub fn second_values_for_cell(&self, cell: Cell) -> BitSet128 {
        BitSet128::from_iter(
            self.values_for_cell(cell.0, cell.1)
                .into_iter()
                .map(|index| index / N),
        )
    }

    pub fn cells_for_value(&self, value: ValuePair) -> BitSet128 {
        let mut cells = BitSet128::empty();

        for cell_index in 0..N * N {
            let cell = Cell::from_index::<N>(cell_index);

            if self
                .values_for_cell(cell.0, cell.1)
                .contains(value.to_index::<N>())
            {
                cells.insert(cell_index);
            }
        }

        cells
        // self.first_values[value.0]
        //     .intersect(self.second_values[value.1])
        //     .intersect(self.empty_cells)
        // .intersect(Self::get_class(value.0 + value.1 * N))
    }

    pub fn most_constrained_cell(&self) -> Option<(Cell, usize)> {
        self.empty_cells
            .into_iter()
            .map(|index| {
                let cell = Cell::from_index::<N>(index);
                (cell, self.values_for_cell(cell.0, cell.1).len())
            })
            .min_by_key(|(_, len)| *len)
    }

    pub fn most_constrained_value(&self) -> Option<(ValuePair, usize)> {
        self.values_left
            .into_iter()
            .map(|index| {
                let value_pair = ValuePair::from_index::<N>(index);
                (value_pair, self.cells_for_value(value_pair).len())
            })
            .min_by_key(|(_, len)| *len)
    }

    pub fn most_constrained(&self) -> Option<CellOrValuePair> {
        for i in 1..N {
            let cell = Cell(i, 0);

            if self.empty_cells.contains(cell.to_index::<N>()) {
                return Some(CellOrValuePair::Cell(cell));
            }
        }

        match (self.most_constrained_cell(), self.most_constrained_value()) {
            (None, None) => None,
            (Some((cell, cell_values)), Some((value, value_cells))) => {
                Some(if cell_values < value_cells {
                    CellOrValuePair::Cell(cell)
                } else {
                    CellOrValuePair::ValuePair(value)
                })
            }
            _ => unreachable!(),
        }
    }

    pub fn to_latin_squares(self) -> LatinSquarePair<N> {
        self.into()
    }

    pub fn is_solved(&self) -> bool {
        self.values_left.is_empty() && self.empty_cells.is_empty()
    }

    pub fn is_solvable(&self) -> bool {
        self.is_solvable_rec(1)
    }

    fn is_solvable_rec(&self, max_depth: usize) -> bool {
        for cell_index in self.empty_cells {
            let cell = Cell::from_index::<N>(cell_index);

            let values = &self.values_for_cell(cell.0, cell.1);
            if values.is_empty() {
                return false;
            }
        }

        for value_index in self.values_left {
            let value_pair = ValuePair::from_index::<N>(value_index);

            let cells = self.cells_for_value(value_pair);
            if cells.is_empty() {
                return false;
            }
        }

        if max_depth > 0 {
            let mut values: Vec<_> = self
                .empty_cells
                .into_iter()
                .map(|cell_index| {
                    let cell = Cell::from_index::<N>(cell_index);
                    let values = self.values_for_cell(cell.0, cell.1);
                    (cell, values)
                })
                .filter(|(_, values)| values.len() > 1 && values.len() <= N)
                .collect();

            values.sort_by_key(|(_, values)| values.len());

            for (cell, values) in values {
                if values.into_iter().all(|value| {
                    let value_pair = ValuePair::from_index::<N>(value);
                    let mut copy = self.clone();
                    copy.set(cell.0, cell.1, value_pair);
                    copy.find_and_set_singles();

                    !copy.is_solvable_rec(max_depth - 1)
                }) {
                    return false;
                }
            }

            let mut values: Vec<_> = self
                .values_left
                .into_iter()
                .map(|value_index| {
                    let value_pair = ValuePair::from_index::<N>(value_index);

                    let cells = self.cells_for_value(value_pair);

                    (value_pair, cells)
                })
                .filter(|(_, cells)| cells.len() > 1 && cells.len() <= N)
                .collect();

            values.sort_by_key(|(_, cells)| cells.len());

            for (value_pair, cells) in values {
                if cells.into_iter().all(|cell| {
                    let cell = Cell::from_index::<N>(cell);
                    let mut copy = self.clone();
                    copy.set(cell.0, cell.1, value_pair);
                    copy.find_and_set_singles();
                    !copy.is_solvable_rec(max_depth - 1)
                }) {
                    return false;
                }
            }
        }

        true
    }

    pub fn find_and_set_singles(&mut self) {
        let mut changed = true;

        while changed {
            changed = false;
            for cell in self.empty_cells {
                let cell = Cell::from_index::<N>(cell);
                let Cell(i, j) = cell;

                let values = self.values_for_cell(i, j);
                if values.is_single() {
                    let value = values.into_iter().next().unwrap();
                    let value_pair = ValuePair::from_index::<N>(value);
                    self.set(i, j, value_pair);
                    changed = true;
                }

                let first_values = self.first_values_for_cell(cell);
                if first_values.is_single() && self.sq_pair.0.get(cell).is_none() {
                    let value = first_values.into_iter().next().unwrap();
                    self.set_first_value(cell, value);
                    changed = true;
                }

                let second_values = self.second_values_for_cell(cell);
                if second_values.is_single() && self.sq_pair.1.get(cell).is_none() {
                    let value = second_values.into_iter().next().unwrap();
                    self.set_second_value(cell, value);
                    changed = true;
                }
            }

            for value in self.values_left {
                let value_pair = ValuePair::from_index::<N>(value);

                let cells = self.cells_for_value(value_pair);
                if cells.is_single() {
                    let cell = cells.into_iter().next().unwrap();
                    let cell = Cell::from_index::<N>(cell);
                    self.set(cell.0, cell.1, value_pair);
                    changed = true;
                }
            }
        }
    }

    pub fn find_singles(&self) -> Vec<CellOrValuePair> {
        let mut singles = vec![];

        for cell in self.empty_cells {
            let cell = Cell::from_index::<N>(cell);
            let Cell(i, j) = cell;

            let values = self.values_for_cell(i, j);
            if values.is_single() {
                singles.push(CellOrValuePair::Cell(cell));
            }
        }

        for value in self.values_left {
            let value_pair = ValuePair::from_index::<N>(value);

            let cells = self.cells_for_value(value_pair);
            if cells.is_single() {
                singles.push(CellOrValuePair::ValuePair(value_pair));
            }
        }

        singles
    }
}

// impl<const N: usize> Debug for PairConstraints<N> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[\n")?;
//         for i in 0..N {
//             write!(f, "    [")?;
//             for j in 0..N {
//                 if self.values_for_cell(i, j).is_single() {
//                     write!(
//                         f,
//                         "{:02}, ",
//                         self.values_for_cell(i, j).into_iter().next().unwrap()
//                     )?;
//                 } else {
//                     write!(f, "??, ")?;
//                 }
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

impl ValuePair {
    pub fn from_index<const N: usize>(index: usize) -> Self {
        ValuePair(index % N, index / N)
    }

    pub fn to_index<const N: usize>(self) -> usize {
        self.0 + self.1 * N
    }
}
