use std::{
    ffi::c_ushort,
    fmt::{Debug, Display},
    mem::MaybeUninit,
};

use crate::{
    bitset::{BitSet128, BitSet16},
    latin_square::{self, Cell, LatinSquare},
    latin_square_oa_generator::LatinSquareOAGenerator,
    partial_latin_square::PartialLatinSquare,
    permutation::Permutation,
    tuple_iterator::TupleIterator,
};

type BigBitSet = BitSet128;
type SmallBitSet = BitSet16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValuePair(pub usize, pub usize);

impl ValuePair {
    pub fn from_index<const N: usize>(index: usize) -> Self {
        ValuePair(index % N, index / N)
    }

    pub fn to_index<const N: usize>(self) -> usize {
        self.0 + self.1 * N
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PartialOrthogonalArray<const N: usize, const MOLS: usize> {
    columns: [[[Option<u8>; N]; N]; MOLS],
}

impl<const N: usize, const MOLS: usize> PartialOrthogonalArray<N, MOLS> {
    pub fn empty() -> Self {
        PartialOrthogonalArray {
            columns: [[[None; N]; N]; MOLS],
        }
    }

    pub fn squares(&self) -> [PartialLatinSquare<N>; MOLS] {
        self.columns.map(|col| {
            let mut new_col = [[None; N]; N];

            for i in 0..N {
                for j in 0..N {
                    new_col[i][j] = col[i][j];
                }
            }

            PartialLatinSquare::from_array(new_col)
        })
    }

    pub fn get(&self, column: usize, i: usize, j: usize) -> Option<u8> {
        self.columns[column][i][j]
    }

    pub fn num_entries(&self) -> usize {
        self.columns
            .iter()
            .map(|col| {
                col.iter()
                    .map(|row| row.iter().flatten().count())
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn next_empty_index(&self, start: usize) -> Option<usize> {
        self.columns
            .iter()
            .flat_map(|col| col.iter().flat_map(|row| row.iter()))
            .skip(start)
            .position(|i| i.is_none())
            .map(|i| i + start)
    }

    pub fn first_empty_index(&self) -> Option<usize> {
        self.next_empty_index(0)
    }

    pub fn set(&mut self, column: usize, i: usize, j: usize, value: Option<u8>) {
        self.columns[column][i][j] = value
    }
}

impl<const N: usize, const MOLS: usize> From<OrthogonalArray<N, MOLS>>
    for PartialOrthogonalArray<N, MOLS>
{
    fn from(value: OrthogonalArray<N, MOLS>) -> Self {
        let columns = value
            .columns
            .map(|col| col.map(|row| row.map(|col| Some(col))));
        PartialOrthogonalArray { columns }
    }
}

impl<const N: usize, const MOLS: usize> Display for PartialOrthogonalArray<N, MOLS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.squares().map(|sq| sq.to_string()).join("-"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrthogonalArray<const N: usize, const MOLS: usize> {
    columns: [[[u8; N]; N]; MOLS],
}

impl<const N: usize, const MOLS: usize> OrthogonalArray<N, MOLS> {
    pub fn new(sqs: [LatinSquare<N>; MOLS]) -> Self {
        OrthogonalArray {
            columns: sqs.map(|sq| sq.into()),
        }
    }

    pub fn squares(&self) -> [LatinSquare<N>; MOLS] {
        self.columns.map(|col| {
            let mut new_col = [[0; N]; N];

            for i in 0..N {
                for j in 0..N {
                    new_col[i][j] = col[i][j];
                }
            }

            LatinSquare::new(new_col)
        })
    }

    pub fn unavoidable_sets(&self) -> Vec<Vec<BitSet128>> {
        vec![self.unavoidable_sets_order_1()]
    }

    pub fn unavoidable_sets_order_1(&self) -> Vec<BitSet128> {
        let mut sets = Vec::new();
        let max_size = N * 4 * MOLS;

        let triple_iter = TupleIterator::<4>::new(N);

        for triple in triple_iter {
            for partial in [self.without_rows(&triple), self.without_cols(&triple)]
                .into_iter()
                .chain((0..MOLS).map(|i| self.without_vals(i, &triple)))
            {
                let solutions = LatinSquareOAGenerator::<N, MOLS>::from_partial_oa(&partial);

                for solution in solutions {
                    let difference = self.difference_mask(&solution);

                    if !difference.is_empty()
                        && difference.len() <= max_size
                        && !sets.contains(&difference)
                    {
                        sets.push(difference);
                        // if sets.len() > 10000 {
                        //     max_size -= 1;
                        //     sets.retain(|s| s.len() <= max_size);
                        // }
                    }
                }
            }
        }

        sets
    }

    pub fn mask(&self, mask: BitSet128) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial_oa = PartialOrthogonalArray::empty();

        for i in mask {
            let col = i / (N * N);
            let Cell(i, j) = Cell::from_index::<N>(i % (N * N));

            partial_oa.set(col, i, j, Some(self.get(col, i, j)));
        }

        partial_oa
    }

    pub fn get(&self, column: usize, i: usize, j: usize) -> u8 {
        self.columns[column][i][j]
    }

    fn without_rows(&self, rows: &[usize]) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial = PartialOrthogonalArray::from(self.clone());

        for column in 0..MOLS {
            for i in rows {
                for j in 0..N {
                    partial.set(column, *i, j, None);
                }
            }
        }

        partial
    }

    fn without_cols(&self, cols: &[usize]) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial = PartialOrthogonalArray::from(self.clone());

        for column in 0..MOLS {
            for i in 0..N {
                for j in cols {
                    partial.set(column, i, *j, None);
                }
            }
        }

        partial
    }

    fn without_vals(&self, index: usize, values: &[usize]) -> PartialOrthogonalArray<N, MOLS> {
        let mut partial = PartialOrthogonalArray::from(self.clone());

        for i in 0..N {
            for j in 0..N {
                if values.contains(&(self.get(index, i, j) as usize)) {
                    for column in 0..MOLS {
                        partial.set(column, i, j, None);
                    }
                }
            }
        }

        partial
    }

    fn difference_mask(&self, other: &OrthogonalArray<N, MOLS>) -> BitSet128 {
        let mut mask = BitSet128::empty();

        for col in 0..MOLS {
            for i in 0..N {
                for j in 0..N {
                    if self.get(col, i, j) != other.get(col, i, j) {
                        let index = col * N * N + Cell(i, j).to_index::<N>();
                        assert!(index < 128);
                        mask.insert(index);
                    }
                }
            }
        }

        mask
    }

    pub fn permute_rows(&self, permutation: &Permutation<N>) -> Self {
        let mut new = self.clone();

        for i in 0..MOLS {
            new.columns[i] = permutation.apply_array(new.columns[i]);
        }

        new
    }
}

impl<const N: usize, const MOLS: usize> Display for OrthogonalArray<N, MOLS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.squares().map(|sq| sq.to_string()).join("-"))
    }
}

const SEPARATOR: char = '-';

#[derive(Debug)]
pub enum Error {
    InvalidLength {
        len: usize,
        expected: usize,
    },
    InvalidSeparators {
        count: usize,
        expected: usize,
    },
    InvalidLatinSquare {
        index: usize,
        error: latin_square::Error,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength { len, expected } => {
                write!(f, "Invalid len: {len}, expected {expected}")
            }
            Error::InvalidSeparators { count, expected } => write!(
                f,
                "Invalid number of separators ({SEPARATOR}): {count}, expected {expected}"
            ),
            Error::InvalidLatinSquare { index, error } => {
                write!(f, "Error in latin square {}: {error}", index + 1)
            }
        }
    }
}

impl<const N: usize, const MOLS: usize> TryFrom<&str> for OrthogonalArray<N, MOLS> {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != N * N * MOLS + MOLS - 1 {
            return Err(Error::InvalidLength {
                len: value.len(),
                expected: N * N * MOLS + MOLS - 1,
            });
        }

        let separator_count = value.chars().filter(|c| *c == SEPARATOR).count();
        if separator_count != MOLS - 1 {
            return Err(Error::InvalidSeparators {
                count: separator_count,
                expected: MOLS - 1,
            });
        }

        let sqs: Vec<Result<LatinSquare<N>, Error>> = value
            .split(SEPARATOR)
            .enumerate()
            .map(|(i, split)| {
                LatinSquare::try_from(split)
                    .map_err(|error| Error::InvalidLatinSquare { index: i, error })
            })
            .collect();

        let mut sqs_array = [MaybeUninit::uninit(); MOLS];
        for (i, sq) in sqs.into_iter().enumerate() {
            sqs_array[i].write(sq?);
        }

        let sqs = sqs_array.map(|sq| unsafe { sq.assume_init() });

        let oa = OrthogonalArray::new(sqs);
        Ok(oa)
    }
}

#[derive(Clone, Debug)]
pub struct OAConstraints<const N: usize, const MOLS: usize> {
    oa: PartialOrthogonalArray<N, MOLS>,
    column_pair_values: [[BigBitSet; MOLS]; MOLS],
    cell_values: [[[SmallBitSet; N]; N]; MOLS],
    empty_cells: [BigBitSet; MOLS],
    rows: [[SmallBitSet; N]; MOLS],
    columns: [[SmallBitSet; N]; MOLS],
}

impl<const N: usize, const MOLS: usize> OAConstraints<N, MOLS> {
    pub fn new() -> Self {
        OAConstraints {
            oa: PartialOrthogonalArray::empty(),
            column_pair_values: [[BigBitSet::all_less_than(N * N); MOLS]; MOLS],
            cell_values: [[[SmallBitSet::all_less_than(N); N]; N]; MOLS],
            empty_cells: [BigBitSet::all_less_than(N * N); MOLS],
            rows: [[SmallBitSet::all_less_than(N); N]; MOLS],
            columns: [[SmallBitSet::all_less_than(N); N]; MOLS],
        }
    }

    pub fn new_reduced() -> Self {
        let mut constraints = Self::new();

        for col in 1..(MOLS - 1) {
            let next_col = col + 1;

            let min_val = constraints
                .values_for_cell(col, N)
                .into_iter()
                .next()
                // .or(constraints.oa.columns[col][1][0].map(|val| val as usize))
                .unwrap();

            let mask = SmallBitSet::all_less_than(min_val + 1).complement();

            let values = &mut constraints.cell_values[next_col][1][0];
            *values = values.intersect(mask);
        }

        for i in 0..MOLS {
            for j in 0..N {
                constraints.set_and_propagate(i, j, j);
            }
        }

        for j in 1..N {
            constraints.set_and_propagate(0, j * N, j);
        }

        // constraints.cell_values[0][1][1] = SmallBitSet::from_iter([0, 2]);
        // constraints.cell_values[0][1][2] = SmallBitSet::from_iter([0, 3]);
        // constraints.cell_values[0][1][3] = SmallBitSet::from_iter([2, 4]);

        // constraints.cell_values[0][1][1] = SmallBitSet::from_iter([3, 4]);

        constraints.find_and_set_singles();

        constraints
    }

    pub fn from_partial_sq(sq: PartialLatinSquare<N>) -> Self {
        let mut constraints = Self::new();

        for i in 0..N {
            for j in 0..N {
                let Some(value) = sq.get(i, j) else {
                    continue;
                };

                let index = Cell(i, j).to_index::<N>();
                constraints.set_and_propagate(0, index, value);
            }
        }

        constraints
    }

    pub fn from_partial_oa(oa: &PartialOrthogonalArray<N, MOLS>) -> Self {
        let mut constraints = Self::new();

        for column in 0..MOLS {
            for i in 0..N {
                for j in 0..N {
                    let Some(value) = oa.get(column, i, j) else {
                        continue;
                    };

                    let index = Cell(i, j).to_index::<N>();
                    constraints.set_and_propagate(column, index, value as usize);
                }
            }
        }

        constraints
    }

    pub fn from_partial_sq_reduced(sq: PartialLatinSquare<N>) -> Self {
        let mut constraints = Self::new_reduced();

        assert!(sq.is_reduced());

        for i in 0..N {
            for j in 0..N {
                let Some(value) = sq.get(i, j) else {
                    continue;
                };

                let index = Cell(i, j).to_index::<N>();
                if constraints.oa.squares()[0].get(i, j).is_none() {
                    constraints.set_and_propagate(0, index, value);
                }
            }
        }

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
            index += MOLS - i - 2;
        }

        &mut self.column_pair_values[min][max]
    }

    fn get_column_pair_values(&self, column1: usize, column2: usize) -> &BigBitSet {
        let min = column1.min(column2);
        let max = column1.max(column2);

        assert!(min != max);

        let mut index = max - 1;
        for i in 0..min {
            index += MOLS - i - 2;
        }

        &self.column_pair_values[min][max]
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

        let Cell(i, j) = Cell::from_index::<N>(index);

        (self.oa.columns[min][i][j], self.oa.columns[max][i][j])
    }

    pub fn set_and_propagate(&mut self, column: usize, index: usize, value: usize) {
        self.set(column, index, value);

        self.propagate_constraints();
    }

    fn set(&mut self, column: usize, index: usize, value: usize) {
        assert!(
            self.values_for_cell(column, index).contains(value),
            "{:?}, {column}, {index}, {value}, {:?}",
            self.oa,
            self.values_for_cell(column, index)
        );
        assert!(
            self.empty_cells[column].contains(index),
            "{:?}, {column}, {index}, {value}, {:?}",
            self.oa,
            self.empty_cells[column]
        );

        let Cell(row, col) = Cell::from_index::<N>(index);

        self.oa.columns[column][row][col] = Some(value as u8);
        self.empty_cells[column].remove(index);
        self.cell_values[column][row][col] = SmallBitSet::empty();
        self.rows[column][row].remove(value);
        self.columns[column][col].remove(value);

        for i in 0..MOLS {
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

            for first_column in 0..MOLS {
                for second_column in (first_column + 1)..MOLS {
                    let mut second_vals_for_first_val = [SmallBitSet::empty(); N];
                    let mut first_vals_for_second_val = [SmallBitSet::empty(); N];

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
                        let Cell(i, j) = Cell::from_index::<N>(index);
                        let first_value = self.oa.columns[first_column][i][j].unwrap() as usize;

                        let second_values = &mut self.cell_values[second_column][i][j];
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
                        let Cell(i, j) = Cell::from_index::<N>(index);
                        let second_value = self.oa.columns[second_column][i][j].unwrap() as usize;

                        let first_values = &mut self.cell_values[first_column][i][j];
                        if !first_values.is_subset_of(first_vals_for_second_val[second_value]) {
                            *first_values =
                                first_values.intersect(first_vals_for_second_val[second_value]);
                            changed = true;
                        }
                    }
                }
            }

            // mols are in increasing order
            for col in 1..(MOLS - 1) {
                let next_col = col + 1;

                let min_val = self
                    .values_for_cell(col, N)
                    .into_iter()
                    .next()
                    .or(self.oa.columns[col][1][0].map(|val| val as usize))
                    .unwrap();

                let mask = SmallBitSet::all_less_than(min_val + 1).complement();

                let values = &mut self.cell_values[next_col][1][0];
                *values = values.intersect(mask);
            }
        }
    }

    pub fn values_for_cell(&self, column: usize, index: usize) -> SmallBitSet {
        let Cell(row, col) = Cell::from_index::<N>(index);

        let row_values = self.rows[column][row];
        let column_values = self.columns[column][col];

        self.cell_values[column][row][col]
            .intersect(row_values)
            .intersect(column_values)
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

        let mut first_values = vec![false; N * N];
        for i in 0..N {
            for j in 0..N {
                first_values[i * N + j] = if let Some(val) = self.oa.columns[min][i][j] {
                    val as usize == value_pair.0
                } else {
                    self.values_for_cell(min, i * N + j).contains(value_pair.0)
                };
            }
        }

        let mut second_values = vec![false; N * N];
        for i in 0..N {
            for j in 0..N {
                second_values[i * N + j] = if let Some(val) = self.oa.columns[max][i][j] {
                    val as usize == value_pair.1
                } else {
                    self.values_for_cell(max, i * N + j).contains(value_pair.1)
                };
            }
        }

        let cells: BigBitSet = first_values
            .into_iter()
            .zip(second_values)
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

        let Cell(i, j) = Cell::from_index::<N>(index);

        let values1 = if let Some(v1) = self.oa.columns[min][i][j] {
            SmallBitSet::single(v1 as usize)
        } else {
            self.values_for_cell(min, index)
        };
        let values2 = if let Some(v2) = self.oa.columns[max][i][j] {
            SmallBitSet::single(v2 as usize)
        } else {
            self.values_for_cell(max, index)
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

    pub fn most_constrained_cell(&self) -> Option<(usize, usize)> {
        let mut min = N + 1;
        let mut min_dist = N * N;
        let mut min_cell = (0, 0);

        for column in 0..MOLS {
            for index in self.empty_cells[column] {
                let cell = (column, index);

                let row = index / N;
                let col = index % N;
                let dist = (row * N + col).min(row + col * N);

                let len = self.values_for_cell(column, index).len();
                if len.cmp(&min).then(dist.cmp(&min_dist)).is_le() {
                    min = len;
                    min_dist = dist;
                    min_cell = cell;
                }
            }
        }

        (min != N + 1).then_some(min_cell)
    }

    pub fn is_solvable(&self) -> bool {
        self.is_solvable_rec(0)
    }

    fn is_solvable_rec(&self, max_depth: usize) -> bool {
        for column in 0..MOLS {
            for index in self.empty_cells[column] {
                if self.values_for_cell(column, index).is_empty() {
                    return false;
                }
            }

            for col2 in (column + 1)..MOLS {
                let pair = self.get_column_pair_values(column, col2);

                for value_pair in *pair {
                    let value_pair = ValuePair::from_index::<N>(value_pair);
                    let cells_for_pair = self.cells_for_pair(column, col2, value_pair);
                    if cells_for_pair.is_empty() {
                        return false;
                    }
                }

                for index in self.empty_cells[column].union(self.empty_cells[col2]) {
                    let pairs_for_cell = self.pairs_for_cell(column, col2, index);
                    if pairs_for_cell.is_empty() {
                        return false;
                    }
                }
            }
        }

        if max_depth > 0 {
            let mut cells = Vec::new();
            for column in 0..MOLS {
                for index in self.empty_cells[column] {
                    let len = self.values_for_cell(column, index).len();

                    cells.push((column, index, len));
                }
            }

            cells.retain(|(_, _, len)| *len <= 3);
            cells.sort_by_key(|(_, _, len)| *len);

            for (column, index, _) in cells {
                if self
                    .values_for_cell(column, index)
                    .into_iter()
                    .all(|value| {
                        let mut copy = self.clone();
                        copy.set_and_propagate(column, index, value);
                        copy.find_and_set_singles();

                        !copy.is_solvable_rec(max_depth - 1)
                    })
                {
                    return false;
                }
            }

            for col1 in 0..MOLS {
                for col2 in (col1 + 1)..MOLS {
                    let pairs = *self.get_column_pair_values(col1, col2);

                    for pair in pairs {
                        let value_pair = ValuePair::from_index::<N>(pair);
                        let cells_for_pair = self.cells_for_pair(col1, col2, value_pair);
                        if cells_for_pair.len() <= 4
                            && cells_for_pair.into_iter().all(|cell| {
                                let mut copy = self.clone();
                                if !copy.values_for_cell(col1, cell).contains(value_pair.0) {
                                    return false;
                                }
                                copy.set_and_propagate(col1, cell, value_pair.0);
                                if !copy.values_for_cell(col2, cell).contains(value_pair.1) {
                                    return false;
                                }
                                copy.set_and_propagate(col2, cell, value_pair.1);
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
            for column in 0..MOLS {
                for index in self.empty_cells[column] {
                    if self.values_for_cell(column, index).is_single() {
                        let value = self
                            .values_for_cell(column, index)
                            .into_iter()
                            .next()
                            .unwrap();
                        self.set_and_propagate(column, index, value);
                        changed = true;
                    }
                }
            }

            for column1 in 0..MOLS {
                for column2 in (column1 + 1)..MOLS {
                    let pair = self.get_column_pair_values(column1, column2);

                    for value_pair in *pair {
                        let value_pair = ValuePair::from_index::<N>(value_pair);

                        let cells_for_pair = self.cells_for_pair(column1, column2, value_pair);
                        if cells_for_pair.is_single() {
                            let cell = cells_for_pair.into_iter().next().unwrap();

                            if self.empty_cells[column1].contains(cell) {
                                self.set_and_propagate(column1, cell, value_pair.0);
                                changed = true;
                            }
                            if self.empty_cells[column2].contains(cell) {
                                self.set_and_propagate(column2, cell, value_pair.1);
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
                                self.set_and_propagate(column1, index, value_pair.0);
                                changed = true;
                            }
                            if self.empty_cells[column2].contains(index) {
                                self.set_and_propagate(column2, index, value_pair.1);
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

    pub fn possible_values_log(&self) -> f64 {
        self.empty_cells
            .iter()
            .enumerate()
            .map(|(i, col)| {
                col.into_iter()
                    .map(|index| (self.values_for_cell(i, index).len() as f64).log2())
                    .sum::<f64>()
            })
            .sum()
    }

    pub fn is_solved(&self) -> bool {
        self.empty_cells
            .map(|bitset| bitset.len())
            .into_iter()
            .sum::<usize>()
            == 0
    }

    pub fn find_and_set_const(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            for column in 0..MOLS {
                for cell_index in self.empty_cells[column] {
                    let values = self.values_for_cell(column, cell_index);

                    if values.is_empty() {
                        return;
                    }

                    let mut squares = Vec::new();

                    for value in values {
                        let mut copy = self.clone();

                        copy.set_and_propagate(column, cell_index, value);
                        // copy.find_and_set_singles();

                        squares.push(copy.squares());
                    }

                    for column in 0..MOLS {
                        for cell_index in self.empty_cells[column] {
                            let Cell(i, j) = Cell::from_index::<N>(cell_index);

                            if let Some(target_value) = squares[0][column].get(i, j) {
                                if squares
                                    .iter()
                                    .skip(1)
                                    .all(|sq| sq[column].get(i, j) == Some(target_value))
                                {
                                    self.set_and_propagate(column, cell_index, target_value);
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<const N: usize, const MOLS: usize> Debug for PartialOrthogonalArray<N, MOLS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for i in 0..N {
            for j in 0..N {
                write!(f, "    [")?;
                for k in 0..MOLS {
                    if let Some(value) = self.columns[k][i][j] {
                        write!(f, "{:2}, ", value)?;
                    } else {
                        write!(f, "??, ")?;
                    }
                }
                write!(f, "]")?;
            }

            if i != N - 1 {
                writeln!(f, ",")?;
            }
        }
        write!(f, "\n]")?;
        Ok(())
    }
}
