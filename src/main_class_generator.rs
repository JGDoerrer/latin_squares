use std::collections::HashSet;

use crate::{
    constraints::Constraints, latin_square::LatinSquare,
    latin_square_generator::LatinSquareGenerator, partial_latin_square::PartialLatinSquare,
    permutation::Permutation, tuple_iterator::TupleIterator,
};

pub struct MainClassGenerator<const N: usize> {
    cycle_structures: Vec<Vec<usize>>,
    generator: Option<LatinSquareGenerator<N>>,
    sqs: HashSet<LatinSquare<N>>,
}

impl<const N: usize> MainClassGenerator<N> {
    pub fn new() -> Self {
        let mut cycle_structures = generate_cycle_structures(N);
        cycle_structures.reverse();

        MainClassGenerator {
            cycle_structures,
            generator: None,
            sqs: HashSet::new(),
        }
    }

    fn get_next_sq(&mut self) -> Option<LatinSquare<N>> {
        if let Some(generator) = &mut self.generator {
            while let Some(sq) = generator.next() {
                let main_class = sq.main_class_reduced();

                if self.sqs.insert(main_class) {
                    return Some(main_class);
                }
            }
        }
        None
    }
}

impl<const N: usize> Iterator for MainClassGenerator<N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.get_next_sq() {
            return Some(value);
        }

        if self.cycle_structures.is_empty() {
            return None;
        }

        let cycle_structure = self.cycle_structures.pop().unwrap();

        let mut partial_sq = PartialLatinSquare::<N>::empty();

        for i in 0..N {
            partial_sq.set(0, i, Some(i));
            partial_sq.set(i, 0, Some(i));
        }

        let mut index = 0;
        for cycle in &cycle_structure {
            let start_index = index;
            index += cycle;
            for j in 0..*cycle {
                partial_sq.set(1, start_index + j, Some(start_index + (j + 1) % cycle));
            }
        }

        let mut cycles = self.cycle_structures.clone();
        cycles.push(cycle_structure);
        self.generator = Some(LatinSquareGenerator::from_partial_sq(&partial_sq));
        self.next()
    }
}

struct RowGenerator<const N: usize> {
    sq: PartialLatinSquare<N>,
    indices: [usize; N],
}

impl<const N: usize> RowGenerator<N> {
    fn new(sq: PartialLatinSquare<N>) -> Self {
        RowGenerator {
            sq,
            indices: [0; N],
        }
    }
}

impl<const N: usize> Iterator for RowGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut constraints = Constraints::new_partial(&self.sq);
        let row = (0..N).find(|row| self.sq.get(*row, 1).is_none()).unwrap();

        for i in 1..N {
            let index = self.indices[i];

            let Some(value) = constraints.get_possibilities(row, i).into_iter().nth(index) else {
                if i != 1 {
                    self.indices[i - 1] += 1;
                    for i in i..N {
                        self.indices[i] = 0;
                    }
                    return self.next();
                } else {
                    return None;
                }
            };

            constraints.set(row, i, value);
        }
        self.indices[N - 1] += 1;

        Some(constraints.partial_sq().clone())
    }
}

struct SqGenerator<const N: usize> {
    row_generators: Vec<RowGenerator<N>>,
    cycles: Vec<Vec<usize>>,
}

impl<const N: usize> SqGenerator<N> {
    fn new(sq: PartialLatinSquare<N>, cycles: Vec<Vec<usize>>) -> Self {
        SqGenerator {
            row_generators: vec![RowGenerator::new(sq)],
            cycles,
        }
    }
}

impl<const N: usize> Iterator for SqGenerator<N> {
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

            let completed_rows = (0..N)
                .filter(|row| sq.get_row(*row).iter().all(|v| v.is_some()))
                .count();

            for rows in (0..completed_rows - 1)
                .flat_map(|i| [[i, completed_rows - 1], [completed_rows - 1, i]])
            {
                let rows = rows.map(|i| sq.get_row(i).map(|v| v.unwrap()));

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

                if !self.cycles.contains(&cycles) {
                    continue 'r;
                }
            }

            self.row_generators.push(RowGenerator::new(sq));
        }

        None
    }
}

fn generate_cycle_structures(n: usize) -> Vec<Vec<usize>> {
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
    }
}
