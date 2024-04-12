use crate::{
    bitset::BitSet,
    latin_square::{Cell, LatinSquarePair},
};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct PairConstraints<const N: usize> {
    values_left: BitSet,
    empty_cells: BitSet,
    rows: [BitSet; N],
    cols: [BitSet; N],
    first_values: [BitSet; N],
    second_values: [BitSet; N],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValuePair(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub enum CellOrValuePair {
    Cell(Cell),
    ValuePair(ValuePair),
}

impl<const N: usize> PairConstraints<N> {
    const VALUE_PAIRS_WITHOUT_FIRST: [BitSet; N] = {
        let mut table = [BitSet::empty(); N];

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
            table[i] = BitSet::from_bits(BitSet::single(i).bits() * every_nth).complement();
            i += 1;
        }

        table
    };
    const VALUE_PAIRS_WITHOUT_SECOND: [BitSet; N] = {
        let mut table = [BitSet::empty(); N];

        let mut i = 0;
        while i < N {
            table[i] = BitSet::from_bits(BitSet::all_less_than(N).bits() << (i * N)).complement();
            i += 1;
        }

        table
    };
    const CELLS_WITHOUT_ROW: [BitSet; N] = Self::VALUE_PAIRS_WITHOUT_SECOND;
    const CELLS_WITHOUT_COLUMN: [BitSet; N] = Self::VALUE_PAIRS_WITHOUT_FIRST;

    const RELATIONS: [[BitSet; 4]; 3] = [
        [
            BitSet::from_slice(&[
                00, 11, 12, 13, 21, 22, 23, 31, 32, 33, 44, 45, 46, 54, 55, 56, 64, 65, 66, 77, 78,
                79, 87, 88, 89, 97, 98, 99,
            ]),
            BitSet::from_slice(&[
                01, 02, 03, 10, 20, 30, 47, 48, 49, 57, 58, 59, 67, 68, 69, 74, 75, 76, 84, 85, 86,
                94, 95, 96,
            ]),
            BitSet::from_slice(&[
                04, 05, 06, 17, 18, 19, 27, 28, 29, 37, 38, 39, 40, 50, 60, 71, 72, 73, 81, 82, 83,
                91, 92, 93,
            ]),
            BitSet::from_slice(&[
                07, 08, 09, 14, 15, 16, 24, 25, 26, 34, 35, 36, 41, 42, 43, 51, 52, 53, 61, 62, 63,
                70, 80, 90,
            ]),
        ],
        [
            BitSet::from_slice(&[
                00, 01, 10, 11, 22, 23, 32, 33, 44, 45, 54, 55, 66, 67, 68, 69, 76, 77, 78, 79, 86,
                87, 88, 89, 96, 97, 98, 99,
            ]),
            BitSet::from_slice(&[
                02, 03, 12, 13, 20, 21, 30, 31, 46, 47, 48, 49, 56, 57, 58, 59, 64, 65, 74, 75, 84,
                85, 94, 95,
            ]),
            BitSet::from_slice(&[
                04, 05, 14, 15, 26, 27, 28, 29, 36, 37, 38, 39, 40, 41, 50, 51, 62, 63, 72, 73, 82,
                83, 92, 93,
            ]),
            BitSet::from_slice(&[
                06, 07, 08, 09, 16, 17, 18, 19, 24, 25, 34, 35, 42, 43, 52, 53, 60, 61, 70, 71, 80,
                81, 90, 91,
            ]),
        ],
        [
            BitSet::from_slice(&[
                00, 01, 10, 11, 22, 23, 32, 33, 44, 45, 54, 55, 66, 67, 68, 69, 76, 77, 78, 79, 86,
                87, 88, 89, 96, 97, 98, 99,
            ]),
            BitSet::from_slice(&[
                02, 03, 12, 13, 20, 21, 30, 31, 46, 47, 48, 49, 56, 57, 58, 59, 64, 65, 74, 75, 84,
                85, 94, 95,
            ]),
            BitSet::from_slice(&[
                04, 05, 14, 15, 26, 27, 28, 29, 36, 37, 38, 39, 40, 41, 50, 51, 62, 63, 72, 73, 82,
                83, 92, 93,
            ]),
            BitSet::from_slice(&[
                06, 07, 08, 09, 16, 17, 18, 19, 24, 25, 34, 35, 42, 43, 52, 53, 60, 61, 70, 71, 80,
                81, 90, 91,
            ]),
        ],
    ];

    fn get_class(i: usize) -> BitSet {
        *Self::RELATIONS[2]
            .iter()
            .find(|bitset| bitset.contains(i))
            .unwrap()
    }

    pub fn new() -> Self {
        PairConstraints {
            values_left: BitSet::all_less_than(N * N),
            empty_cells: BitSet::all_less_than(N * N),
            rows: [(BitSet::all_less_than(N * N)); N],
            cols: [(BitSet::all_less_than(N * N)); N],
            first_values: [(BitSet::all_less_than(N * N)); N],
            second_values: [(BitSet::all_less_than(N * N)); N],
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
            let value_pair = ((value % N) as usize, (value / N) as usize);
            constraints.set(0, i, value_pair);
        }

        constraints
    }

    pub fn set(&mut self, i: usize, j: usize, value: (usize, usize)) {
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
    }

    pub fn values_for_cell(&self, i: usize, j: usize) -> BitSet {
        self.rows[i]
            .intersect(self.cols[j])
            .intersect(self.values_left)
        // .intersect(Self::get_class(i * N + j))
    }

    pub fn first_values_for_cell(&self, cell: Cell) -> BitSet {
        BitSet::from_iter(
            self.values_for_cell(cell.0, cell.1)
                .into_iter()
                .map(|index| index % N),
        )
    }

    pub fn second_values_for_cell(&self, cell: Cell) -> BitSet {
        BitSet::from_iter(
            self.values_for_cell(cell.0, cell.1)
                .into_iter()
                .map(|index| index / N),
        )
    }

    pub fn cells_for_value(&self, value: (usize, usize)) -> BitSet {
        self.first_values[value.0]
            .intersect(self.second_values[value.1])
            .intersect(self.empty_cells)
        // .intersect(Self::get_class(value.0 + value.1 * N))
    }

    pub fn most_constrained_cell(&self) -> Option<(Cell, usize)> {
        self.empty_cells
            .into_iter()
            .map(|index| {
                let cell = Cell(index / N, index % N);
                (cell, self.values_for_cell(cell.0, cell.1).len())
            })
            .min_by_key(|(_, len)| *len)
    }

    pub fn most_constrained_value(&self) -> Option<(ValuePair, usize)> {
        self.values_left
            .into_iter()
            .map(|index| {
                let value_pair = ValuePair(index % N, index / N);
                (
                    value_pair,
                    self.cells_for_value((value_pair.0, value_pair.1)).len(),
                )
            })
            .min_by_key(|(_, len)| *len)
    }

    pub fn most_constrained(&self) -> Option<CellOrValuePair> {
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
        for cell_index in self.empty_cells {
            if self
                .values_for_cell(cell_index / N, cell_index % N)
                .is_empty()
            {
                return false;
            }
        }

        for value_index in self.values_left {
            if self
                .cells_for_value((value_index % N, value_index / N))
                .is_empty()
            {
                return false;
            }
        }

        true
    }

    pub fn find_singles(&mut self) {
        let mut counts = vec![0; N * N];
        for i in 0..N {
            for j in 0..N {
                if !self.values_for_cell(i, j).is_single() {
                    for value in self.values_for_cell(i, j) {
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
                    if self.values_for_cell(i, j).contains(value) {
                        let value_pair = ((value % N) as usize, (value / N) as usize);
                        self.set(i, j, value_pair);
                    }
                }
            }
        }
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
