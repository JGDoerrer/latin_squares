use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Write},
    mem::MaybeUninit,
};

use crate::{
    bitset::{BitSet128, BitSet16},
    latin_square_trait::{LatinSquareTrait, PartialLatinSquareTrait},
    oa_generator::OAGenerator,
    partial_latin_square::PartialLatinSquare,
    permutation::{
        generate_cycle_structures, Permutation, PermutationDyn, PermutationDynIter,
        PermutationIter, CYCLE_STRUCTURES,
    },
    tuple_iterator::TupleIterator,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LatinSquare<const N: usize> {
    rows: [[u8; N]; N],
}

#[derive(Debug, Clone, Copy)]
pub struct Cell(pub usize, pub usize);

impl<const N: usize> PartialLatinSquareTrait for LatinSquare<N> {
    fn n(&self) -> usize {
        N
    }

    fn get_partial(&self, row: usize, col: usize) -> Option<usize> {
        Some(self.rows[row][col].into())
    }
}

impl<const N: usize> LatinSquareTrait for LatinSquare<N> {
    fn get(&self, row: usize, col: usize) -> usize {
        self.rows[row][col].into()
    }
}

impl<const N: usize> LatinSquare<N> {
    pub fn new(values: [[u8; N]; N]) -> Self {
        debug_assert!(Self::is_valid(&values));

        LatinSquare { rows: values }
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
        &self.rows[i]
    }

    pub fn get_col(&self, i: usize) -> [u8; N] {
        let mut col = [0; N];

        for (j, val) in col.iter_mut().enumerate() {
            *val = self.rows[j][i];
        }

        col
    }

    /// Returns for each row, in which column the value `i` appears
    pub fn get_val(&self, i: usize) -> [u8; N] {
        let mut val = [0; N];

        for j in 0..N {
            val[j] = self
                .get_row(j)
                .iter()
                .position(|v| *v as usize == i)
                .unwrap() as u8;
        }

        val
    }

    pub fn values(self) -> [[u8; N]; N] {
        self.rows
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
            if self.rows[0][i] != i as u8 || self.rows[i][0] != i as u8 {
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
                let row_reduced = sq.permuted_vals(&permutation).permuted_cols(&permutation);
                let reduced = row_reduced.permuted_rows(&Permutation::from_array(
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

    pub fn num_transversals(&self) -> usize {
        let mut count = 0;

        'p: for permutation in PermutationIter::<N>::new() {
            let mut used_cols = [false; N];

            for i in 0..N {
                let val = permutation.as_array()[i];

                let col = self
                    .get_row(i)
                    .iter()
                    .position(|v| *v as usize == val)
                    .unwrap();

                if used_cols[col] {
                    continue 'p;
                } else {
                    used_cols[col] = true;
                }
            }

            count += 1;
        }

        count
    }

    pub fn transversals(&self) -> impl Iterator<Item = [usize; N]> + '_ {
        // TODO: not use permutations
        PermutationIter::<N>::new().filter_map(|permutation| {
            let mut used_cols = [false; N];

            for i in 0..N {
                let val = permutation.as_array()[i];

                let col = self
                    .get_row(i)
                    .iter()
                    .position(|v| *v as usize == val)
                    .unwrap();

                if used_cols[col] {
                    return None;
                } else {
                    used_cols[col] = true;
                }
            }

            let mut cols = [0; N];
            for (row, v) in permutation.into_array().into_iter().enumerate() {
                cols[row] = self
                    .get_row(row)
                    .iter()
                    .position(|a| *a as usize == v)
                    .unwrap();
            }

            Some(cols)
        })
    }

    // pub fn max_disjoint_transversals(&self) -> usize {
    //     let mut max = 0;

    //     let transversals: Vec<_> = self.transversals().collect();
    //     for (i, transversal) in transversals.iter().enumerate() {
    //         // TODO: check all possibilities with backtracking
    //         let mut disjoint = vec![transversal];

    //         for other in transversals.iter().skip(i + 1) {
    //             let is_disjoint = disjoint
    //                 .iter()
    //                 .all(|t| other.iter().zip(*t).all(|(a, b)| *a != *b));

    //             if is_disjoint {
    //                 disjoint.push(other);
    //             }
    //         }

    //         max = max.max(disjoint.len());
    //         if max == N {
    //             break;
    //         }
    //     }

    //     max
    // }

    pub fn max_disjoint_transversals(&self) -> usize {
        let mut transversals_by_start = [(); N].map(|_| Vec::new());

        for t in self.transversals() {
            transversals_by_start[t[0]].push(t);
        }

        transversals_by_start[0]
            .iter()
            .map(|transversal| {
                let mut disjoint = vec![*transversal];
                let mut max_len = 1;

                let mut indices = vec![0];

                'i: while let Some(index) = indices.last_mut() {
                    let i = disjoint.len();

                    for other in transversals_by_start[i].iter().skip(*index) {
                        *index += 1;

                        let is_disjoint = disjoint
                            .iter()
                            .all(|t| other.iter().zip(t).all(|(a, b)| *a != *b));

                        if is_disjoint {
                            disjoint.push(*other);
                            max_len = max_len.max(disjoint.len());
                            if max_len == N {
                                return max_len;
                            }
                            indices.push(0);
                            continue 'i;
                        }
                    }

                    indices.pop();
                    disjoint.pop();
                }

                max_len
            })
            .max()
            .unwrap_or(0)
    }

    pub fn full_disjoint_transversals(&self) -> impl Iterator<Item = [[usize; N]; N]> {
        let mut transversals_by_start = [(); N].map(|_| Vec::new());

        for t in self.transversals() {
            transversals_by_start[t[0]].push(t);
        }

        transversals_by_start[0]
            .clone()
            .into_iter()
            .flat_map(move |transversal| {
                let mut disjoint = [[0; N]; N];
                disjoint[0] = transversal;
                let mut all = vec![];

                let mut indices = vec![0];

                'i: while !indices.is_empty() {
                    let i = indices.len();
                    let index = indices.last_mut().unwrap();

                    for other in transversals_by_start[i].iter().skip(*index) {
                        *index += 1;

                        let is_disjoint = disjoint
                            .iter()
                            .take(i)
                            .all(|t| other.iter().zip(t).all(|(a, b)| *a != *b));

                        if is_disjoint {
                            disjoint[i] = *other;

                            if i == N - 1 {
                                all.push(disjoint);
                            } else {
                                indices.push(0);
                            }
                            continue 'i;
                        }
                    }

                    indices.pop();
                }

                all
            })
    }

    pub fn orthogonal_squares(&self) -> impl Iterator<Item = LatinSquare<N>> + '_ {
        self.full_disjoint_transversals().map(move |transversals| {
            let mut rows = [[0; N]; N];

            for (i, t) in transversals.into_iter().enumerate() {
                for (row, col) in t.into_iter().enumerate() {
                    rows[row][col] = i as u8;
                }
            }

            let sq = LatinSquare::new(rows);
            debug_assert!(self.is_orthogonal_to(&sq));

            sq
        })
    }

    pub fn reduced(&self) -> Self {
        let first_row = self.get_row(0).map(|i| i as usize);
        let row_reduced = self.permuted_cols(&Permutation::from_array(first_row));

        let first_col = row_reduced.get_col(0).map(|i| i as usize);
        let reduced = row_reduced.permuted_rows(&Permutation::from_array(first_col));

        debug_assert!(reduced.is_reduced(), "{reduced:?}");

        reduced
    }

    pub fn all_reduced(&self) -> impl Iterator<Item = Self> + '_ {
        (0..N).map(|i| {
            let mut new_values = self.rows;
            new_values.swap(0, i);

            Self::new(new_values).reduced()
        })
    }

    /// returns all permutations of rows, columns and values
    pub fn conjugates(&self) -> impl Iterator<Item = Self> + '_ {
        PermutationIter::new().map(|perm| self.permuted_rcv(&perm))
    }

    pub fn isotopy_class_permutation(&self) -> (Self, [Permutation<N>; 3]) {
        let sq = &self.reduced();

        let mut isotopic = *sq;
        let mut isotopic_permutation = [
            Permutation::identity(),
            Permutation::identity(),
            Permutation::identity(),
        ];

        let mut candidates = Vec::new();
        let mut min_cycles = vec![N];

        for [row0, row1] in
            TupleIterator::<2>::new(N).flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
        {
            let rows = [*sq.get_row(row0), *sq.get_row(row1)];
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let mut cycles: Vec<_> = row_permutation
                .cycles()
                .into_iter()
                .map(|c| c.len())
                .collect();
            cycles.sort();

            if cycles < min_cycles {
                min_cycles = cycles.clone();
                candidates.clear();
            }

            if cycles == min_cycles {
                candidates.push(rows);
            }
        }

        for rows in candidates {
            let permutations = minimize_rows(&rows);

            for (s, c) in permutations {
                let mut new_sq = sq.permuted_vals(&s).permuted_cols(&c);
                let mut r = Permutation::identity();

                for i in 0..N {
                    let index = new_sq
                        .rows
                        .iter()
                        .position(|row| row[0] as usize == i)
                        .unwrap();
                    new_sq.rows.swap(i, index);
                    r.swap(i, index);
                }

                // new_sq.rows.sort(); // sort rows

                if new_sq.cmp_rows(&isotopic).is_lt() {
                    isotopic = new_sq;
                    isotopic_permutation = [r, c, s];
                }
            }
        }

        // dbg!(self.to_string());
        // assert_eq!(self.reduced_paratopic_old(), paratopic);

        (isotopic, isotopic_permutation)
    }

    pub fn isotopy_class(&self) -> Self {
        self.isotopy_class_permutation().0
    }

    pub fn isotopy_class_lookup(
        &self,
        lookup: &[Vec<(Permutation<N>, Permutation<N>)>],
    ) -> Self {
        let mut isotopy_class = *self;
        let mut min_cycles = vec![N];
        let mut candidates = Vec::new();

        for [row0, row1] in
            TupleIterator::<2>::new(N).flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
        {
            let rows = [*self.get_row(row0), *self.get_row(row1)];
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let mut cycles: Vec<_> = row_permutation.cycle_lengths();
            cycles.sort();

            if cycles < min_cycles {
                min_cycles = cycles.clone();
                candidates.clear();
            }
            if cycles == min_cycles {
                candidates.push((rows, cycles));
            }
        }

        for (rows, _) in candidates {
            let permutations = minimize_rows_with_lookup(&rows, lookup);

            for (s, c) in permutations {
                let mut new_sq = *self;
                new_sq.permute_cols_vals_simd(c, s);

                new_sq.rows.sort();

                if new_sq.cmp_rows(&isotopy_class).is_lt() {
                    isotopy_class = new_sq;
                }
            }
        }

        isotopy_class
    }

    pub fn symmetries(&self) -> Vec<Permutation<3>> {
        let (isotopy_class, _) = self.isotopy_class_permutation();

        let mut symmetries = Vec::new();
        for (sq, permutation) in
            PermutationIter::new().map(|permutation| (self.permuted_rcv(&permutation), permutation))
        {
            if sq.isotopy_class_permutation().0 == isotopy_class {
                symmetries.push(permutation);
            }
        }
        symmetries
    }

    pub fn main_class_old(&self) -> Self {
        let sq = &self.reduced();

        let mut paratopic = *sq;

        for sq in sq.conjugates() {
            for s in PermutationIter::new() {
                let sq = sq.permuted_vals(&s);
                for c in PermutationIter::new() {
                    let sq = sq.permuted_cols(&c);
                    let sq = sq
                        .permuted_rows(&Permutation::from_array(sq.get_col(0).map(|i| i as usize)));

                    if sq.cmp_rows(&paratopic).is_lt() {
                        paratopic = sq;
                    }
                }
            }
        }

        paratopic
    }

    pub fn main_class(&self) -> Self {
        let sq = &self.reduced();

        let mut paratopic = *sq;
        let mut min_cycles = vec![N];

        for sq in sq.conjugates() {
            let mut candidates = Vec::new();

            for [row0, row1] in
                TupleIterator::<2>::new(N).flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
            {
                let rows = [*sq.get_row(row0), *sq.get_row(row1)];
                let row_permutation = {
                    let mut permutation = [0; N];

                    for i in 0..N {
                        let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                        permutation[i] = rows[1][position].into();
                    }

                    Permutation::from_array(permutation)
                };

                let mut cycles: Vec<_> = row_permutation.cycle_lengths();
                cycles.sort();

                if cycles < min_cycles {
                    min_cycles = cycles.clone();
                    candidates.clear();
                }
                if cycles == min_cycles {
                    candidates.push((rows, cycles));
                }
            }

            for (rows, _) in candidates {
                let permutations = minimize_rows(&rows);

                for (s, c) in permutations {
                    let mut new_sq = sq;
                    new_sq.permute_cols_vals_simd(c, s);
                    new_sq.rows.sort();

                    if new_sq.cmp_rows(&paratopic).is_lt() {
                        paratopic = new_sq;
                    }
                }
            }
        }

        // dbg!(self.to_string());
        // assert_eq!(self.reduced_paratopic_old(), paratopic);

        paratopic
    }

    pub fn main_class_lookup(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> Self {
        debug_assert!(self.is_reduced());

        let mut main_class = *self;
        let mut min_cycle_index = CYCLE_STRUCTURES[N].len() - 1;

        for sq in self.conjugates() {
            let mut candidates = Vec::new();

            for [row0, row1] in
                TupleIterator::<2>::new(N).flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
            {
                let rows = [*sq.get_row(row0), *sq.get_row(row1)];
                let row_permutation = {
                    let mut permutation = [0; N];

                    for i in 0..N {
                        let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                        permutation[i] = rows[1][position].into();
                    }

                    Permutation::from_array(permutation)
                };

                let cycles = row_permutation.cycle_lengths_index();

                if cycles < min_cycle_index {
                    min_cycle_index = cycles;
                    candidates.clear();
                }
                if cycles == min_cycle_index {
                    candidates.push((rows, row_permutation));
                }
            }

            for (rows, row_permutation) in candidates {
                let mut cycles = row_permutation.cycles();
                cycles.sort_by_key(|c| c.len());

                let symbol_permutation = {
                    let mut permutation = [0; N];

                    let mut index = 0;
                    for cycle in cycles {
                        let cycle_len = cycle.len();
                        let start_index = index;
                        index += cycle_len;
                        for (i, j) in cycle.into_iter().enumerate() {
                            permutation[j] = start_index + (i + 1) % cycle_len;
                        }
                    }

                    Permutation::from_array(permutation)
                };

                let column_permutation =
                    Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v.into())));

                let permutations = &lookup[min_cycle_index];

                let mut sq = sq;
                sq.permute_cols_vals_simd(column_permutation.inverse(), symbol_permutation);

                for (s, inverse_c) in permutations {
                    let mut new_sq = sq;
                    new_sq.permute_cols_vals_simd(inverse_c.clone(), s.clone());

                    let mut new_rows = [[0; N]; N];
                    for i in 0..N {
                        new_rows[new_sq.rows[i][0] as usize] = new_sq.rows[i];
                    }

                    let new_sq = LatinSquare::new(new_rows);

                    if new_sq.cmp_rows(&main_class).is_lt() {
                        main_class = new_sq;
                    }
                }
            }
        }

        // dbg!(self.to_string());
        // assert_eq!(self.main_class(), paratopic);

        main_class
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
                let solutions =
                    OAGenerator::<N, 1>::from_partial_sq(partial).map(|s| s.squares()[0]);

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
                            break;
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
                values[i][j] = self.rows[rows[i]][*col];
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

    pub fn intercalates(&self) -> usize {
        self.subsquares_order_2_iter().count()
    }

    pub fn row_cycles(&self) -> Vec<Vec<usize>> {
        let mut cycles = Vec::new();

        for rows in TupleIterator::<2>::new(N).map(|rows| rows.map(|row| self.get_row(row))) {
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let mut cycle: Vec<_> = row_permutation.cycle_lengths();
            cycle.sort();

            cycles.push(cycle);
        }

        cycles.sort();
        cycles
    }

    pub fn col_cycles(&self) -> Vec<Vec<usize>> {
        let mut cycles = Vec::new();

        for cols in TupleIterator::<2>::new(N).map(|cols| cols.map(|row| self.get_col(row))) {
            let col_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = cols[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = cols[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let mut cycle: Vec<_> = col_permutation.cycle_lengths();
            cycle.sort();

            cycles.push(cycle);
        }

        cycles.sort();
        cycles
    }

    pub fn val_cycles(&self) -> Vec<Vec<usize>> {
        let mut cycles = Vec::new();

        for vals in TupleIterator::<2>::new(N).map(|vals| vals.map(|val| self.get_val(val))) {
            let val_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = vals[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = vals[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let mut cycle: Vec<_> = val_permutation.cycle_lengths();
            cycle.sort();

            cycles.push(cycle);
        }

        cycles.sort();
        cycles
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

    pub fn permuted_rcv(&self, permutation: &Permutation<3>) -> Self {
        let mut rows = [[0; N]; N];
        for (i, row) in rows.iter_mut().enumerate() {
            *row = [i; N];
        }

        let mut col = [0; N];

        for (i, val) in col.iter_mut().enumerate() {
            *val = i;
        }

        let cols = [col; N];
        let vals = self.rows.map(|row| row.map(|val| val as usize));

        let [rows, cols, vals] = permutation.apply_array([rows, cols, vals]);
        Self::from_rcv(rows, cols, vals)
    }

    pub fn permuted_rows(&self, permutation: &Permutation<N>) -> Self {
        let new_values = permutation.apply_array(self.rows);

        Self::new(new_values)
    }

    pub fn permuted_cols(&self, permutation: &Permutation<N>) -> Self {
        let mut new = *self;
        new.permute_cols(permutation);
        new
    }

    pub fn permute_cols(&mut self, permutation: &Permutation<N>) {
        permutation.apply_arrays(&mut self.rows);
    }

    pub fn permute_cols_simd(&mut self, permutation: &Permutation<N>) {
        use std::simd::Simd;

        assert!(N <= 16);

        let mut permutation_simd = [0; 16];
        permutation_simd[0..N]
            .copy_from_slice(&permutation.clone().inverse().into_array().map(|v| v as u8));
        let permutation = Simd::from_array(permutation_simd);

        for i in 0..N {
            let mut simd = [0; 16];
            simd[0..N].copy_from_slice(&self.rows[i]);
            let simd = Simd::from_array(simd);
            let new_row = simd.swizzle_dyn(permutation);
            self.rows[i].copy_from_slice(&new_row[0..N]);
        }
    }

    pub fn permuted_rows_and_cols(&self, permutation: &Permutation<N>) -> Self {
        let mut values = [[MaybeUninit::uninit(); N]; N];

        for (i, new_row) in values.iter_mut().enumerate() {
            let row = self.rows[permutation.apply(i)];
            for j in 0..N {
                new_row[j].write(row[permutation.apply(j)]);
            }
        }

        let values = values.map(|row| row.map(|val| unsafe { val.assume_init() }));

        Self { rows: values }
    }

    pub fn permuted_vals(&self, permutation: &Permutation<N>) -> Self {
        let mut new = *self;
        new.permute_vals(permutation);
        new
    }

    pub fn permute_vals(&mut self, permutation: &Permutation<N>) {
        for row in &mut self.rows {
            for val in row {
                *val = permutation.apply_u8(*val);
            }
        }
    }

    pub fn permute_vals_simd(&mut self, permutation: &Permutation<N>) {
        use std::simd::Simd;

        assert!(N <= 16);

        let mut permutation_simd = [0; 16];
        permutation_simd[0..N].copy_from_slice(&permutation.clone().into_array().map(|v| v as u8));
        let permutation = Simd::from_array(permutation_simd);

        for i in 0..N {
            let mut simd = [0; 16];
            simd[0..N].copy_from_slice(&self.rows[i]);
            let simd = Simd::from_array(simd);
            let new_row = permutation.swizzle_dyn(simd);
            self.rows[i].copy_from_slice(&new_row[0..N]);
        }
    }

    pub fn permute_cols_vals_simd(
        &mut self,
        inverse_col_permutation: Permutation<N>,
        val_permutation: Permutation<N>,
    ) {
        use std::simd::Simd;

        assert!(N <= 16);

        let mut col_permutation_simd = [0; 16];
        col_permutation_simd[0..N]
            .copy_from_slice(&inverse_col_permutation.as_array().map(|v| v as u8));
        let col_permutation = Simd::from_array(col_permutation_simd);

        let mut val_permutation_simd = [0; 16];
        val_permutation_simd[0..N].copy_from_slice(&val_permutation.as_array().map(|v| v as u8));
        let val_permutation = Simd::from_array(val_permutation_simd);

        for i in 0..N {
            let mut simd = [0; 16];
            simd[0..N].copy_from_slice(&self.rows[i]);
            let simd = Simd::from_array(simd);
            let new_row = val_permutation
                .swizzle_dyn(simd)
                .swizzle_dyn(col_permutation);

            self.rows[i].copy_from_slice(&new_row[0..N]);
        }
    }

    pub fn cmp_diagonal(&self, other: &Self) -> Ordering {
        for i in 0..N {
            for j in (0..=i).rev() {
                match self.rows[j][i].cmp(&other.rows[j][i]) {
                    Ordering::Equal => {}
                    o => return o,
                }
                match self.rows[i][j].cmp(&other.rows[i][j]) {
                    Ordering::Equal => {}
                    o => return o,
                }
            }
        }

        Ordering::Equal
    }

    pub fn cmp_rows(&self, other: &Self) -> Ordering {
        self.rows.cmp(&other.rows)
    }

    pub fn cmp_row_col(&self, other: &Self) -> Ordering {
        for i in 0..N {
            for j in i..N {
                match self.rows[i][j].cmp(&other.rows[i][j]) {
                    Ordering::Equal => {}
                    o => return o,
                }
            }
            for j in i + 1..N {
                match self.rows[j][i].cmp(&other.rows[j][i]) {
                    Ordering::Equal => {}
                    o => return o,
                }
            }
        }

        Ordering::Equal
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
        let mut sq = LatinSquare { rows: [[0; N]; N] };

        for i in 0..N {
            for j in 0..N {
                sq.rows[i][j] = value.get_partial(i, j).unwrap() as u8;
            }
        }

        Ok(sq)
    }
}

impl<const N: usize> From<LatinSquare<N>> for [[u8; N]; N] {
    fn from(value: LatinSquare<N>) -> Self {
        value.rows
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

struct CyclePermutations<const N: usize> {
    cycles_by_len: [Vec<Vec<usize>>; N],
    cycle_permutations: [Option<(PermutationDyn, PermutationDynIter)>; N],
    per_cycle_permutation: [[usize; N]; N],
    rows: [[u8; N]; 2],
}

impl<const N: usize> CyclePermutations<N> {
    fn new(rows: [[u8; N]; 2]) -> Self {
        let row_permutation = {
            let mut permutation = [0; N];

            for i in 0..N {
                let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                permutation[i] = rows[1][position].into();
            }

            Permutation::from_array(permutation)
        };

        let cycles = row_permutation.cycles();

        let cycles_by_len = {
            const EMPTY_VEC: Vec<Vec<usize>> = Vec::new();
            let mut array = [EMPTY_VEC; N];

            for cycle in cycles {
                array[cycle.len() - 1].push(cycle);
            }

            for i in 0..N {
                array[i].sort();
            }

            array
        };

        let per_cycle_permutation = [[0; N]; N];

        const NONE: Option<(PermutationDyn, PermutationDynIter)> = None;
        let mut cycle_permutations = [NONE; N];

        for i in 0..N {
            if cycles_by_len[i].is_empty() {
                continue;
            }
            let mut iter = PermutationDynIter::new(cycles_by_len[i].len());
            cycle_permutations[i] = Some((iter.next().unwrap(), iter));
        }

        CyclePermutations {
            rows,
            cycles_by_len,
            cycle_permutations,
            per_cycle_permutation,
        }
    }

    fn next_permutation(&mut self) -> bool {
        for i in 0..N {
            if self.cycles_by_len[i].is_empty() {
                continue;
            }

            for j in self.per_cycle_permutation[i]
                .iter_mut()
                .take(self.cycles_by_len[i].len())
            {
                if *j == i + 1 {
                    *j = 0;
                } else {
                    *j += 1;
                    return false;
                }
            }
        }

        for i in 0..N {
            let Some((permutation, iter)) = &mut self.cycle_permutations[i] else {
                continue;
            };

            let next = iter.next();
            if let Some(next) = next {
                *permutation = next;
                return false;
            } else {
                *iter = PermutationDynIter::new(self.cycles_by_len[i].len());

                *permutation = iter.next().unwrap();
            }
        }

        true
    }
}

impl<const N: usize> Iterator for CyclePermutations<N> {
    type Item = (Permutation<N>, Permutation<N>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_permutation() {
            return None;
        }

        let symbol_permutation = {
            let mut permutation = [0; N];

            let mut index = 0;

            for (i, cycles) in self.cycles_by_len.iter().enumerate() {
                let cycle_len = i + 1;
                for cycle_index in 0..cycles.len() {
                    let start_index = index;
                    index += cycle_len;

                    let cycle_index = self.cycle_permutations[i]
                        .as_ref()
                        .unwrap()
                        .0
                        .apply(cycle_index);

                    for j in 0..cycle_len {
                        let cycle = &cycles[cycle_index];

                        let offset = self.per_cycle_permutation[i][cycle_index];

                        let v = cycle[(j + offset) % cycle.len()];

                        permutation[v] = start_index + j;
                    }
                }
            }

            Permutation::from_array(permutation)
        };

        let column_permutation =
            Permutation::from_array(self.rows[0].map(|i| symbol_permutation.apply(i.into())))
                .inverse();

        Some((symbol_permutation, column_permutation))
    }
}

pub fn generate_minimize_rows_lookup<const N: usize>() -> Vec<Vec<(Permutation<N>, Permutation<N>)>>
{
    generate_cycle_structures(N)
        .into_iter()
        .map(|cycle| {
            let mut rows = [[0; N]; 2];

            for i in 0..N {
                rows[0][i] = i as u8;
            }

            let mut index = 0;
            for cycle in cycle {
                let start_index = index;
                index += cycle;
                for j in 0..cycle {
                    rows[1][start_index + j] = (start_index + (j + 1) % cycle) as u8;
                }
            }

            let cycle_permutations = CyclePermutations::new(rows);
            let mut permutations: Vec<_> = cycle_permutations.collect();

            permutations.sort_unstable();
            permutations.dedup();
            permutations.shrink_to_fit();

            permutations
        })
        .collect()
}

pub fn minimize_rows<const N: usize>(rows: &[[u8; N]; 2]) -> Vec<(Permutation<N>, Permutation<N>)> {
    let cycle_permutations = CyclePermutations::new(*rows);
    let mut permutations: Vec<_> = cycle_permutations.collect();

    permutations.sort_unstable();
    permutations.dedup();

    permutations
}

pub fn minimize_rows_with_lookup<'a, const N: usize>(
    rows: &[[u8; N]; 2],
    lookup: &'a [Vec<(Permutation<N>, Permutation<N>)>],
) -> Box<dyn Iterator<Item = (Permutation<N>, Permutation<N>)> + 'a> {
    // find (s,c) to normalize
    let row_permutation = {
        let mut permutation = [0; N];

        for i in 0..N {
            let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
            permutation[i] = rows[1][position].into();
        }

        Permutation::from_array(permutation)
    };

    let mut cycles = row_permutation.cycles();
    cycles.sort_by_key(|c| c.len());

    let cycle_lengths: Vec<_> = cycles.iter().map(|c| c.len()).collect();

    let symbol_permutation = {
        let mut permutation = [0; N];

        let mut index = 0;
        for cycle in cycles {
            let cycle_len = cycle.len();
            let start_index = index;
            index += cycle_len;
            for (i, j) in cycle.into_iter().enumerate() {
                permutation[j] = start_index + (i + 1) % cycle_len;
            }
        }

        Permutation::from_array(permutation)
    };

    let column_permutation =
        Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v.into())));

    // dbg!(
    //     rows,
    //     rows.map(|row| row.map(|v| symbol_permutation.apply_u8(v))),
    //     rows.map(|row| column_permutation.apply_array(row.map(|v| symbol_permutation.apply_u8(v))))
    // );

    // lookup
    let cycle_index = CYCLE_STRUCTURES[N]
        .iter()
        .position(|c| c == &cycle_lengths)
        .unwrap();

    let permutations = &lookup[cycle_index];

    // fix lookup by (s,c)
    let symbol_permutation = symbol_permutation.inverse();
    let column_permutation = column_permutation.inverse();

    let permutations = permutations.iter().map(move |(s, c)| {
        (
            Permutation::from_array(symbol_permutation.apply_array(s.clone().into_array())),
            Permutation::from_array(column_permutation.apply_array(c.clone().into_array())),
        )
    });

    // for (s, c) in permutations.clone() {
    //     // dbg!(rows.map(|row| c.apply_array(row.map(|v| s.apply_u8(v)))));
    //     assert_eq!(
    //         rows.map(
    //             |row| column_permutation.apply_array(row.map(|v| symbol_permutation.apply_u8(v)))
    //         ),
    //         rows.map(|row| c.apply_array(row.map(|v| s.apply_u8(v)))),
    //     )
    // }

    Box::new(permutations)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn normalize_main_class() {
        assert_eq!(
            LatinSquare::new([[0, 1, 2, 3], [1, 3, 0, 2], [2, 0, 3, 1], [3, 2, 1, 0]]).main_class(),
            LatinSquare::new([[0, 1, 2, 3], [1, 0, 3, 2], [2, 3, 1, 0], [3, 2, 0, 1]])
        )
    }

    #[test]
    fn transversal() {}
}
