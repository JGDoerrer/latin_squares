use crate::{bitset::BitSet16, permutation::PermutationDyn, tuple_iterator::TupleIteratorDyn};

pub trait LatinSquareTrait: PartialLatinSquareTrait {
    fn get(&self, row: usize, col: usize) -> usize;

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

    fn num_subsquares_dyn(&self, k: usize) -> usize {
        let mut subsquares = 0;
        let n = self.n();
        assert!(n < 16);

        for rows in TupleIteratorDyn::new(n, k) {
            for cols in TupleIteratorDyn::new(n, k) {
                let mut subsquare = self.get_subsquare_dyn(&rows, &cols);

                let mut permutation: Vec<_> = subsquare[0].iter().map(|i| *i as usize).collect();

                for i in 0..n {
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
}

pub trait PartialLatinSquareTrait {
    fn n(&self) -> usize;

    fn get_partial(&self, row: usize, col: usize) -> Option<usize>;
}
