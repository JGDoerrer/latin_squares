use std::{cmp::Ordering, fmt::Debug};

use crate::{
    bitset::{BitSet128, BitSet16},
    latin_square_oa_generator::LatinSquareOAGenerator,
    partial_latin_square::PartialLatinSquare,
    permutation::{Permutation, PermutationIter},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct LatinSquare<const N: usize> {
    values: [[u8; N]; N],
}

#[derive(Debug, Clone, Copy)]
pub struct Cell(pub usize, pub usize);

impl<const N: usize> LatinSquare<N> {
    pub fn new(values: [[u8; N]; N]) -> Self {
        let sq = LatinSquare { values };
        debug_assert!(sq.is_valid());
        sq
    }

    pub fn from_rcv(rows: [[usize; N]; N], cols: [[usize; N]; N], vals: [[usize; N]; N]) -> Self {
        let mut new_values = [[0; N]; N];

        for i in 0..N {
            for j in 0..N {
                let row = rows[i][j];
                let col = cols[i][j];
                let val = vals[i][j] as u8;

                new_values[row][col] = val;
            }
        }

        Self::new(new_values)
    }

    pub fn get(&self, i: usize, j: usize) -> usize {
        self.values[i][j] as usize
    }

    pub fn get_row(&self, i: usize) -> &[u8; N] {
        &self.values[i]
    }

    pub fn get_col(&self, i: usize) -> [u8; N] {
        let mut col = [0; N];

        for j in 0..N {
            col[j] = self.values[j][i];
        }

        col
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
        assert!(self.is_reduced());
        assert!(other.is_reduced());

        for sq in self.all_reduced() {
            for permutation in PermutationIter::new() {
                let row_reduced = sq.permute_vals(permutation).permute_cols(permutation);
                let reduced = row_reduced.permute_rows(Permutation::from_array(
                    row_reduced.get_col(0).map(|i| i as usize),
                ));

                if reduced == *other {
                    return true;
                }
            }
        }

        false
    }

    pub fn has_diagonal_symmetry(&self) -> bool {
        for i in 0..N {
            for j in (i + 1)..N {
                if self.get(i, j) != self.get(j, i) {
                    return false;
                }
            }
        }
        true
    }

    pub fn reduced(&self) -> Self {
        let first_row = self.get_row(0).map(|i| i as usize);
        let row_reduced = self.permute_cols(Permutation::from_array(first_row));

        let first_col = row_reduced.get_col(0).map(|i| i as usize);
        let reduced = row_reduced.permute_rows(Permutation::from_array(first_col));

        debug_assert!(reduced.is_reduced(), "{reduced:?}");

        reduced
    }

    pub fn all_reduced(&self) -> impl Iterator<Item = Self> + '_ {
        (0..N).map(|i| {
            let mut new_values = self.values;
            new_values.swap(0, i);

            let new = Self::new(new_values).reduced();
            new
        })
    }

    pub fn paratopic(&self) -> impl Iterator<Item = Self> + '_ {
        let mut rows = [[0; N]; N];
        for i in 0..N {
            rows[i] = [i; N];
        }

        let mut col = [0; N];

        for i in 0..N {
            col[i] = i;
        }

        let cols = [col; N];
        let vals = self.values.map(|row| row.map(|val| val as usize));

        PermutationIter::new().map(move |perm| {
            let [rows, cols, vals] = perm.apply_array([rows, cols, vals]);
            Self::from_rcv(rows, cols, vals)
        })
    }

    pub fn reduced_isotopic(&self) -> Self {
        debug_assert!(self.is_reduced());

        let mut isotopic = *self;

        for sq in self.all_reduced() {
            for permutation in PermutationIter::new() {
                let maps_to_zero = permutation.to_array().iter().position(|i| *i == 0).unwrap();
                let maps_to_one = permutation.to_array().iter().position(|i| *i == 1).unwrap();
                let maps_to_two = permutation.to_array().iter().position(|i| *i == 2).unwrap();

                let new_second_col = sq
                    .get_row(maps_to_zero)
                    .into_iter()
                    .position(|i| *i as usize == maps_to_one)
                    .unwrap();

                let new_1_1 = sq.get(maps_to_one, new_second_col);

                if new_1_1 != maps_to_zero && new_1_1 != maps_to_two {
                    continue;
                }

                // let new_second_row = Permutation::from_array(new_first_row).apply_array(
                //     sq.get_row(maps_to_one)
                //         .map(|i| permutation.apply(i as usize) as u8),
                // );
                // if new_second_row > isotopic.get_row(1) {
                //     continue;
                // }

                let col_reduced = sq.permute_vals(permutation).permute_rows(permutation);
                let reduced = col_reduced.permute_cols(Permutation::from_array(
                    sq.get_row(maps_to_zero)
                        .map(|i| permutation.apply(i as usize)),
                ));

                debug_assert!(reduced.get(1, 1) == 0 || reduced.get(1, 1) == 2);
                debug_assert!(reduced.is_reduced());
                // debug_assert!(new_second_row == reduced.get_row(1));

                if reduced.values < isotopic.values {
                    isotopic = reduced;
                }
            }
        }

        isotopic
    }

    pub fn reduced_paratopic(&self) -> Self {
        debug_assert!(self.is_reduced());

        let mut paratopic = *self;

        for sq in self.paratopic() {
            for sq in sq.all_reduced() {
                for permutation in PermutationIter::new() {
                    let col_reduced = sq.permute_vals(permutation).permute_rows(permutation);
                    let reduced = col_reduced.permute_cols(Permutation::from_array(
                        col_reduced.get_row(0).map(|i| i as usize),
                    ));

                    debug_assert!(reduced.is_reduced());
                    // debug_assert!(reduced.get(1, 1) == 0 || reduced.get(1, 1) == 2);
                    // debug_assert!(new_second_row == reduced.get_row(1));

                    if reduced < paratopic {
                        paratopic = reduced;
                    }
                }
            }
        }

        paratopic
    }

    pub fn unavoidable_sets(&self) -> Vec<Vec<BitSet128>> {
        let mut order1 = self.unavoidable_sets_order_1();

        order1 = order1
            .iter()
            .filter(|set| {
                // set.len() <= N * 2
                //     &&
                order1
                    .iter()
                    .all(|other| other == *set || !other.is_subset_of(**set))
            })
            .copied()
            .collect();

        order1.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        order1.dedup();

        let mut order2 = vec![];
        let mut max_size = 2 * N;
        for (i, set1) in order1.iter().enumerate() {
            for set2 in order1.iter().skip(i + 1) {
                let new_set = set1.union(*set2);
                if set1.is_disjoint(*set2)
                    && new_set.len() <= max_size
                    && order2.iter().all(|set| !new_set.is_subset_of(*set))
                {
                    order2.push(new_set);
                    if order2.len() > 4000 {
                        max_size -= 1;
                        dbg!(max_size);
                        order2.retain(|s| s.len() <= max_size);
                    }
                }
            }
        }

        order2.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        order2.dedup();

        let mut all_orders = vec![order1.clone(), order2];

        while all_orders.last().is_some_and(|sets| !sets.is_empty()) {
            let last_order = all_orders.last().unwrap();
            let mut next_order = vec![];
            let mut max_size = all_orders.len() * N;

            for set1 in &order1 {
                for set2 in last_order {
                    let new_set = set1.union(*set2);
                    if set1.is_disjoint(*set2) && new_set.len() <= max_size
                    // && last_order.iter().all(|set| !new_set.is_subset_of(*set))
                    {
                        next_order.push(new_set);
                        if next_order.len() > 1000 {
                            max_size -= 1;
                            next_order.retain(|s| s.len() <= max_size);
                        }
                    }
                }
            }

            next_order.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
            next_order.dedup();

            all_orders.push(next_order);
        }

        // let all_orders = vec![order1, order2];
        all_orders
    }

    pub fn unavoidable_sets_order_1(&self) -> Vec<BitSet128> {
        debug_assert!(self.is_reduced());

        let mut sets = Vec::new();
        let mut max_size = 3 * N;

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

                    if !difference.is_empty() && difference.len() <= max_size {
                        sets.push(difference);
                        if sets.len() > 5000 {
                            max_size -= 1;
                            sets.retain(|s| s.len() <= max_size);
                        }
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

    pub fn permute_rows(&self, permutation: Permutation<N>) -> Self {
        let new_values = permutation.apply_array(self.values);

        Self::new(new_values)
    }

    pub fn permute_cols(&self, permutation: Permutation<N>) -> Self {
        let mut new_values = self.values;

        new_values.iter_mut().for_each(|row| {
            *row = permutation.apply_array(*row);
        });

        Self::new(new_values)
    }

    pub fn permute_vals(&self, permutation: Permutation<N>) -> Self {
        let mut new_values = self.values;

        for row in &mut new_values {
            for val in row {
                *val = permutation.apply(*val as usize) as u8;
            }
        }

        Self::new(new_values)
    }
}

impl<const N: usize> Ord for LatinSquare<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..N {
            for j in (0..=i).rev() {
                match self.values[j][i].cmp(&other.values[j][i]) {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Equal => {}
                }
                match self.values[i][j].cmp(&other.values[i][j]) {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Equal => {}
                }
            }
        }

        Ordering::Equal
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

impl<const N: usize> ToString for LatinSquare<N> {
    fn to_string(&self) -> String {
        let mut string = String::with_capacity(N * N);
        for i in 0..N {
            for j in 0..N {
                string.push(char::from_digit(self.get(i, j) as u32, 10).unwrap());
            }
        }
        string
    }
}

impl<const N: usize> TryFrom<&str> for LatinSquare<N> {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != N * N {
            return Err("Invalid length");
        }

        let mut values = [[0; N]; N];
        for (i, c) in value.chars().enumerate() {
            let entry = c.to_digit(10).ok_or("Invalid digit")?;
            if entry >= N as u32 {
                return Err("Invalid digit");
            }
            values[i / N][i % N] = entry as u8;
        }

        Ok(LatinSquare::new(values))
    }
}

impl<const N: usize> From<PartialLatinSquare<N>> for LatinSquare<N> {
    fn from(value: PartialLatinSquare<N>) -> Self {
        let mut sq = LatinSquare {
            values: [[0; N]; N],
        };

        for i in 0..N {
            for j in 0..N {
                sq.values[i][j] = value.get(i, j).unwrap() as u8;
            }
        }

        sq
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
