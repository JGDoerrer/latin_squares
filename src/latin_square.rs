use std::{
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

#[derive(Debug, Clone, Copy)]
pub struct Cell(pub usize, pub usize);

impl<const N: usize> LatinSquare<N> {
    pub fn new(values: [[u8; N]; N]) -> Self {
        debug_assert!(Self::is_valid(&values));

        LatinSquare { rows: values }
    }

    pub fn get(&self, row: usize, col: usize) -> usize {
        self.rows[row][col].into()
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

    pub fn num_transversals(&self) -> usize {
        self.transversals_bitset().len()
    }

    pub fn transversals(&self) -> impl Iterator<Item = [u8; N]> + '_ {
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
                    .unwrap() as u8;
            }

            Some(cols)
        })
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
                    unused_vals.remove(val.into());

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

        for t in self.transversals() {
            transversals_by_start[t[0] as usize].push(t);
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

    pub fn full_disjoint_transversals(&self) -> impl Iterator<Item = [[u8; N]; N]> {
        let mut transversals_by_start = [(); N].map(|_| Vec::new());

        for t in self.transversals() {
            transversals_by_start[t[0] as usize].push(t);
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

    pub fn full_disjoint_transversals_bitset(&self) -> impl Iterator<Item = [BitSet128; N]> {
        let mut transversals_by_start = [(); N].map(|_| Vec::new());

        let transversals = self.transversals_bitset();

        for t in transversals {
            let first = t
                .intersect(BitSet128::all_less_than(N))
                .into_iter()
                .next()
                .unwrap();
            transversals_by_start[first].push(t);
        }

        transversals_by_start[0]
            .clone()
            .into_iter()
            .flat_map(move |transversal| {
                let mut disjoint = [BitSet128::empty(); N];
                disjoint[0] = transversal;
                let mut all = vec![];

                let mut indices = vec![0];

                'i: while !indices.is_empty() {
                    let i = indices.len();

                    if i == N - 1 {
                        let left = disjoint[0..N - 1]
                            .iter()
                            .cloned()
                            .reduce(|a, b| a.union(b))
                            .unwrap()
                            .complement()
                            .intersect(BitSet128::all_less_than(N * N));

                        if disjoint.iter().take(N - 1).all(|t| left.is_disjoint(*t))
                            && transversals_by_start[N - 1].contains(&left)
                        {
                            disjoint[N - 1] = left;
                            all.push(disjoint);
                        }
                    } else {
                        let index = indices.last_mut().unwrap();

                        for other in transversals_by_start[i].iter().skip(*index) {
                            *index += 1;

                            let is_disjoint =
                                disjoint.iter().take(i).all(|t| other.is_disjoint(*t));

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
                    }

                    indices.pop();
                }

                all
            })
    }

    pub fn orthogonal_squares(&self) -> impl Iterator<Item = LatinSquare<N>> + '_ {
        self.full_disjoint_transversals_bitset()
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
        let transversals: Vec<_> = self.full_disjoint_transversals_bitset().collect();

        let mut indices = vec![0];
        let mut current_mols = Vec::new();
        let mut mols = Vec::new();

        'i: while let Some(index) = indices.last_mut() {
            for t in transversals.iter().skip(*index) {
                *index += 1;

                if current_mols.iter().all(|transversal: &[BitSet128; N]| {
                    for other in transversal {
                        for t in t {
                            if !other.intersect(*t).is_single() {
                                return false;
                            }
                        }
                    }
                    true
                }) {
                    current_mols.push(*t);

                    let new_mols = Mols::new(
                        [*self]
                            .into_iter()
                            .chain(current_mols.iter().map(Self::bitset_transversals_to_sq))
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
                    if let Some(new_mols) = new_mols.normalize_main_class_set_sq(lookup, self) {
                        if !mols.contains(&new_mols) {
                            mols.push(new_mols);
                        }
                    }
                    // let new_mols = new_mols.normalize_main_class_set(lookup);
                    // if !mols.contains(&new_mols) {
                    //     mols.push(new_mols);
                    // }

                    let next_index = *index;
                    indices.push(next_index);

                    continue 'i;
                }
            }

            current_mols.pop();
            indices.pop();
        }

        mols
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

    pub fn isotopy_class_permutation(&self) -> (Self, [Permutation<N>; 3]) {
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
        let (isotopy_class, _) = self.isotopy_class_permutation();

        let mut symmetries = Vec::new();
        for (sq, permutation) in
            PermutationIter::new().map(|permutation| (self.permuted_rcs(&permutation), permutation))
        {
            if sq.isotopy_class_permutation().0 == isotopy_class {
                symmetries.push(permutation);
            }
        }
        symmetries
    }

    pub fn main_class(&self) -> Self {
        let sq = self;

        let mut main_class = *sq;
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

                let mut cycles = row_permutation.cycle_lengths();
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

            for rows in candidates {
                let permutations = minimize_rows(&rows);

                for (s, c) in permutations {
                    let mut new_sq = sq;
                    new_sq.permute_cols_vals_simd(&c, &s);
                    new_sq.rows.sort();

                    if new_sq.cmp_rows(&main_class).is_lt() {
                        main_class = new_sq;
                    }
                }
            }
        }

        main_class
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

    fn get_subsquare<const K: usize>(&self, rows: &[usize; K], cols: &[usize; K]) -> [[u8; K]; K] {
        assert!(K <= N);

        let mut values = [[0; K]; K];

        for i in 0..K {
            for (j, col) in cols.iter().enumerate() {
                values[i][j] = self.rows[rows[i]][*col];
            }
        }

        values
    }

    fn subsquares<const K: usize>(&self) -> Vec<([usize; K], [usize; K])> {
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

    fn get_subsquare_dyn(&self, rows: &[usize], cols: &[usize]) -> Vec<Vec<usize>> {
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

    pub fn num_subsquares_dyn(&self, k: usize) -> usize {
        let mut subsquares = 0;
        assert!(N < 16);

        for rows in TupleIteratorDyn::new(N, k) {
            for cols in TupleIteratorDyn::new(N, k) {
                let mut subsquare = self.get_subsquare_dyn(&rows, &cols);

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
