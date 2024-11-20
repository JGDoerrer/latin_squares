use std::{
    array,
    cmp::Ordering,
    fmt::{Debug, Display, Write},
};

use crate::{
    bitset::{BitSet128, BitSet16},
    cycles::{minimize_rows, CYCLE_STRUCTURES},
    mols::Mols,
    partial_latin_square::PartialLatinSquare,
    permutation::{Permutation, PermutationIter},
    permutation_dyn::PermutationDyn,
    tuple_iterator::{TupleIterator, TupleIteratorDyn},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LatinSquare<const N: usize> {
    rows: [[u8; N]; N],
}

impl<const N: usize> LatinSquare<N> {
    pub fn new(values: [[u8; N]; N]) -> Self {
        debug_assert!(Self::is_valid(&values));

        LatinSquare { rows: values }
    }

    pub fn get(&self, row: usize, col: usize) -> usize {
        self.rows[row][col].into()
    }

    pub fn from_rcs(rows: [[usize; N]; N], cols: [[usize; N]; N], vals: [[usize; N]; N]) -> Self {
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

    pub fn to_values(self) -> [[u8; N]; N] {
        self.rows
    }

    pub fn values(&self) -> &[[u8; N]; N] {
        &self.rows
    }

    pub fn is_valid(values: &[[u8; N]; N]) -> bool {
        assert!(N <= 16);

        (0..N).all(|i| {
            (0..N).map(|j| values[i][j] as usize).collect::<BitSet16>()
                == BitSet16::all_less_than(N)
                && (0..N).map(|j| values[j][i] as usize).collect::<BitSet16>()
                    == BitSet16::all_less_than(N)
        })
    }

    pub fn is_orthogonal_to(&self, other: &Self) -> bool {
        assert!(N <= 16);

        for value in 0..N {
            let mut other_values = BitSet16::empty();

            for i in 0..N {
                for j in 0..N {
                    if self.get(i, j) == value {
                        other_values.insert(other.get(i, j));
                    }
                }
            }

            if other_values != BitSet16::all_less_than(N) {
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

    pub fn num_transversals(&self) -> usize {
        self.transversals_bitset().len()
    }

    const BITSET_COLS: [BitSet128; N] = {
        let mut bitsets = [BitSet128::empty(); N];
        let mut i = 0;
        while i < N {
            let mut j = 0;
            while j < N {
                bitsets[i].insert(j * N + i);

                j += 1;
            }
            i += 1;
        }
        bitsets
    };

    const BITSET_ROWS: [BitSet128; N] = {
        let mut bitsets = [BitSet128::empty(); N];
        let mut i = 0;
        while i < N {
            let mut j = 0;
            while j < N {
                bitsets[i].insert(j + i * N);

                j += 1;
            }
            i += 1;
        }
        bitsets
    };

    pub fn transversals_bitset(&self) -> Vec<BitSet128> {
        assert!(N * N <= 128);
        assert!(N <= 16);

        let mut indices = [0; N];

        let mut bitsets = Vec::new();

        // let bits: [[BitSet16; N]; N] = self.rows.map(|row| row.map(|v| BitSet16::single(v.into())));

        let mut value_bitsets = [BitSet128::empty(); N];

        for i in 0..N {
            let cols = self.get_val(i);

            let mut bitset = BitSet128::empty();
            for (i, j) in cols.into_iter().enumerate() {
                bitset.insert(i * N + j as usize);
            }

            value_bitsets[i] = bitset;
        }

        let value_bitsets = value_bitsets;

        'l: loop {
            let mut unused_vals = BitSet16::all_less_than(N);
            let mut bitset = BitSet128::empty();

            let mut used_cols = BitSet128::empty();

            for i in 0..N {
                let index = indices[i];

                let bitset_row = Self::BITSET_ROWS[i];

                if let Some((val, index)) = unused_vals
                    .into_iter()
                    .filter_map(|val| {
                        let index = value_bitsets[val]
                            .intersect(bitset_row)
                            .intersect(used_cols.complement())
                            .into_iter()
                            .next()?;

                        Some((val, index))
                    })
                    .nth(index)
                {
                    bitset.insert(index);
                    unused_vals.remove(val);

                    let col = index % N;
                    used_cols = used_cols.union(Self::BITSET_COLS[col]);
                } else if i != 0 {
                    indices[i - 1] += 1;
                    for j in i..N {
                        indices[j] = 0;
                    }
                    continue 'l;
                } else {
                    break 'l;
                }
            }

            indices[N - 1] += 1;
            // bitset.print_sq(N);
            bitsets.push(bitset);
        }

        bitsets
    }

    pub fn max_disjoint_transversals(&self) -> usize {
        let mut transversals_by_start = [(); N].map(|_| Vec::new());

        for t in self.transversals_bitset() {
            let first = t
                .intersect(BitSet128::all_less_than(N))
                .into_iter()
                .next()
                .unwrap();
            transversals_by_start[first].push(t);
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

                        let is_disjoint = disjoint.iter().all(|t| other.is_disjoint(*t));

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

    pub fn full_disjoint_transversals_bitset(&self) -> Vec<[BitSet128; N]> {
        let mut transversals_by_start: [[Vec<_>; N]; N] =
            array::from_fn(|_| array::from_fn(|_| Vec::new()));

        let transversals = self.transversals_bitset();

        for t in transversals {
            let first = t
                .intersect(BitSet128::all_less_than(N))
                .into_iter()
                .next()
                .unwrap();
            let second = t
                .intersect(BitSet128::from_range(N..2 * N))
                .into_iter()
                .next()
                .unwrap()
                - N;
            transversals_by_start[first][second].push(t);
        }

        let mut disjoint_transversals = Vec::new();

        for i in 0..N {
            for transversal in &transversals_by_start[0][i] {
                let mut disjoint = [BitSet128::empty(); N];
                disjoint[0] = *transversal;

                let second_row_left = transversal
                    .complement()
                    .intersect(BitSet128::from_range(N..2 * N))
                    .shift_right(N);
                let mut indices = vec![(0, second_row_left, *transversal)];

                'i: while !indices.is_empty() {
                    let i = indices.len();

                    if i == N - 1 {
                        let (_, second_row_left, union) = indices.last().unwrap();

                        let left = union
                            .complement()
                            .intersect(BitSet128::all_less_than(N * N));

                        debug_assert!(second_row_left.len() == 1);
                        let second_row = second_row_left.into_iter().next().unwrap();

                        if transversals_by_start[N - 1][second_row].contains(&left) {
                            disjoint[N - 1] = left;
                            disjoint_transversals.push(disjoint);
                            if disjoint_transversals.len() % 1000 == 0 {
                                dbg!(disjoint_transversals.len());
                            }
                        }
                    } else {
                        let (index, second_row_left, union) = indices.last_mut().unwrap();

                        while let Some(second_row) = second_row_left.into_iter().next() {
                            for other in transversals_by_start[i][second_row].iter().skip(*index) {
                                *index += 1;

                                if union.is_disjoint(*other) {
                                    disjoint[i] = *other;

                                    let union = union.union(*other);

                                    let next_second_row_left = union
                                        .complement()
                                        .intersect(BitSet128::from_range(N..2 * N))
                                        .shift_right(N);

                                    indices.push((0, next_second_row_left, union));
                                    continue 'i;
                                }
                            }
                            *index = 0;
                            second_row_left.pop();
                        }
                    }

                    indices.pop();
                }
            }
        }

        disjoint_transversals
    }

    pub fn orthogonal_squares(&self) -> impl Iterator<Item = LatinSquare<N>> + '_ {
        self.full_disjoint_transversals_bitset()
            .into_iter()
            .map(|transversals| {
                let sq = Self::bitset_transversals_to_sq(&transversals);
                debug_assert!(self.is_orthogonal_to(&sq));

                sq
            })
    }

    fn bitset_transversals_to_sq(transversals: &[BitSet128; N]) -> LatinSquare<N> {
        let mut rows = [[0; N]; N];

        for (i, t) in transversals.iter().enumerate() {
            for index in t.into_iter() {
                let row = index / N;
                let col = index % N;
                rows[row][col] = i as u8;
            }
        }

        LatinSquare::new(rows)
    }

    pub fn mols(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> Vec<Mols<N>> {
        let transversals = self.transversals_bitset();

        let mut indices = vec![0];
        let mut current_mols = vec![*self];
        let mut disjoint_transversals = vec![n_disjoint_transversals_bitset(&transversals)];
        let mut intersections = vec![transversals.clone()];

        let mut all_mols = Vec::new();

        'i: while let Some(index) = indices.last_mut() {
            for disjoint_transversal in disjoint_transversals.last().unwrap().iter().skip(*index) {
                let sq = Self::bitset_transversals_to_sq(&disjoint_transversal);

                *index += 1;

                current_mols.push(sq);

                let new_mols = Mols::new_unchecked(current_mols.clone());

                if let Some(new_mols) = new_mols.normalize_main_class_set_sq(lookup, self) {
                    if !all_mols.contains(&new_mols) {
                        all_mols.push(new_mols);
                        if all_mols.len() % 1000 == 0 {
                            dbg!(&indices, all_mols.len());
                        }
                    }
                }

                let new_transversals = sq.transversals_bitset();
                let mut intersection = intersections.last().unwrap().clone();
                intersection.retain(|t| new_transversals.contains(t));

                disjoint_transversals.push(n_disjoint_transversals_bitset(&intersection));
                intersections.push(intersection);
                indices.push(0);

                continue 'i;
            }

            current_mols.pop();
            indices.pop();
            disjoint_transversals.pop();
            intersections.pop();
        }

        all_mols
    }

    pub fn kmols(
        &self,
        k: usize,
        _lookup: &[Vec<(Permutation<N>, Permutation<N>)>],
    ) -> Vec<Mols<N>> {
        let transversals = self.transversals_bitset();

        let mut indices = vec![0];
        let mut current_mols = vec![*self];
        let mut disjoint_transversals = vec![n_disjoint_transversals_bitset(&transversals)];
        let mut intersections = vec![transversals.clone()];

        let mut all_mols = Vec::new();

        'i: while let Some(index) = indices.last_mut() {
            for disjoint_transversal in disjoint_transversals.last().unwrap().iter().skip(*index) {
                let sq = Self::bitset_transversals_to_sq(&disjoint_transversal);

                *index += 1;

                current_mols.push(sq);

                if current_mols.len() == k {
                    let new_mols = Mols::new_unchecked(current_mols.clone());

                    all_mols.push(new_mols);
                    if all_mols.len() % 1000 == 0 {
                        dbg!(all_mols.len());
                    }

                    current_mols.pop();
                    continue;
                } else {
                    let new_transversals = sq.transversals_bitset();
                    let mut intersection = intersections.last().unwrap().clone();
                    intersection.retain(|t| new_transversals.contains(t));

                    disjoint_transversals.push(n_disjoint_transversals_bitset(&intersection));
                    intersections.push(intersection);
                    indices.push(0);

                    continue 'i;
                }
            }

            current_mols.pop();
            indices.pop();
            disjoint_transversals.pop();
            intersections.pop();
        }

        all_mols
    }

    /// Counts how many rows are the same until a differing row is found
    pub fn num_same_rows(&self, other: &Self) -> usize {
        for i in 0..N {
            for j in 0..N {
                if self.get(i, j) != other.get(i, j) {
                    return i;
                }
            }
        }

        N
    }

    /// returns all permutations of rows, columns and values
    pub fn conjugates(&self) -> impl Iterator<Item = Self> + '_ {
        PermutationIter::new().map(|perm| self.permuted_rcs(&perm))
    }

    fn isotopy_class_permutation(&self) -> (Self, [Permutation<N>; 3]) {
        let mut candidates = Vec::new();
        let mut min_cycles = vec![N];

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

            let mut cycles: Vec<_> = row_permutation
                .cycles()
                .into_iter()
                .map(|c| c.len())
                .collect();
            cycles.sort();

            match cycles.cmp(&min_cycles) {
                Ordering::Less => {
                    min_cycles = cycles;
                    candidates.clear();
                    candidates.push(rows);
                }
                Ordering::Equal => {
                    candidates.push(rows);
                }
                Ordering::Greater => {}
            }
        }

        let mut isotopic = *self;
        let mut isotopic_permutation = [
            Permutation::identity(),
            Permutation::identity(),
            Permutation::identity(),
        ];

        for rows in candidates {
            let permutations = minimize_rows(&rows);

            for (s, c) in permutations {
                let c = c.inverse();
                let mut new_sq = self.permuted_vals(&s).permuted_cols(&c);
                let r = Permutation::from_array(new_sq.get_col(0).map(|i| i as usize));

                new_sq = new_sq.permuted_rows(&r);

                if new_sq.cmp_rows(&isotopic).is_lt() {
                    isotopic = new_sq;
                    isotopic_permutation = [r, c, s];
                }
            }
        }

        // assert_eq!(
        //     self.permuted_rows(&isotopic_permutation[0])
        //         .permuted_cols(&isotopic_permutation[1])
        //         .permuted_vals(&isotopic_permutation[2]),
        //     isotopic
        // );

        (isotopic, isotopic_permutation)
    }

    pub fn isotopy_class_permutations(
        &self,
        lookup: &[Vec<(Permutation<N>, Permutation<N>)>],
    ) -> (Self, Vec<[Permutation<N>; 3]>) {
        let mut candidates = Vec::with_capacity(N * N);
        let mut min_cycle_index = CYCLE_STRUCTURES[N].len() - 1;

        for [row0, row1] in
            TupleIterator::<2>::new(N).flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
        {
            let rows = [self.get_row(row0), self.get_row(row1)];
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let cycles = row_permutation.cycle_lengths_index();

            match cycles.cmp(&min_cycle_index) {
                Ordering::Less => {
                    min_cycle_index = cycles;
                    candidates.clear();
                    candidates.push((rows.map(|r| *r), row_permutation));
                }
                Ordering::Equal => {
                    candidates.push((rows.map(|r| *r), row_permutation));
                }
                Ordering::Greater => {}
            }
        }

        let mut isotopic = *self;
        let mut isotopic_permutations = Vec::with_capacity(1000);
        isotopic_permutations.push([
            Permutation::identity(),
            Permutation::identity(),
            Permutation::identity(),
        ]);

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
                Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v.into())))
                    .inverse();

            let permutations = &lookup[min_cycle_index];

            let mut sq = *self;
            sq.permute_cols_vals_simd(&column_permutation, &symbol_permutation);

            for (s, inverse_c) in permutations {
                let mut new_sq = sq;
                new_sq.permute_cols_vals_simd(inverse_c, s);

                let r = Permutation::from_array(new_sq.get_col(0).map(|i| i as usize));

                let mut new_rows = [[0; N]; N];
                for i in 0..N {
                    new_rows[new_sq.rows[i][0] as usize] = new_sq.rows[i];
                }
                let new_sq = LatinSquare::new(new_rows);

                let c = Permutation::from_array(
                    column_permutation.apply_array(inverse_c.inverse().clone().into_array()),
                );
                let s = symbol_permutation
                    .inverse()
                    .apply_array(s.clone().into_array())
                    .into();

                match new_sq.cmp_rows(&isotopic) {
                    Ordering::Less => {
                        isotopic = new_sq;
                        isotopic_permutations.clear();
                        isotopic_permutations.push([r, c, s]);
                    }
                    Ordering::Equal => {
                        isotopic_permutations.push([r, c, s]);
                    }
                    Ordering::Greater => {}
                }
            }
        }

        // for perm in &isotopic_permutations {
        //     assert_eq!(
        //         self.permuted_rows(&perm[0])
        //             .permuted_cols(&perm[1])
        //             .permuted_vals(&perm[2]),
        //         isotopic
        //     );
        // }

        (isotopic, isotopic_permutations)
    }

    pub fn isotopy_class(&self) -> Self {
        self.isotopy_class_permutation().0
    }

    pub fn isotopy_class_lookup(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> Self {
        let mut candidates = Vec::with_capacity(N * N);
        let mut min_cycle_index = CYCLE_STRUCTURES[N].len() - 1;

        for [row0, row1] in
            TupleIterator::<2>::new(N).flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
        {
            let rows = [self.get_row(row0), self.get_row(row1)];
            let row_permutation = {
                let mut permutation = [0; N];

                for i in 0..N {
                    let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                    permutation[i] = rows[1][position].into();
                }

                Permutation::from_array(permutation)
            };

            let cycles = row_permutation.cycle_lengths_index();

            match cycles.cmp(&min_cycle_index) {
                Ordering::Less => {
                    min_cycle_index = cycles;
                    candidates.clear();
                    candidates.push((rows.map(|r| *r), row_permutation));
                }
                Ordering::Equal => {
                    candidates.push((rows.map(|r| *r), row_permutation));
                }
                Ordering::Greater => {}
            }
        }

        let mut isotopic = *self;

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
                Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v.into())))
                    .inverse();

            let permutations = &lookup[min_cycle_index];

            let mut sq = *self;
            sq.permute_cols_vals_simd(&column_permutation, &symbol_permutation);

            for (s, inverse_c) in permutations {
                let mut new_sq = sq;
                new_sq.permute_cols_vals_simd(inverse_c, s);

                let mut new_rows = [[0; N]; N];
                for i in 0..N {
                    new_rows[new_sq.rows[i][0] as usize] = new_sq.rows[i];
                }
                let new_sq = LatinSquare::new(new_rows);

                match new_sq.cmp_rows(&isotopic) {
                    Ordering::Less => {
                        isotopic = new_sq;
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => {}
                }
            }
        }

        // for perm in &isotopic_permutations {
        //     assert_eq!(
        //         self.permuted_rows(&perm[0])
        //             .permuted_cols(&perm[1])
        //             .permuted_vals(&perm[2]),
        //         isotopic
        //     );
        // }

        isotopic
    }

    pub fn symmetries(&self) -> Vec<Permutation<3>> {
        let isotopy_class = self.isotopy_class();

        let mut symmetries = Vec::new();
        for (sq, permutation) in
            PermutationIter::new().map(|permutation| (self.permuted_rcs(&permutation), permutation))
        {
            if sq.isotopy_class() == isotopy_class {
                symmetries.push(permutation);
            }
        }
        symmetries
    }

    pub fn main_class_permutation(&self) -> (Self, Permutation<3>, [Permutation<N>; 3]) {
        let mut min = *self;
        let mut permutation = (
            Permutation::identity(),
            [
                Permutation::identity(),
                Permutation::identity(),
                Permutation::identity(),
            ],
        );

        for (rcs, sq) in PermutationIter::new().map(|rcs| (rcs.clone(), self.permuted_rcs(&rcs))) {
            let (isotopy_class, perm) = sq.isotopy_class_permutation();

            if isotopy_class < min {
                min = isotopy_class;
                permutation = (rcs, perm);
            }
        }

        (min, permutation.0, permutation.1)
    }

    pub fn main_class_permutations(
        &self,
        lookup: &[Vec<(Permutation<N>, Permutation<N>)>],
    ) -> (Self, Vec<(Permutation<3>, [Permutation<N>; 3])>) {
        let mut isotopic = *self;
        let mut isotopic_permutations = Vec::with_capacity(1000);
        isotopic_permutations.push((
            Permutation::identity(),
            [
                Permutation::identity(),
                Permutation::identity(),
                Permutation::identity(),
            ],
        ));

        for (rcs, sq) in PermutationIter::new().map(|rcs| (rcs.clone(), self.permuted_rcs(&rcs))) {
            let mut candidates = Vec::with_capacity(N * N);
            let mut min_cycle_index = CYCLE_STRUCTURES[N].len() - 1;

            for [row0, row1] in
                TupleIterator::<2>::new(N).flat_map(|rows| [[rows[0], rows[1]], [rows[1], rows[0]]])
            {
                let rows = [sq.get_row(row0), sq.get_row(row1)];
                let row_permutation = {
                    let mut permutation = [0; N];

                    for i in 0..N {
                        let position = rows[0].iter().position(|v| *v as usize == i).unwrap();
                        permutation[i] = rows[1][position].into();
                    }

                    Permutation::from_array(permutation)
                };

                let cycles = row_permutation.cycle_lengths_index();

                match cycles.cmp(&min_cycle_index) {
                    Ordering::Less => {
                        min_cycle_index = cycles;
                        candidates.clear();
                        candidates.push((rows.map(|r| *r), row_permutation));
                    }
                    Ordering::Equal => {
                        candidates.push((rows.map(|r| *r), row_permutation));
                    }
                    Ordering::Greater => {}
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
                    Permutation::from_array(rows[0].map(|v| symbol_permutation.apply(v.into())))
                        .inverse();

                let permutations = &lookup[min_cycle_index];

                let mut sq = sq;
                sq.permute_cols_vals_simd(&column_permutation, &symbol_permutation);

                for (s, inverse_c) in permutations {
                    let mut new_sq = sq;
                    new_sq.permute_cols_vals_simd(inverse_c, s);

                    let r = Permutation::from_array(new_sq.get_col(0).map(|i| i as usize));

                    let mut new_rows = [[0; N]; N];
                    for i in 0..N {
                        new_rows[new_sq.rows[i][0] as usize] = new_sq.rows[i];
                    }
                    let new_sq = LatinSquare::new(new_rows);

                    let c = Permutation::from_array(
                        column_permutation.apply_array(inverse_c.inverse().clone().into_array()),
                    );
                    let s = symbol_permutation
                        .inverse()
                        .apply_array(s.clone().into_array())
                        .into();

                    match new_sq.cmp_rows(&isotopic) {
                        Ordering::Less => {
                            isotopic = new_sq;
                            isotopic_permutations.clear();
                            isotopic_permutations.push((rcs.clone(), [r, c, s]));
                        }
                        Ordering::Equal => {
                            isotopic_permutations.push((rcs.clone(), [r, c, s]));
                        }
                        Ordering::Greater => {}
                    }
                }
            }
        }

        for (rcs, perm) in &isotopic_permutations {
            assert_eq!(
                self.permuted_rcs(rcs)
                    .permuted_rows(&perm[0])
                    .permuted_cols(&perm[1])
                    .permuted_vals(&perm[2]),
                isotopic
            );
        }

        (isotopic, isotopic_permutations)
    }

    pub fn main_class_lookup(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> Self {
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
                sq.permute_cols_vals_simd(&column_permutation.inverse(), &symbol_permutation);

                for (s, inverse_c) in permutations {
                    let mut new_sq = sq;
                    new_sq.permute_cols_vals_simd(inverse_c, s);

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
        // assert_eq!(self.main_class(), main_class);

        main_class
    }

    pub fn num_isotopy_classes(&self, lookup: &[Vec<(Permutation<N>, Permutation<N>)>]) -> usize {
        let conjugates: [_; 6] = self.conjugates().collect::<Vec<_>>().try_into().unwrap();

        let mut isotopy_classes = conjugates.map(|c| c.isotopy_class_lookup(lookup));

        isotopy_classes.sort();

        let mut unique = 1;
        let mut prev = &isotopy_classes[0];

        for i in 1..isotopy_classes.len() {
            if isotopy_classes[i] != *prev {
                unique += 1;
            }
            prev = &isotopy_classes[i];
        }

        unique
    }

    fn get_subsquare(&self, rows: &[usize], cols: &[usize]) -> Vec<Vec<usize>> {
        debug_assert!(rows.len() == cols.len());

        let k = rows.len();

        let mut values = vec![vec![0; k]; k];

        for i in 0..k {
            for (j, col) in cols.iter().enumerate() {
                values[i][j] = self.get(rows[i], *col);
            }
        }

        values
    }

    pub fn num_subsquares(&self, k: usize) -> usize {
        let mut subsquares = 0;
        assert!(N < 16);

        for rows in TupleIteratorDyn::new(N, k) {
            for cols in TupleIteratorDyn::new(N, k) {
                let mut subsquare = self.get_subsquare(&rows, &cols);

                let mut permutation: Vec<_> = subsquare[0].to_vec();

                for i in 0..N {
                    if !permutation.contains(&i) {
                        permutation.push(i);
                    }
                }

                let permutation = PermutationDyn::from_vec(permutation).inverse();

                for row in subsquare.iter_mut() {
                    for val in row.iter_mut() {
                        *val = permutation.apply(*val);
                    }
                }

                let is_subsquare = (0..k).all(|i| {
                    (0..k).map(|j| subsquare[i][j]).collect::<BitSet16>()
                        == BitSet16::all_less_than(k)
                        && (0..k).map(|j| subsquare[j][i]).collect::<BitSet16>()
                            == BitSet16::all_less_than(k)
                });
                if is_subsquare {
                    subsquares += 1;
                }
            }
        }

        subsquares
    }

    pub fn subsquares_bitset(&self, k: usize) -> Vec<BitSet128> {
        let mut subsquares = Vec::new();
        assert!(N < 16);

        for rows in TupleIteratorDyn::new(N, k) {
            for cols in TupleIteratorDyn::new(N, k) {
                let mut subsquare = self.get_subsquare(&rows, &cols);

                let mut permutation: Vec<_> = subsquare[0].to_vec();

                for i in 0..N {
                    if !permutation.contains(&i) {
                        permutation.push(i);
                    }
                }

                let permutation = PermutationDyn::from_vec(permutation).inverse();

                for row in subsquare.iter_mut() {
                    for val in row.iter_mut() {
                        *val = permutation.apply(*val);
                    }
                }

                let is_subsquare = (0..k).all(|i| {
                    (0..k).map(|j| subsquare[i][j]).collect::<BitSet16>()
                        == BitSet16::all_less_than(k)
                        && (0..k).map(|j| subsquare[j][i]).collect::<BitSet16>()
                            == BitSet16::all_less_than(k)
                });
                if is_subsquare {
                    let bitset = rows
                        .iter()
                        .flat_map(|row| cols.iter().map(move |col| row * N + col))
                        .collect();

                    subsquares.push(bitset);
                }
            }
        }

        subsquares
    }

    pub fn mask(&self, mask: BitSet128) -> PartialLatinSquare<N> {
        let mut partial_sq = PartialLatinSquare::empty();

        for i in mask {
            let (i, j) = (i / N, i % N);

            partial_sq.set(i, j, Some(self.get(i, j)));
        }

        partial_sq
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

    pub fn permuted_rcs(&self, permutation: &Permutation<3>) -> Self {
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
        Self::from_rcs(rows, cols, vals)
    }

    pub fn permuted_rows(&self, permutation: &Permutation<N>) -> Self {
        let mut new = *self;
        new.permute_rows(permutation);
        new
    }

    pub fn permute_rows(&mut self, permutation: &Permutation<N>) {
        self.rows = permutation.apply_array(self.rows);
    }

    pub fn permuted_cols(&self, permutation: &Permutation<N>) -> Self {
        let mut new = *self;
        new.permute_cols(permutation);
        new
    }

    pub fn permute_cols(&mut self, permutation: &Permutation<N>) {
        permutation.apply_arrays(&mut self.rows);
    }

    pub fn permute_cols_simd(&mut self, inverse_permutation: &Permutation<N>) {
        use std::simd::Simd;

        assert!(N <= 16);

        let mut permutation_simd = [0; 16];
        permutation_simd[0..N].copy_from_slice(&inverse_permutation.as_array().map(|v| v as u8));
        let permutation = Simd::from_array(permutation_simd);

        for i in 0..N {
            let mut simd = [0; 16];
            simd[0..N].copy_from_slice(&self.rows[i]);
            let simd = Simd::from_array(simd);
            let new_row = simd.swizzle_dyn(permutation);
            self.rows[i].copy_from_slice(&new_row[0..N]);
        }
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
        permutation_simd[0..N].copy_from_slice(&permutation.as_array().map(|v| v as u8));
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
        inverse_col_permutation: &Permutation<N>,
        val_permutation: &Permutation<N>,
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

    pub fn cmp_rows(&self, other: &Self) -> Ordering {
        self.rows.cmp(&other.rows)
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
        assert!(N <= 16);
        for i in 0..N {
            for j in 0..N {
                f.write_char(char::from_digit(self.get(i, j) as u32, 16).unwrap())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidLength { len: usize, expected: usize },
    InvalidChar { index: usize, char: char },
    NotALatinSquare,
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
            Error::NotALatinSquare => write!(f, "The latin square property is not met"),
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
                .to_digit(16)
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
            Err(Error::NotALatinSquare)
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

pub fn n_disjoint_transversals_bitset<const N: usize>(
    transversals: &[BitSet128],
) -> Vec<[BitSet128; N]> {
    let mut transversals_by_start: [[Vec<_>; N]; N] =
        array::from_fn(|_| array::from_fn(|_| Vec::new()));

    for t in transversals {
        let first = t
            .intersect(BitSet128::all_less_than(N))
            .into_iter()
            .next()
            .unwrap();
        let second = t
            .intersect(BitSet128::from_range(N..2 * N))
            .into_iter()
            .next()
            .unwrap()
            - N;
        transversals_by_start[first][second].push(*t);
    }

    let mut disjoint_transversals = Vec::new();

    for i in 0..N {
        for transversal in &transversals_by_start[0][i] {
            let mut disjoint = [BitSet128::empty(); N];
            disjoint[0] = *transversal;

            let second_row_left = transversal
                .complement()
                .intersect(BitSet128::from_range(N..2 * N))
                .shift_right(N);
            let mut indices = vec![(0, second_row_left, *transversal)];

            'i: while !indices.is_empty() {
                let i = indices.len();

                if i == N - 1 {
                    let (_, second_row_left, union) = indices.last().unwrap();

                    let left = union
                        .complement()
                        .intersect(BitSet128::all_less_than(N * N));

                    debug_assert!(second_row_left.len() == 1);
                    let second_row = second_row_left.into_iter().next().unwrap();

                    if transversals_by_start[N - 1][second_row].contains(&left) {
                        disjoint[N - 1] = left;
                        disjoint_transversals.push(disjoint);
                        if disjoint_transversals.len() % 1000 == 0 {
                            dbg!(disjoint_transversals.len());
                        }
                    }
                } else {
                    let (index, second_row_left, union) = indices.last_mut().unwrap();

                    while let Some(second_row) = second_row_left.into_iter().next() {
                        for other in transversals_by_start[i][second_row].iter().skip(*index) {
                            *index += 1;

                            if union.is_disjoint(*other) {
                                disjoint[i] = *other;

                                let union = union.union(*other);

                                let next_second_row_left = union
                                    .complement()
                                    .intersect(BitSet128::from_range(N..2 * N))
                                    .shift_right(N);

                                indices.push((0, next_second_row_left, union));
                                continue 'i;
                            }
                        }
                        *index = 0;
                        second_row_left.pop();
                    }
                }

                indices.pop();
            }
        }
    }

    disjoint_transversals
}

#[cfg(test)]
mod test {

    use crate::cycles::generate_minimize_rows_lookup;

    use super::*;

    #[test]
    fn normalize_main_class() {
        let lookup = generate_minimize_rows_lookup();

        assert_eq!(
            LatinSquare::new([[0, 1, 2, 3], [1, 3, 0, 2], [2, 0, 3, 1], [3, 2, 1, 0]])
                .main_class_lookup(&lookup),
            LatinSquare::new([[0, 1, 2, 3], [1, 0, 3, 2], [2, 3, 1, 0], [3, 2, 0, 1]])
        )
    }
}
