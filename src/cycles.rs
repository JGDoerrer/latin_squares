use crate::{
    permutation::Permutation,
    permutation_dyn::{PermutationDyn, PermutationDynIter},
    permutation_simd::PermutationSimd,
};

/// Generates all possible cycle structures of a permutation with no fixed points
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

pub const CYCLE_STRUCTURES: [&[&[usize]]; 12] = [
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
    &[
        &[2, 2, 2, 2, 2],
        &[2, 2, 2, 4],
        &[2, 2, 3, 3],
        &[2, 2, 6],
        &[2, 3, 5],
        &[2, 4, 4],
        &[2, 8],
        &[3, 3, 4],
        &[3, 7],
        &[4, 6],
        &[5, 5],
        &[10],
    ],
    &[
        &[2, 2, 2, 2, 3],
        &[2, 2, 2, 5],
        &[2, 2, 3, 4],
        &[2, 2, 7],
        &[2, 3, 3, 3],
        &[2, 3, 6],
        &[2, 4, 5],
        &[2, 9],
        &[3, 3, 5],
        &[3, 4, 4],
        &[3, 8],
        &[4, 7],
        &[5, 6],
        &[11],
    ],
];

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

        let inverse_column_permutation =
            Permutation::from_array(self.rows[0].map(|i| symbol_permutation.apply(i.into())))
                .inverse();

        Some((symbol_permutation, inverse_column_permutation))
    }
}

pub type PermutationSimdLookup = Vec<Vec<(PermutationSimd, PermutationSimd)>>;

pub fn generate_minimize_rows_lookup_simd<const N: usize>() -> PermutationSimdLookup {
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
            let mut permutations: Vec<_> = cycle_permutations
                .map(|(s, c)| (s.into_simd(), c.into_simd()))
                .collect();

            permutations.sort_unstable();
            permutations.dedup();
            permutations.shrink_to_fit();

            permutations
        })
        .collect()
}

pub type PermutationLookup<const N: usize> = Vec<Vec<(Permutation<N>, Permutation<N>)>>;

pub fn generate_minimize_rows_lookup<const N: usize>() -> PermutationLookup<N> {
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

    // lookup
    let cycle_index = CYCLE_STRUCTURES[N]
        .iter()
        .position(|c| c == &cycle_lengths)
        .unwrap();

    let permutations = &lookup[cycle_index];

    // fix lookup by (s,c)
    let symbol_permutation = symbol_permutation.inverse();

    let permutations = permutations.iter().map(move |(s, c)| {
        (
            Permutation::from_array(symbol_permutation.apply_array(s.clone().into_array())),
            Permutation::from_array(column_permutation.apply_array(c.clone().into_array())),
        )
    });

    Box::new(permutations)
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
