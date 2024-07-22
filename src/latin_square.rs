use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Write},
    mem::MaybeUninit,
};

use crate::{
    bitset::{BitSet128, BitSet16},
    latin_square_oa_generator::LatinSquareOAGenerator,
    latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait},
    partial_latin_square::PartialLatinSquare,
    permutation::{Permutation, PermutationDyn, PermutationIter},
    tuple_iterator::{TupleIterator, TupleIteratorDyn},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LatinSquare<const N: usize> {
    values: [[u8; N]; N],
}

#[derive(Debug, Clone, Copy)]
pub struct Cell(pub usize, pub usize);

impl<const N: usize> PartialLatinSquareTrait for LatinSquare<N> {
    fn n(&self) -> usize {
        N
    }

    fn get_partial(&self, row: usize, col: usize) -> Option<usize> {
        Some(self.values[row][col].into())
    }
}

impl<const N: usize> LatinSquareTrait for LatinSquare<N> {
    fn get(&self, row: usize, col: usize) -> usize {
        self.values[row][col].into()
    }
}

impl<const N: usize> LatinSquare<N> {
    pub fn new(values: [[u8; N]; N]) -> Self {
        debug_assert!(Self::is_valid(&values));

        LatinSquare { values }
    }

    pub fn z() -> Self {
        let mut values = [[0; N]; N];

        for (i, row) in values.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = ((i + j) % N) as u8;
            }
        }

        LatinSquare::new(values)
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

    pub fn get_row(&self, i: usize) -> &[u8; N] {
        &self.values[i]
    }

    pub fn get_col(&self, i: usize) -> [u8; N] {
        let mut col = [0; N];

        for (j, val) in col.iter_mut().enumerate() {
            *val = self.values[j][i];
        }

        col
    }

    pub fn values(self) -> [[u8; N]; N] {
        self.values
    }

    pub fn is_valid(values: &[[u8; N]; N]) -> bool {
        (0..N).all(|i| {
            (0..N).map(|j| values[i][j] as usize).collect::<BitSet16>()
                == BitSet16::all_less_than(N)
                && (0..N).map(|j| values[j][i] as usize).collect::<BitSet16>()
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
                let row_reduced = sq.permute_vals(&permutation).permute_cols(&permutation);
                let reduced = row_reduced.permute_rows(&Permutation::from_array(
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
        let row_reduced = self.permute_cols(&Permutation::from_array(first_row));

        let first_col = row_reduced.get_col(0).map(|i| i as usize);
        let reduced = row_reduced.permute_rows(&Permutation::from_array(first_col));

        debug_assert!(reduced.is_reduced(), "{reduced:?}");

        reduced
    }

    pub fn all_reduced(&self) -> impl Iterator<Item = Self> + '_ {
        (0..N).map(|i| {
            let mut new_values = self.values;
            new_values.swap(0, i);

            Self::new(new_values).reduced()
        })
    }

    pub fn paratopic(&self) -> impl Iterator<Item = Self> + '_ {
        let mut rows = [[0; N]; N];
        for (i, row) in rows.iter_mut().enumerate() {
            *row = [i; N];
        }

        let mut col = [0; N];

        for (i, val) in col.iter_mut().enumerate() {
            *val = i;
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
                let maps_to_zero = permutation.as_array().iter().position(|i| *i == 0).unwrap();
                let maps_to_one = permutation.as_array().iter().position(|i| *i == 1).unwrap();
                let maps_to_two = permutation.as_array().iter().position(|i| *i == 2).unwrap();

                let new_second_col = sq
                    .get_row(maps_to_zero)
                    .iter()
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

                let col_reduced = sq.permute_vals(&permutation).permute_rows(&permutation);
                let reduced = col_reduced.permute_cols(&Permutation::from_array(
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

    pub fn reduced_paratopic_old(&self) -> Self {
        let sq = &self.reduced();

        let mut paratopic = *sq;

        for sq in sq.paratopic() {
            for s in PermutationIter::new() {
                let sq = sq.permute_vals(&s);
                for c in PermutationIter::new() {
                    let sq = sq.permute_cols(&c);
                    let sq = sq
                        .permute_rows(&Permutation::from_array(sq.get_col(0).map(|i| i as usize)));

                    if sq.cmp_rows(&paratopic).is_lt() {
                        paratopic = sq;
                    }
                }
            }
        }

        paratopic
    }

    pub fn reduced_paratopic(&self) -> Self {
        let sq = &self.reduced();

        let mut paratopic = *sq;

        for sq in sq.paratopic() {
            for s in PermutationIter::new() {
                let sq = sq.permute_vals(&s);

                for row0 in 0..N {
                    let mut new_sq = sq;

                    new_sq.values.swap(0, row0);

                    let col_permutation =
                        Permutation::from_array(new_sq.get_row(0).map(|i| i.into()));

                    new_sq = new_sq.permute_cols(&col_permutation);
                    new_sq.values[1..].sort();

                    if new_sq.cmp_rows(&paratopic).is_lt() {
                        paratopic = new_sq;
                    }
                }
            }
        }

        debug_assert_eq!(self.reduced_paratopic_old(), paratopic);

        paratopic
    }

    pub fn reduced_nauty(&self) -> Self {
        todo!()
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

        let mut order2 = self.unavoidable_sets_order_2();

        order2 = order2
            .iter()
            .filter(|set| {
                order2
                    .iter()
                    .all(|other| other == *set || !other.is_subset_of(**set))
            })
            .copied()
            .collect();

        order2.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        order2.dedup();

        let all_orders = vec![order1, order2];
        all_orders
    }

    pub fn unavoidable_sets_order_1(&self) -> Vec<BitSet128> {
        if N < 2 {
            return vec![];
        }
        if N < 3 {
            return vec![BitSet128::all_less_than(N * N)];
        }

        let mut sets = Vec::new();
        let max_size = N * 3;

        let triple_iter = TupleIterator::<3>::new(N);

        for triple in triple_iter {
            for partial in [
                self.without_rows(triple),
                self.without_cols(triple),
                self.without_vals(triple),
            ] {
                let solutions = LatinSquareOAGenerator::<N, 1>::from_partial_sq(partial)
                    .map(|s| s.squares()[0]);

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

    pub fn unavoidable_sets_order_2(&self) -> Vec<BitSet128> {
        if N < 3 {
            return vec![];
        }

        let mut sets = Vec::new();

        // these may not be all
        for (rows, cols) in self.subsquares::<3>() {
            let mut set = BitSet128::empty();

            for row in rows {
                for col in &cols {
                    set.insert(row * N + col);
                }
            }

            sets.push(set);
        }

        for rows in TupleIterator::<3>::new(N) {
            for cols in TupleIterator::<3>::new(N) {
                let mut subsquare = self.get_subsquare(&rows, &cols);

                let mut permutation = [None; N];

                for (i, element) in permutation.iter_mut().enumerate().take(3) {
                    *element = Some(subsquare[0][i] as usize);
                }

                for i in 3..N {
                    for j in 0..N {
                        if !permutation.contains(&Some(j)) {
                            permutation[i] = Some(j);
                        }
                    }
                }

                let permutation: Permutation<N> =
                    PermutationDyn::from_array(permutation.map(|i| i.unwrap()))
                        .pad_with_id()
                        .inverse();

                for row in &mut subsquare {
                    for val in row.iter_mut() {
                        *val = permutation.apply(*val as usize) as u8;
                    }
                }

                if subsquare == [[0, 1, 2], [1, 0, 3], [2, 3, 0]] {
                    let mut set = BitSet128::empty();

                    for row in rows {
                        for col in &cols {
                            set.insert(row * N + col);
                        }
                    }

                    sets.push(set);
                }
            }
        }

        sets
    }

    fn subsquares_order_2_iter(&self) -> impl Iterator<Item = [usize; 4]> + '_ {
        let rows_iter = TupleIterator::new(N);

        rows_iter.flat_map(|[row1, row2]| {
            let cols_iter = TupleIterator::new(N);
            cols_iter
                .map(move |[col1, col2]| [row1, row2, col1, col2])
                .filter(|[row1, row2, col1, col2]| {
                    self.get(*row1, *col1) == self.get(*row2, *col2)
                        && self.get(*row1, *col2) == self.get(*row2, *col1)
                })
        })
    }

    pub fn get_subsquare<const K: usize>(
        &self,
        rows: &[usize; K],
        cols: &[usize; K],
    ) -> [[u8; K]; K] {
        assert!(K <= N);

        let mut values = [[0; K]; K];

        for i in 0..K {
            for (j, col) in cols.iter().enumerate() {
                values[i][j] = self.values[rows[i]][*col];
            }
        }

        values
    }

    pub fn get_subsquare_dyn(&self, rows: &[usize], cols: &[usize]) -> Vec<Vec<u8>> {
        debug_assert!(rows.len() == cols.len());

        let k = rows.len();

        let mut values = vec![vec![0; k]; k];

        for i in 0..k {
            for (j, col) in cols.iter().enumerate() {
                values[i][j] = self.values[rows[i]][*col];
            }
        }

        values
    }

    pub fn subsquares<const K: usize>(&self) -> Vec<([usize; K], [usize; K])> {
        if K > N {
            return vec![];
        }

        let mut subsquares = Vec::new();

        for rows in TupleIterator::<K>::new(N) {
            for cols in TupleIterator::<K>::new(N) {
                let mut subsquare = self.get_subsquare::<K>(&rows, &cols);

                let mut permutation = [None; N];

                for (i, element) in permutation.iter_mut().enumerate().take(K) {
                    *element = Some(subsquare[0][i] as usize);
                }

                for i in K..N {
                    for j in 0..N {
                        if !permutation.contains(&Some(j)) {
                            permutation[i] = Some(j);
                        }
                    }
                }

                let permutation: Permutation<N> =
                    PermutationDyn::from_array(permutation.map(|i| i.unwrap()))
                        .pad_with_id()
                        .inverse();

                for row in subsquare.iter_mut() {
                    for val in row.iter_mut() {
                        *val = permutation.apply(*val as usize) as u8;
                    }
                }

                if LatinSquare::<K>::is_valid(&subsquare) {
                    subsquares.push((rows, cols));
                }
            }
        }

        subsquares
    }

    pub fn num_subsquares_dyn(&self, k: usize) -> usize {
        let mut subsquares = 0;

        for rows in TupleIteratorDyn::new(N, k) {
            for cols in TupleIteratorDyn::new(N, k) {
                let mut subsquare = self.get_subsquare_dyn(&rows, &cols);

                let mut permutation: Vec<_> = subsquare[0].iter().map(|i| *i as usize).collect();

                for i in 0..N {
                    if !permutation.contains(&i) {
                        permutation.push(i);
                    }
                }

                let permutation: Permutation<N> = PermutationDyn::from_vec(permutation)
                    .pad_with_id()
                    .inverse();

                for row in subsquare.iter_mut() {
                    for val in row.iter_mut() {
                        *val = permutation.apply(*val as usize) as u8;
                    }
                }
                let is_subsquare = (0..k).all(|i| {
                    (0..k)
                        .map(|j| subsquare[i][j] as usize)
                        .collect::<BitSet16>()
                        == BitSet16::all_less_than(k)
                        && (0..k)
                            .map(|j| subsquare[j][i] as usize)
                            .collect::<BitSet16>()
                            == BitSet16::all_less_than(k)
                });
                if is_subsquare {
                    subsquares += 1;
                }
            }
        }

        subsquares
    }

    pub fn intercalates(&self) -> usize {
        self.subsquares_order_2_iter().count()
    }

    pub fn mask(&self, mask: BitSet128) -> PartialLatinSquare<N> {
        let mut partial_sq = PartialLatinSquare::empty();

        for i in mask {
            let Cell(i, j) = Cell::from_index::<N>(i);

            partial_sq.set(i, j, Some(self.get(i, j)));
        }

        partial_sq
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

    pub fn permute_rows(&self, permutation: &Permutation<N>) -> Self {
        let new_values = permutation.apply_array(self.values);

        Self::new(new_values)
    }

    pub fn permute_cols(&self, permutation: &Permutation<N>) -> Self {
        let mut new_values = self.values;

        new_values.iter_mut().for_each(|row| {
            *row = permutation.apply_array(*row);
        });

        Self::new(new_values)
    }

    pub fn permute_rows_and_cols(&self, permutation: &Permutation<N>) -> Self {
        let mut values = [[MaybeUninit::uninit(); N]; N];

        for (i, new_row) in values.iter_mut().enumerate() {
            let row = self.values[permutation.apply(i)];
            for j in 0..N {
                new_row[j].write(row[permutation.apply(j)]);
            }
        }

        let values = values.map(|row| row.map(|val| unsafe { val.assume_init() }));

        Self { values }
    }

    pub fn permute_vals(&self, permutation: &Permutation<N>) -> Self {
        let mut new_values = self.values;

        for row in &mut new_values {
            for val in row {
                *val = permutation.apply(*val as usize) as u8;
            }
        }

        Self::new(new_values)
    }

    pub fn cmp_diagonal(&self, other: &Self) -> Ordering {
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

    pub fn cmp_rows(&self, other: &Self) -> Ordering {
        self.values.cmp(&other.values)
    }
}

impl<const N: usize> PartialOrd for LatinSquare<N> {
    fn partial_cmp(&self, other: &LatinSquare<N>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for LatinSquare<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_rows(other)
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

impl<const N: usize> Display for LatinSquare<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..N {
            for j in 0..N {
                f.write_char(char::from_digit(self.get(i, j) as u32, 10).unwrap())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidLength { len: usize, expected: usize },
    InvalidChar { index: usize, char: char },
    InvalidLatinSquare,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength { len, expected } => {
                write!(f, "Invalid len: {len}, expected {expected}")
            }
            Error::InvalidChar { index, char } => {
                write!(f, "Invalid char at index {index}: {char}")
            }
            Error::InvalidLatinSquare => write!(f, "The latin square property is not met"),
        }
    }
}

impl<const N: usize> TryFrom<&str> for LatinSquare<N> {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != N * N {
            return Err(Error::InvalidLength {
                len: value.len(),
                expected: N * N,
            });
        }

        let mut values = [[0; N]; N];
        for (i, c) in value.chars().enumerate() {
            let entry = c
                .to_digit(10)
                .ok_or(Error::InvalidChar { index: i, char: c })?;
            if entry >= N as u32 {
                return Err(Error::InvalidChar { index: i, char: c });
            }
            values[i / N][i % N] = entry as u8;
        }

        values.try_into()
    }
}

impl<const N: usize> TryFrom<[[u8; N]; N]> for LatinSquare<N> {
    type Error = Error;
    fn try_from(value: [[u8; N]; N]) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(LatinSquare::new(value))
        } else {
            Err(Error::InvalidLatinSquare)
        }
    }
}

impl<const N: usize> TryFrom<PartialLatinSquare<N>> for LatinSquare<N> {
    type Error = ();

    fn try_from(value: PartialLatinSquare<N>) -> Result<Self, ()> {
        let mut sq = LatinSquare {
            values: [[0; N]; N],
        };

        for i in 0..N {
            for j in 0..N {
                sq.values[i][j] = value.get_partial(i, j).unwrap() as u8;
            }
        }

        Ok(sq)
    }
}

impl<const N: usize> From<LatinSquare<N>> for [[u8; N]; N] {
    fn from(value: LatinSquare<N>) -> Self {
        value.values
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
