use std::collections::HashSet;

use crate::{
    constraints::Constraints, latin_square::LatinSquare, partial_latin_square::PartialLatinSquare,
    permutation::Permutation, tuple_iterator::TupleIterator,
};

pub struct MainClassGenerator<'a, const N: usize> {
    cycle_generator: CycleGenerator<N>,
    generator: Option<SqGenerator<'a, N>>,
    sqs: HashSet<LatinSquare<N>>,
    lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
}

impl<'a, const N: usize> MainClassGenerator<'a, N> {
    pub fn new(lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>) -> Self {
        MainClassGenerator {
            cycle_generator: CycleGenerator::new(),
            generator: None,
            sqs: HashSet::new(),
            lookup,
        }
    }

    fn get_next_sq(&mut self) -> Option<LatinSquare<N>> {
        if let Some(generator) = &mut self.generator {
            for sq in generator.by_ref() {
                if sq
                    .col_cycles()
                    .into_iter()
                    .any(|c| !self.cycle_generator.cycles().contains(&c))
                    || sq
                        .val_cycles()
                        .into_iter()
                        .any(|c| !self.cycle_generator.cycles().contains(&c))
                {
                    continue;
                }

                let main_class = sq.main_class_lookup(&self.lookup);

                if self.sqs.insert(main_class) {
                    return Some(main_class);
                }
            }
        }
        None
    }
}

impl<'a, const N: usize> Iterator for MainClassGenerator<'a, N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.get_next_sq() {
            return Some(value);
        }

        let Some(partial_sq) = self.cycle_generator.next() else {
            return None;
        };

        dbg!(partial_sq);

        self.generator = Some(SqGenerator::new(
            partial_sq,
            self.cycle_generator.cycles().to_vec(),
            &self.lookup,
        ));
        self.next()
    }
}

struct CycleGenerator<const N: usize> {
    cycle_structures: Vec<Vec<usize>>,
    row_cycle_index: usize,
}

impl<const N: usize> CycleGenerator<N> {
    fn new() -> Self {
        let cycle_structures = generate_cycle_structures(N);
        CycleGenerator {
            cycle_structures,
            row_cycle_index: 0,
        }
    }

    fn cycles(&self) -> &[Vec<usize>] {
        &self.cycle_structures[self.row_cycle_index - 1..]
    }
}

impl<const N: usize> Iterator for CycleGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row_cycle_index >= self.cycle_structures.len() {
            return None;
        }

        let row_cycle = &self.cycle_structures[self.row_cycle_index];

        self.row_cycle_index += 1;

        let mut sq = PartialLatinSquare::<N>::empty();

        for i in 0..N {
            sq.set(0, i, Some(i));
        }

        let mut index = 0;
        for cycle in row_cycle {
            let start_index = index;
            index += cycle;
            for j in 0..*cycle {
                sq.set(1, start_index + j, Some(start_index + (j + 1) % cycle));
            }
        }

        Some(sq)
    }
}

struct SqGenerator<'a, const N: usize> {
    row_generators: Vec<RowGenerator<'a, N>>,
    cycles: Vec<Vec<usize>>,
    all_cycles: bool,
    lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
}

impl<'a, const N: usize> SqGenerator<'a, N> {
    fn new(
        sq: PartialLatinSquare<N>,
        cycles: Vec<Vec<usize>>,
        lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
    ) -> Self {
        SqGenerator {
            row_generators: vec![RowGenerator::new(&sq, 2, lookup)],
            all_cycles: cycles == generate_cycle_structures(N),
            cycles,
            lookup,
        }
    }
}

impl<'a, const N: usize> Iterator for SqGenerator<'a, N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row_generators.is_empty() {
            return None;
        }

        'r: while let Some(generator) = self.row_generators.last_mut() {
            let Some(sq) = generator.next() else {
                self.row_generators.pop();
                continue;
            };

            if sq.is_complete() {
                return Some(sq.try_into().unwrap());
            }

            let new_row_index = (0..N)
                .find(|row| sq.get_row(*row).iter().all(|v| v.is_none()))
                .unwrap();

            let full_rows: Vec<_> = (0..N)
                .filter(|row| sq.get_row(*row).iter().all(|v| v.is_some()))
                .collect();

            if !self.all_cycles {
                for rows in TupleIterator::<2>::new(full_rows.len())
                    .flat_map(|[row0, row1]| [[row0, row1], [row1, row0]])
                {
                    let rows = rows.map(|i| sq.get_row(full_rows[i]).map(|v| v.unwrap()));

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

                    if !self.cycles.contains(&cycles) {
                        continue 'r;
                    }
                }
            }

            self.row_generators
                .push(RowGenerator::new(&sq, new_row_index, self.lookup));
        }

        None
    }
}

struct RowGenerator<'a, const N: usize> {
    constraints: Constraints<N>,
    indices: [usize; N],
    row: usize,
    sqs: HashSet<PartialLatinSquare<N>>,
    lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
}

impl<'a, const N: usize> RowGenerator<'a, N> {
    fn new(
        sq: &PartialLatinSquare<N>,
        row: usize,
        lookup: &'a Vec<Vec<(Permutation<N>, Permutation<N>)>>,
    ) -> Self {
        let mut constraints = Constraints::new_partial(sq);
        constraints.set(
            row,
            0,
            constraints
                .get_possibilities(row, 0)
                .into_iter()
                .next()
                .unwrap(),
        );

        RowGenerator {
            constraints,
            row,
            indices: [0; N],
            sqs: HashSet::new(),
            lookup,
        }
    }
}

impl<'a, const N: usize> Iterator for RowGenerator<'a, N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.row;
        let mut constraints = self.constraints.clone();

        for i in 1..N {
            let index = self.indices[i];

            let Some(value) = constraints.get_possibilities(row, i).into_iter().nth(index) else {
                if i == 1 {
                    return None;
                } else {
                    self.indices[i - 1] += 1;
                    for i in i..N {
                        self.indices[i] = 0;
                    }
                    return self.next();
                }
            };

            constraints.set(row, i, value);
        }
        self.indices[N - 1] += 1;

        if !constraints.is_solvable() {
            return None;
        }

        let sq = constraints.partial_sq().minimize_rows(&self.lookup);
        if self.sqs.insert(sq) {
            Some(sq)
        } else {
            self.next()
        }
    }
}

pub fn generate_cycle_structures(n: usize) -> Vec<Vec<usize>> {
    let mut cycles = Vec::new();
    cycles.push(vec![n]);

    for i in 2..=n / 2 {
        let left = n - i;

        for mut cycle in generate_cycle_structures(left) {
            cycle.push(i);
            cycle.sort();
            cycles.push(cycle);
        }
    }

    cycles.sort();
    cycles.dedup();
    cycles
}

pub const CYCLE_STRUCTURES: [&[&[usize]]; 10] = [
    &[&[0]],
    &[&[1]],
    &[&[2]],
    &[&[3]],
    &[&[2, 2], &[4]],
    &[&[2, 3], &[5]],
    &[&[2, 2, 2], &[2, 4], &[3, 3], &[6]],
    &[&[2, 2, 3], &[2, 5], &[3, 4], &[7]],
    &[
        &[2, 2, 2, 2],
        &[2, 2, 4],
        &[2, 3, 3],
        &[2, 6],
        &[3, 5],
        &[4, 4],
        &[8],
    ],
    &[
        &[2, 2, 2, 3],
        &[2, 2, 5],
        &[2, 3, 4],
        &[2, 7],
        &[3, 3, 3],
        &[3, 6],
        &[4, 5],
        &[9],
    ],
];

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn cycle_structures() {
        assert_eq!(generate_cycle_structures(3), vec![vec![3]]);
        assert_eq!(generate_cycle_structures(4), vec![vec![2, 2], vec![4]]);
        assert_eq!(generate_cycle_structures(5), vec![vec![2, 3], vec![5]]);
        assert_eq!(
            generate_cycle_structures(6),
            vec![vec![2, 2, 2], vec![2, 4], vec![3, 3], vec![6]]
        );
        assert_eq!(
            generate_cycle_structures(7),
            vec![vec![2, 2, 3], vec![2, 5], vec![3, 4], vec![7]]
        );
        assert_eq!(
            generate_cycle_structures(8),
            vec![
                vec![2, 2, 2, 2],
                vec![2, 2, 4],
                vec![2, 3, 3],
                vec![2, 6],
                vec![3, 5],
                vec![4, 4],
                vec![8]
            ]
        );
    }

    #[test]
    fn check_table() {
        for i in 0..CYCLE_STRUCTURES.len() {
            assert_eq!(generate_cycle_structures(i), CYCLE_STRUCTURES[i]);
        }
    }
}
