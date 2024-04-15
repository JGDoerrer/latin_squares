use crate::{
    bitset::BitSet,
    latin_square::{Cell, PartialLatinSquare},
    pair_constraints::{CellOrValuePair, PairConstraints, ValuePair},
};

#[derive(Debug, Clone)]
pub struct TripleConstraints<const N: usize> {
    squares: [PartialLatinSquare<N>; 3],
    empty_cells: BitSet,
    rows: [[BitSet; N]; 3],
    cols: [[BitSet; N]; 3],
    vals: [[BitSet; N]; 3],
    pair01: PairConstraints<N>,
    pair02: PairConstraints<N>,
    pair12: PairConstraints<N>,
}

pub type ValueTriple = [usize; 3];

#[derive(Debug, Clone, Copy)]
pub enum CellOrValueTriple {
    Cell(Cell),
    ValueTriple(ValueTriple),
}

impl<const N: usize> TripleConstraints<N> {
    const CELLS_WITHOUT_COLUMN: [BitSet; N] = {
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
    const CELLS_WITHOUT_ROW: [BitSet; N] = {
        let mut table = [BitSet::empty(); N];

        let mut i = 0;
        while i < N {
            table[i] = BitSet::from_bits(BitSet::all_less_than(N).bits() << (i * N)).complement();
            i += 1;
        }

        table
    };

    pub fn new() -> Self {
        TripleConstraints {
            squares: [PartialLatinSquare::new(); 3],
            empty_cells: BitSet::all_less_than(N * N),
            rows: [[BitSet::all_less_than(N); N]; 3],
            cols: [[BitSet::all_less_than(N); N]; 3],
            vals: [[BitSet::all_less_than(N * N); N]; 3],
            pair01: PairConstraints::new(),
            pair02: PairConstraints::new(),
            pair12: PairConstraints::new(),
        }
    }

    pub fn filled_cells(&self) -> usize {
        self.empty_cells
            .complement()
            .intersect(BitSet::all_less_than(N * N))
            .len()
    }

    pub fn squares(&self) -> [PartialLatinSquare<N>; 3] {
        self.squares
    }

    pub fn set(&mut self, cell: Cell, values: ValueTriple) {
        assert!(self
            .pair01
            .values_for_cell(cell.0, cell.1)
            .contains(ValuePair(values[0], values[1]).to_index::<N>()));
        assert!(self
            .pair02
            .values_for_cell(cell.0, cell.1)
            .contains(ValuePair(values[0], values[2]).to_index::<N>()));
        assert!(self
            .pair12
            .values_for_cell(cell.0, cell.1)
            .contains(ValuePair(values[1], values[2]).to_index::<N>()));

        self.pair01
            .set(cell.0, cell.1, ValuePair(values[0], values[1]));
        self.pair02
            .set(cell.0, cell.1, ValuePair(values[0], values[2]));
        self.pair12
            .set(cell.0, cell.1, ValuePair(values[1], values[2]));
        self.empty_cells.remove(cell.to_index::<N>());

        for i in 0..3 {
            self.squares[i].set(cell.0, cell.1, values[i]);
        }
    }

    pub fn set_first_value(&mut self, cell: Cell, value: usize) {
        assert!(self.pair01.first_values_for_cell(cell).contains(value));
        assert!(self.pair02.first_values_for_cell(cell).contains(value));

        self.pair01.set_first_value(cell, value);
        self.pair02.set_first_value(cell, value);

        self.squares[0].set(cell.0, cell.1, value);
    }

    pub fn set_second_value(&mut self, cell: Cell, value: usize) {
        assert!(self.pair01.second_values_for_cell(cell).contains(value));
        assert!(self.pair12.first_values_for_cell(cell).contains(value));

        self.pair01.set_second_value(cell, value);
        self.pair12.set_first_value(cell, value);

        self.squares[1].set(cell.0, cell.1, value);
    }

    pub fn set_third_value(&mut self, cell: Cell, value: usize) {
        assert!(self.pair02.second_values_for_cell(cell).contains(value));
        assert!(self.pair12.second_values_for_cell(cell).contains(value));

        self.pair02.set_second_value(cell, value);
        self.pair12.set_second_value(cell, value);

        self.squares[2].set(cell.0, cell.1, value);
    }

    pub fn values_for_cell(&self, cell: Cell) -> Vec<ValueTriple> {
        let values01 = self.pair01.values_for_cell(cell.0, cell.1);
        let values02 = self.pair02.values_for_cell(cell.0, cell.1);
        let values12 = self.pair12.values_for_cell(cell.0, cell.1);

        let mut values = vec![];
        for index in values01 {
            let ValuePair(i, j) = ValuePair::from_index::<N>(index);
            for k in 0..N {
                if values02.contains(ValuePair(i, k).to_index::<N>())
                    && values12.contains(ValuePair(j, k).to_index::<N>())
                {
                    let triple = [i, j, k];
                    values.push(triple)
                }
            }
        }

        values

        // let vals0 = self
        //     .pair01
        //     .first_values_for_cell(cell)
        //     .intersect(self.pair02.first_values_for_cell(cell));

        // let vals1 = self
        //     .pair01
        //     .second_values_for_cell(cell)
        //     .intersect(self.pair12.first_values_for_cell(cell));

        // let vals2 = self
        //     .pair02
        //     .second_values_for_cell(cell)
        //     .intersect(self.pair12.second_values_for_cell(cell));

        // let mut values = vec![];

        // for i in vals0 {
        //     for j in vals1 {
        //         for k in vals2 {
        //             if values01.contains(i + j * N)
        //                 && values02.contains(i + k * N)
        //                 && values12.contains(j + k * N)
        //             {
        //                 values.push([i, j, k]);
        //             }
        //         }
        //     }
        // }

        // let mut values02map = [BitSet::empty(); N];
        // for index in values02 {
        //     let (i, j) = (index % N, index / N);
        //     values02map[i].insert(j);
        // }

        // let mut values12map = [BitSet::empty(); N];
        // for index in values12 {
        //     let (i, j) = (index % N, index / N);
        //     values12map[i].insert(j);
        // }

        // for index in values01 {
        //     let (i, j) = (index % N, index / N);

        //     let values2 = values02map[i].intersect(values12map[j]);

        //     for k in values2 {
        //         values.push([i, j, k]);
        //     }
        // }
        // values
    }

    pub fn values_for_cell_len(&self, cell: Cell) -> usize {
        let values01 = self.pair01.values_for_cell(cell.0, cell.1);
        let values02 = self.pair02.values_for_cell(cell.0, cell.1);
        let values12 = self.pair12.values_for_cell(cell.0, cell.1);

        let mut values = 0;
        for index in values01 {
            let ValuePair(i, j) = ValuePair::from_index::<N>(index);
            for k in 0..N {
                if values02.contains(ValuePair(i, k).to_index::<N>())
                    && values12.contains(ValuePair(j, k).to_index::<N>())
                {
                    values += 1;
                }
            }
        }

        values

        // let vals0 = self
        //     .pair01
        //     .first_values_for_cell(cell)
        //     .intersect(self.pair02.first_values_for_cell(cell));

        // let vals1 = self
        //     .pair01
        //     .second_values_for_cell(cell)
        //     .intersect(self.pair12.first_values_for_cell(cell));

        // let vals2 = self
        //     .pair02
        //     .second_values_for_cell(cell)
        //     .intersect(self.pair12.second_values_for_cell(cell));

        // let mut values = 0;

        // for i in vals0 {
        //     for j in vals1 {
        //         for k in vals2 {
        //             if values01.contains(i + j * N)
        //                 && values02.contains(i + k * N)
        //                 && values12.contains(j + k * N)
        //             {
        //                 values += 1;
        //             }
        //         }
        //     }
        // }

        // values

        // let mut values02map = [BitSet::empty(); N];
        // for index in values02 {
        //     let (i, j) = (index % N, index / N);
        //     values02map[i].insert(j);
        // }

        // let mut values12map = [BitSet::empty(); N];
        // for index in values12 {
        //     let (i, j) = (index % N, index / N);
        //     values12map[i].insert(j);
        // }

        // let mut values = 0;
        // for index in values01 {
        //     let (i, j) = (index % N, index / N);

        //     let values2 = values02map[i].intersect(values12map[j]);

        //     values += values2.len();
        // }
        // values
    }

    pub fn first_values_for_cell(&self, cell: Cell) -> BitSet {
        self.pair01
            .first_values_for_cell(cell)
            .intersect(self.pair02.first_values_for_cell(cell))
    }

    pub fn second_values_for_cell(&self, cell: Cell) -> BitSet {
        self.pair01
            .second_values_for_cell(cell)
            .intersect(self.pair12.first_values_for_cell(cell))
    }

    pub fn third_values_for_cell(&self, cell: Cell) -> BitSet {
        self.pair02
            .second_values_for_cell(cell)
            .intersect(self.pair12.second_values_for_cell(cell))
    }

    pub fn cells_for_value(&self, values: ValueTriple) -> Vec<Cell> {
        let cell01 = self.pair01.cells_for_value(ValuePair(values[0], values[1]));
        let cell02 = self.pair02.cells_for_value(ValuePair(values[0], values[2]));
        let cell12 = self.pair12.cells_for_value(ValuePair(values[1], values[2]));

        let mut cells = vec![];
        for cell in self.vals[0][values[0]]
            .intersect(self.vals[1][values[1]])
            .intersect(self.vals[2][values[2]])
            .intersect(self.empty_cells)
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

    pub fn cells_for_value_len(&self, values: ValueTriple) -> usize {
        let cell01 = self.pair01.cells_for_value(ValuePair(values[0], values[1]));
        let cell02 = self.pair02.cells_for_value(ValuePair(values[0], values[2]));
        let cell12 = self.pair12.cells_for_value(ValuePair(values[1], values[2]));

        let mut cells = 0;
        for cell in self.vals[0][values[0]]
            .intersect(self.vals[1][values[1]])
            .intersect(self.vals[2][values[2]])
            .intersect(self.empty_cells)
            .intersect(cell01)
            .intersect(cell02)
            .intersect(cell12)
        {
            let cell = Cell::from_index::<N>(cell);

            if self.values_for_cell(cell).contains(&values) {
                cells += 1;
            }
        }

        cells
    }

    pub fn most_constrained(&self) -> Option<CellOrValueTriple> {
        for j in 0..2 {
            for i in 0..N {
                let cell = Cell(j, i);

                if self.empty_cells.contains(cell.to_index::<N>()) {
                    return Some(CellOrValueTriple::Cell(cell));
                }
            }
            for i in 0..N {
                let cell = Cell(i, j);

                if self.empty_cells.contains(cell.to_index::<N>()) {
                    return Some(CellOrValueTriple::Cell(cell));
                }
            }
        }

        match (self.most_constrained_cell(), self.most_constrained_value()) {
            (None, None) => None,
            (Some((cell, _)), None) => Some(CellOrValueTriple::Cell(cell)),
            // (None, Some((value, _))) => Some(CellOrValueTriple::ValueTriple(value)),
            (Some((cell, cell_values)), Some((value, value_cells))) => {
                Some(if cell_values < value_cells {
                    CellOrValueTriple::Cell(cell)
                } else {
                    CellOrValueTriple::ValueTriple(value)
                })
            }
            _ => unreachable!(),
        }
        // self.most_constrained_cell()
        //     .map(|(cell, _)| CellOrValueTriple::Cell(cell))

        // self.empty_cells
        //     .into_iter()
        //     .next()
        //     .map(|cell| CellOrValueTriple::Cell(Cell::from_index::<N>(cell)))
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
        // let mut min_value = [0, 0, 0];

        // for i in 0..N {
        //     for j in 0..N {
        //         for k in 0..N {
        //             let value = [i, j, k];
        //             let cells = self.cells_for_value_len(value);

        //             if cells == 1 {
        //                 min = 1;
        //                 min_value = value;
        //             }
        //         }
        //     }
        // }
        // (min == 1).then(|| (min_value, min))

        // let value_pair01 = self
        //     .pair01
        //     .most_constrained_value()
        //     .map(|(pair01, cells)| ([pair01.0, pair01.1, N], cells));
        // let value_pair02 = self
        //     .pair02
        //     .most_constrained_value()
        //     .map(|(pair02, cells)| ([pair02.0, N, pair02.1], cells));
        // let value_pair12 = self
        //     .pair12
        //     .most_constrained_value()
        //     .map(|(pair12, cells)| ([N, pair12.0, pair12.1], cells));

        // let min_value_pair = value_pair01
        //     .into_iter()
        //     .chain(value_pair02.into_iter())
        //     .chain(value_pair12.into_iter())
        //     .min_by_key(|(_, cells)| *cells);

        // if min_value_pair.is_some_and(|(_, cells)| cells == 1) {
        //     min_value_pair
        // } else {
        //     None
        // }

        let value_pair01 = self.pair01.most_constrained_value();
        let value_pair02 = self.pair02.most_constrained_value();
        let value_pair12 = self.pair12.most_constrained_value();

        if let Some((value_pair, 1)) = value_pair01 {
            let cell = Cell::from_index::<N>(
                self.pair01
                    .cells_for_value(value_pair)
                    .into_iter()
                    .next()
                    .unwrap(),
            );

            let values = self.values_for_cell(cell);

            if values.len() == 1 {
                return Some((values[0], 1));
            }
        }
        if let Some((value_pair, 1)) = value_pair02 {
            let cell = Cell::from_index::<N>(
                self.pair02
                    .cells_for_value(value_pair)
                    .into_iter()
                    .next()
                    .unwrap(),
            );

            let values = self.values_for_cell(cell);

            if values.len() == 1 {
                return Some((values[0], 1));
            }
        }
        if let Some((value_pair, 1)) = value_pair12 {
            let cell = Cell::from_index::<N>(
                self.pair12
                    .cells_for_value(value_pair)
                    .into_iter()
                    .next()
                    .unwrap(),
            );

            let values = self.values_for_cell(cell);

            if values.len() == 1 {
                return Some((values[0], 1));
            }
        }

        None
    }

    pub fn is_solvable(&self) -> bool {
        self.is_solvable_rec(0)
    }

    fn is_solvable_rec(&self, max_depth: usize) -> bool {
        for i in self.empty_cells {
            let cell = Cell::from_index::<N>(i);
            if self.first_values_for_cell(cell).is_empty()
                || self.second_values_for_cell(cell).is_empty()
                || self.third_values_for_cell(cell).is_empty()
            {
                return false;
            }

            let values = self.values_for_cell(cell);

            if max_depth > 0
                && values.len() > 1
                && values.len() < N
                && values.into_iter().all(|value| {
                    let mut copy = self.clone();
                    copy.set(cell, value);
                    copy.find_and_set_singles();

                    !copy.is_solvable_rec(max_depth - 1)
                })
            {
                return false;
            }
        }

        if !self.pair01.is_solvable() || !self.pair02.is_solvable() || !self.pair12.is_solvable() {
            return false;
        }

        true
    }

    pub fn is_solved(&self) -> bool {
        self.empty_cells.is_empty()
    }

    pub fn find_and_set_singles(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            let singles01 = self.pair01.find_singles();

            for single in singles01 {
                match single {
                    CellOrValuePair::Cell(cell) => {
                        let Some(index) = self
                            .pair01
                            .values_for_cell(cell.0, cell.1)
                            .into_iter()
                            .next()
                        else {
                            continue;
                        };
                        let value_pair = ValuePair::from_index::<N>(index);

                        self.pair01.set(cell.0, cell.1, value_pair);
                        self.pair02.set_first_value(cell, value_pair.0);
                        self.pair12.set_first_value(cell, value_pair.1);

                        self.squares[0].set(cell.0, cell.1, value_pair.0);
                        self.squares[1].set(cell.0, cell.1, value_pair.1);
                        changed = true;
                    }
                    CellOrValuePair::ValuePair(value_pair) => {
                        let Some(index) =
                            self.pair01.cells_for_value(value_pair).into_iter().next()
                        else {
                            continue;
                        };
                        let cell = Cell::from_index::<N>(index);

                        self.pair01.set(cell.0, cell.1, value_pair);
                        self.pair02.set_first_value(cell, value_pair.0);
                        self.pair12.set_first_value(cell, value_pair.1);

                        self.squares[0].set(cell.0, cell.1, value_pair.0);
                        self.squares[1].set(cell.0, cell.1, value_pair.1);
                        changed = true;
                    }
                }
            }

            let singles02 = self.pair02.find_singles();

            for single in singles02 {
                match single {
                    CellOrValuePair::Cell(cell) => {
                        let Some(index) = self
                            .pair02
                            .values_for_cell(cell.0, cell.1)
                            .into_iter()
                            .next()
                        else {
                            continue;
                        };
                        let value_pair = ValuePair::from_index::<N>(index);

                        self.pair02.set(cell.0, cell.1, value_pair);
                        self.pair01.set_first_value(cell, value_pair.0);
                        self.pair12.set_second_value(cell, value_pair.1);

                        self.squares[0].set(cell.0, cell.1, value_pair.0);
                        self.squares[2].set(cell.0, cell.1, value_pair.1);
                        changed = true;
                    }
                    CellOrValuePair::ValuePair(value_pair) => {
                        let Some(index) =
                            self.pair02.cells_for_value(value_pair).into_iter().next()
                        else {
                            continue;
                        };
                        let cell = Cell::from_index::<N>(index);

                        self.pair02.set(cell.0, cell.1, value_pair);
                        self.pair01.set_first_value(cell, value_pair.0);
                        self.pair12.set_second_value(cell, value_pair.1);

                        self.squares[0].set(cell.0, cell.1, value_pair.0);
                        self.squares[2].set(cell.0, cell.1, value_pair.1);
                        changed = true;
                    }
                }
            }

            let singles12 = self.pair12.find_singles();

            for single in singles12 {
                match single {
                    CellOrValuePair::Cell(cell) => {
                        let Some(index) = self
                            .pair12
                            .values_for_cell(cell.0, cell.1)
                            .into_iter()
                            .next()
                        else {
                            continue;
                        };
                        let value_pair = ValuePair::from_index::<N>(index);

                        self.pair12.set(cell.0, cell.1, value_pair);
                        self.pair01.set_second_value(cell, value_pair.0);
                        self.pair02.set_second_value(cell, value_pair.1);

                        self.squares[1].set(cell.0, cell.1, value_pair.0);
                        self.squares[2].set(cell.0, cell.1, value_pair.1);
                        changed = true;
                    }
                    CellOrValuePair::ValuePair(value_pair) => {
                        let Some(index) =
                            self.pair12.cells_for_value(value_pair).into_iter().next()
                        else {
                            continue;
                        };
                        let cell = Cell::from_index::<N>(index);

                        self.pair12.set(cell.0, cell.1, value_pair);
                        self.pair01.set_second_value(cell, value_pair.0);
                        self.pair02.set_second_value(cell, value_pair.1);

                        self.squares[1].set(cell.0, cell.1, value_pair.0);
                        self.squares[2].set(cell.0, cell.1, value_pair.1);
                        changed = true;
                    }
                }
            }

            for cell in self.empty_cells {
                let cell = Cell::from_index::<N>(cell);
                if self.values_for_cell_len(cell) == 1 {
                    let value = self.values_for_cell(cell).into_iter().next().unwrap();

                    self.set(cell, value);
                    changed = true;
                    continue;
                }

                let first_values = self.first_values_for_cell(cell);
                if first_values.is_single() && self.squares[0].get(cell).is_none() {
                    let value = first_values.into_iter().next().unwrap();
                    self.set_first_value(cell, value);
                    changed = true;
                }

                let second_values = self.second_values_for_cell(cell);
                if second_values.is_single() && self.squares[1].get(cell).is_none() {
                    let value = second_values.into_iter().next().unwrap();
                    self.set_second_value(cell, value);
                    changed = true;
                }

                let third_values = self.third_values_for_cell(cell);
                if third_values.is_single() && self.squares[2].get(cell).is_none() {
                    let value = third_values.into_iter().next().unwrap();
                    self.set_third_value(cell, value);
                    changed = true;
                }
            }
        }
    }
}

pub fn to_index<const N: usize>(values: ValueTriple) -> usize {
    values[0] + values[1] * N + values[2] * N * N
}

pub fn from_index<const N: usize>(value: usize) -> ValueTriple {
    [value % N, (value / N) % N, value / (N * N)]
}
