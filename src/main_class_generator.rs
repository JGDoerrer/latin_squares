use std::collections::HashSet;

use crate::{
    constraints::Constraints, latin_square::LatinSquare,
    latin_square_generator::LatinSquareGenerator, partial_latin_square::PartialLatinSquare,
    permutation::Permutation,
};

pub struct MainClassGenerator<const N: usize> {
    cycle_generator: CycleGenerator<N>,
    generator: Option<LatinSquareGenerator<N>>,
    sqs: HashSet<LatinSquare<N>>,
}

impl<const N: usize> MainClassGenerator<N> {
    pub fn new() -> Self {
        MainClassGenerator {
            cycle_generator: CycleGenerator::new(),
            generator: None,
            sqs: HashSet::new(),
        }
    }

    fn get_next_sq(&mut self) -> Option<LatinSquare<N>> {
        if let Some(generator) = &mut self.generator {
            for sq in generator.by_ref() {
                // dbg!(sq);
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

        let Some(partial_sq) = self.cycle_generator.next() else {
            return None;
        };

        dbg!(partial_sq);

        self.generator = Some(LatinSquareGenerator::from_partial_sq(&partial_sq));
        self.next()
    }
}

struct CycleGenerator<const N: usize> {
    cycle_structures: Vec<Vec<usize>>,
    row_cycle_index: usize,
    // col_cycle_index: usize,
    // col_permutation: PermutationDynIter,
}

impl<const N: usize> CycleGenerator<N> {
    fn new() -> Self {
        let cycle_structures = generate_cycle_structures(N);
        CycleGenerator {
            cycle_structures,
            row_cycle_index: 0,
            // col_cycle_index: 0,
            // col_permutation: PermutationDynIter::new(N - 2),
        }
    }

    // fn is_compatible(row_cycle: &Vec<usize>, col_cycle: &Vec<usize>) -> bool {
    //     if row_cycle.starts_with(&[2]) {
    //         col_cycle.starts_with(&[2])
    //     } else {
    //         true
    //     }
    // }

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
        // let col_cycle = &self.cycle_structures[self.col_cycle_index];
        // let mut col_permutation = self.col_permutation.next();

        // if col_permutation.is_none() {
        //     self.col_cycle_index += 1;
        //     if self.col_cycle_index >= self.cycle_structures.len() {
        //         self.row_cycle_index += 1;
        //         self.col_cycle_index = self.row_cycle_index;
        //     }
        //     self.col_permutation = PermutationDynIter::new(N - 2);
        //     col_permutation = self.col_permutation.next();
        // }
        // let col_permutation = col_permutation.unwrap();

        // if !Self::is_compatible(row_cycle, col_cycle) {
        //     self.col_cycle_index += 1;
        //     if self.col_cycle_index >= self.cycle_structures.len() {
        self.row_cycle_index += 1;
        // self.col_cycle_index = self.row_cycle_index;
        //     }
        //     self.col_permutation = PermutationDynIter::new(N - 2);
        //     return self.next();
        // }

        // dbg!(row_cycle, col_cycle);
        let mut sq = PartialLatinSquare::<N>::empty();

        for i in 0..N {
            sq.set(0, i, Some(i));
            sq.set(i, 0, Some(i));
        }

        let mut index = 0;
        for cycle in row_cycle {
            let start_index = index;
            index += cycle;
            for j in 0..*cycle {
                sq.set(1, start_index + j, Some(start_index + (j + 1) % cycle));
            }
        }

        // let mut col_permutation = col_permutation.into_vec();
        // for v in col_permutation.iter_mut() {
        //     *v += 2;
        // }
        // col_permutation.insert(0, 0);
        // col_permutation.insert(1, 1);
        // // col_permutation.insert(2, 2);
        // let col_permutation = PermutationDyn::from_vec(col_permutation).pad_with_id::<N>();

        // let mut index = 0;
        // for cycle in col_cycle {
        //     let start_index = index;
        //     index += cycle;
        //     for j in 0..*cycle {
        //         let value = start_index + (j + 1) % cycle;
        //         let value = col_permutation.apply(value);
        //         let row = col_permutation.apply(start_index + j);
        //         sq.set(row, 1, Some(value));
        //     }
        // }

        Some(sq)
    }
}

struct RowGenerator<const N: usize> {
    constraints: Constraints<N>,
    indices: [usize; N],
    row: usize,
}

impl<const N: usize> RowGenerator<N> {
    fn new(sq: &PartialLatinSquare<N>, row: usize) -> Self {
        RowGenerator {
            constraints: Constraints::new_partial(sq),
            row,
            indices: [0; N],
        }
    }
}

impl<const N: usize> Iterator for RowGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut constraints = self.constraints.clone();
        if !constraints.is_solvable() {
            return None;
        }

        let row = self.row;

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

        Some(*constraints.partial_sq())
    }
}

struct SqGenerator<const N: usize> {
    row_generators: Vec<RowGenerator<N>>,
    cycles: Vec<Vec<usize>>,
    all_cycles: bool,
}

impl<const N: usize> SqGenerator<N> {
    fn new(sq: PartialLatinSquare<N>, cycles: Vec<Vec<usize>>) -> Self {
        SqGenerator {
            row_generators: vec![RowGenerator::new(&sq, 2)],
            all_cycles: cycles == generate_cycle_structures(N),
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

            let new_row_index = self.row_generators.len() + 1;

            if !self.all_cycles {
                for rows in (0..new_row_index - 1)
                    .flat_map(|i| [[i, new_row_index - 1], [new_row_index - 1, i]])
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
            }

            self.row_generators
                .push(RowGenerator::new(&sq, new_row_index + 1));
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
