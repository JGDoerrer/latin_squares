use std::collections::btree_map::Values;

use crate::{
    bitset::BitSet,
    latin_square::{Cell, LatinSquare},
    pair_constraints::{PairConstraints, ValuePair},
};

#[derive(Debug, Clone)]
pub struct TripleConstraints<const N: usize> {
    empty_cells: BitSet,
    pair01: PairConstraints<N>,
    pair02: PairConstraints<N>,
    pair12: PairConstraints<N>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueTriple(pub usize, pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub enum CellOrValueTriple {
    Cell(Cell),
    ValueTriple(ValueTriple),
}

impl<const N: usize> TripleConstraints<N> {
    pub fn new() -> Self {
        TripleConstraints {
            empty_cells: BitSet::all_less_than(N * N),
            pair01: PairConstraints::new(),
            pair02: PairConstraints::new(),
            pair12: PairConstraints::new(),
        }
    }

    pub fn set(&mut self, cell: Cell, values: ValueTriple) {
        assert!(self
            .pair01
            .values_for_cell(cell.0, cell.1)
            .contains(ValuePair(values.0, values.1).to_index::<N>()));
        assert!(self
            .pair02
            .values_for_cell(cell.0, cell.1)
            .contains(ValuePair(values.0, values.2).to_index::<N>()));
        assert!(self
            .pair12
            .values_for_cell(cell.0, cell.1)
            .contains(ValuePair(values.1, values.2).to_index::<N>()));

        self.pair01.set(cell.0, cell.1, (values.0, values.1));
        self.pair02.set(cell.0, cell.1, (values.0, values.2));
        self.pair12.set(cell.0, cell.1, (values.1, values.2));
        self.empty_cells.remove(cell.to_index::<N>())
    }

    pub fn values_for_cell(&self, cell: Cell) -> Vec<ValueTriple> {
        let values01 = self
            .pair01
            .values_for_cell(cell.0, cell.1)
            .into_iter()
            .map(|index| ValuePair::from_index::<N>(index));
        let values02 = self.pair02.values_for_cell(cell.0, cell.1);
        let values12 = self.pair12.values_for_cell(cell.0, cell.1);

        let mut values = vec![];
        for ValuePair(i, j) in values01 {
            for k in 0..N {
                if values02.contains(ValuePair(i, k).to_index::<N>())
                    && values12.contains(ValuePair(j, k).to_index::<N>())
                {
                    let triple = ValueTriple(i, j, k);
                    values.push(triple)
                }
            }
        }

        values
    }

    pub fn values_for_cell_len(&self, cell: Cell) -> usize {
        let values01 = self
            .pair01
            .values_for_cell(cell.0, cell.1)
            .into_iter()
            .map(|index| ValuePair::from_index::<N>(index));
        let values02 = self.pair02.values_for_cell(cell.0, cell.1);
        let values12 = self.pair12.values_for_cell(cell.0, cell.1);

        let mut values = 0;
        for ValuePair(i, j) in values01 {
            for k in 0..N {
                if values02.contains(ValuePair(i, k).to_index::<N>())
                    && values12.contains(ValuePair(j, k).to_index::<N>())
                {
                    values += 1;
                }
            }
        }

        values
    }

    pub fn cells_for_value(&self, values: ValueTriple) -> Vec<Cell> {
        let cell01 = self.pair01.cells_for_value((values.0, values.1));
        let cell02 = self.pair02.cells_for_value((values.0, values.2));
        let cell12 = self.pair12.cells_for_value((values.1, values.2));

        let mut cells = vec![];
        for cell in self
            .empty_cells
            .intersect(cell01)
            .intersect(cell02)
            .intersect(cell12)
        {
            let cell = Cell::from_index::<N>(cell);

            if self.values_for_cell(cell).contains(&values) {
                cells.push(cell);
            }
        }
        cells.sort_by_key(|cell| cell.to_index::<N>());

        cells
    }

    pub fn most_constrained(&self) -> Option<CellOrValueTriple> {
        match (self.most_constrained_cell(), self.most_constrained_value()) {
            (None, None) => None,
            (Some((cell, _)), None) => Some(CellOrValueTriple::Cell(cell)),
            (None, Some((value, _))) => Some(CellOrValueTriple::ValueTriple(value)),
            (Some((cell, cell_values)), Some((value, value_cells))) => {
                Some(if cell_values < value_cells {
                    CellOrValueTriple::Cell(cell)
                } else {
                    CellOrValueTriple::ValueTriple(value)
                })
            }
        }
    }

    pub fn most_constrained_cell(&self) -> Option<(Cell, usize)> {
        let mut min = N * N * N + 1;
        let mut min_cell = Cell(0, 0);

        for cell in self.empty_cells {
            let cell = Cell::from_index::<N>(cell);
            let values = self.values_for_cell_len(cell);

            if values < min {
                min = values;
                min_cell = cell;
            }
        }

        (min != N * N * N + 1).then(|| (min_cell, min))
    }

    pub fn most_constrained_value(&self) -> Option<(ValueTriple, usize)> {
        // let mut min = N * N * N + 1;
        // let mut min_value = ValueTriple(0, 0, 0);

        // for i in 0..N {
        //     for j in 0..N {
        //         for k in 0..N {
        //             let value = ValueTriple(i, j, k);
        //             let cells = self.cells_for_value(value);

        //             if cells.len() > 0 && cells.len() < min {
        //                 min = cells.len();
        //                 min_value = value;
        //             }
        //         }
        //     }
        // }
        // (min != N * N * N + 1).then(|| (min_value, min))

        let value_pair01 = self
            .pair01
            .most_constrained_value()
            .map(|(pair01, cells)| (ValueTriple(pair01.0, pair01.1, N), cells));
        let value_pair02 = self
            .pair02
            .most_constrained_value()
            .map(|(pair02, cells)| (ValueTriple(pair02.0, N, pair02.1), cells));
        let value_pair12 = self
            .pair12
            .most_constrained_value()
            .map(|(pair12, cells)| (ValueTriple(N, pair12.0, pair12.1), cells));

        let min_value_pair = value_pair01
            .into_iter()
            .chain(value_pair02.into_iter())
            .chain(value_pair12.into_iter())
            .min_by_key(|(_, cells)| *cells);

        min_value_pair
    }

    pub fn is_solvable(&self) -> bool {
        if !self.pair01.is_solvable() || !self.pair02.is_solvable() || !self.pair12.is_solvable() {
            return false;
        }
        // for i in self.empty_cells {
        //     let cell = Cell::from_index::<N>(i);
        //     if self.values_for_cell(cell).is_empty() {
        //         return false;
        //     }
        // }

        true
    }
}

impl ValueTriple {
    pub fn to_index<const N: usize>(self) -> usize {
        self.0 + self.1 * N + self.2 * N * N
    }

    pub fn from_index<const N: usize>(value: usize) -> Self {
        ValueTriple(value % N, (value / N) % N, value / (N * N))
    }
}
