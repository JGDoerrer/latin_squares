use std::mem::MaybeUninit;

pub fn factorial(n: usize) -> usize {
    (2..=n).product()
}

/// A permutation of N elements
#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Permutation<const N: usize>([usize; N]);

impl<const N: usize> Permutation<N> {
    pub const fn identity() -> Self {
        let mut elements = [0; N];

        let mut i = 0;
        while i < N {
            elements[i] = i;
            i += 1;
        }

        Permutation(elements)
    }

    pub fn from_array(elements: [usize; N]) -> Self {
        for i in 0..N {
            debug_assert!(elements.contains(&i));
        }

        Permutation(elements)
    }

    pub fn from_slice(elements: &[u8]) -> Self {
        let mut array = [0; N];

        for i in 0..N {
            array[i] = elements[i] as usize;
        }

        Permutation::from_array(array)
    }

    pub fn into_array(self) -> [usize; N] {
        self.0
    }

    pub fn as_array(&self) -> &[usize; N] {
        &self.0
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.0.swap(i, j);
    }

    pub fn inverse(self) -> Self {
        let mut inverse = Self::identity().into_array();

        for i in 0..N {
            inverse[self.0[i]] = i;
        }

        Self::from_array(inverse)
    }

    pub fn order(&self) -> usize {
        let mut permutation = self.clone();

        let mut order = 1;
        while permutation != Permutation::identity() {
            permutation.0 = permutation.0.map(|i| self.apply(i));
            order += 1;
        }
        order
    }

    pub fn num_fixed_points(&self) -> usize {
        self.fixed_points().count()
    }

    pub fn fixed_points(&self) -> impl Iterator<Item = usize> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter(|(i, j)| i == *j)
            .map(|(i, _)| i)
    }

    pub fn conjugate_by(&self, other: &Permutation<N>) -> Self {
        other
            .0
            .map(|i| self.apply(i))
            .map(|i| other.clone().inverse().apply(i))
            .into()
    }

    pub fn cycles(&self) -> Vec<Vec<usize>> {
        let mut cycles = Vec::new();
        let mut used = [false; N];

        for start in self.0 {
            if used[start] {
                continue;
            }

            let mut cycle = vec![start];
            let mut current = self.apply(start);

            while current != start {
                used[current] = true;
                cycle.push(current);
                current = self.apply(current);
            }

            cycle.rotate_right(1);
            cycles.push(cycle);
        }

        cycles
    }

    pub fn cycle_lengths(&self) -> Vec<usize> {
        let mut cycles = Vec::with_capacity(N / 2);
        let mut used = [false; N];

        for start in self.0 {
            if used[start] {
                continue;
            }

            used[start] = true;
            let mut cycle_len = 1;
            let mut current = self.0[start];

            while current != start {
                used[current] = true;
                cycle_len += 1;
                current = self.0[current];
            }

            cycles.push(cycle_len);
        }

        cycles
    }

    pub fn cycle_lengths_index(&self) -> usize {
        let mut cycles = [0; N];
        let mut cycle_count = 0;
        let mut used = [false; N];

        for start in self.0 {
            if used[start] {
                continue;
            }

            used[start] = true;
            let mut cycle_len = 1;
            let mut current = self.0[start];

            while current != start {
                used[current] = true;
                cycle_len += 1;
                current = self.0[current];
            }

            cycles[cycle_count] = cycle_len;
            cycle_count += 1
        }

        cycles[0..cycle_count].sort();

        CYCLE_STRUCTURES[N]
            .iter()
            .position(|c| c == &&cycles[0..cycle_count])
            .unwrap()
    }

    #[inline]
    pub fn apply(&self, num: usize) -> usize {
        self.0[num]
    }

    #[inline]
    pub fn apply_u8(&self, num: u8) -> u8 {
        self.0[num as usize] as u8
    }

    /// permutes the values of the array
    pub fn apply_array<T>(&self, array: [T; N]) -> [T; N]
    where
        T: Copy,
    {
        let permutation = self.0;

        let mut new_array = [MaybeUninit::uninit(); N];

        for (i, p) in permutation.into_iter().enumerate() {
            new_array[p].write(array[i]);
        }

        new_array.map(|i| unsafe { i.assume_init() })
    }

    /// permutes the values of each array
    pub fn apply_arrays<T>(&self, arrays: &mut [[T; N]]) {
        let mut permutation = self.0;

        while let Some((a, &b)) = permutation.iter().enumerate().find(|(a, b)| *a != **b) {
            permutation.swap(a, b);
            for array in arrays.iter_mut() {
                array.swap(a, b);
            }
        }
    }
}

impl<const N: usize> From<[usize; N]> for Permutation<N> {
    fn from(value: [usize; N]) -> Self {
        Permutation::from_array(value)
    }
}

/// An iterater that generates all permutations
pub struct PermutationIter<const N: usize> {
    indices: [usize; N],
    left: usize,
}

impl<const N: usize> PermutationIter<N> {
    pub fn new() -> Self {
        let mut indices = [0; N];

        for (i, index) in indices.iter_mut().enumerate() {
            *index = N - i - 1;
        }

        PermutationIter {
            indices,
            left: factorial(N),
        }
    }
}

impl<const N: usize> Iterator for PermutationIter<N> {
    type Item = Permutation<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sorted = 1;

        for i in (0..N - 1).rev() {
            if self.indices[i] > self.indices[i + 1] {
                sorted += 1;
            } else {
                break;
            }
        }

        if sorted != N {
            let mut next_unsorted = N - sorted - 1;

            for i in (N - sorted - 1..N).rev() {
                if self.indices[i] > self.indices[N - sorted - 1] {
                    next_unsorted = i;
                    break;
                }
            }

            self.indices.swap(next_unsorted, N - sorted - 1);
        }

        self.indices[N - sorted..N].reverse();

        if self.left == 0 {
            None
        } else {
            self.left -= 1;
            Some(Permutation(self.indices))
        }
    }
}

impl<const N: usize> ExactSizeIterator for PermutationIter<N> {
    fn len(&self) -> usize {
        self.left
    }
}

/// A permutation of elements
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PermutationDyn(Vec<usize>);

impl PermutationDyn {
    pub fn identity(n: usize) -> Self {
        let mut elements = vec![0; n];
        for (i, element) in elements.iter_mut().enumerate() {
            *element = i;
        }
        PermutationDyn(elements)
    }

    pub fn from_array<const N: usize>(elements: [usize; N]) -> Self {
        for i in 0..N {
            debug_assert!(elements.contains(&i));
        }

        PermutationDyn(elements.to_vec())
    }

    pub fn from_vec(elements: Vec<usize>) -> Self {
        for i in 0..elements.len() {
            debug_assert!(elements.contains(&i));
        }

        PermutationDyn(elements)
    }

    pub fn into_vec(self) -> Vec<usize> {
        self.0
    }

    pub fn as_vec(&self) -> &Vec<usize> {
        &self.0
    }

    pub fn pad_with_id<const N: usize>(&self) -> Permutation<N> {
        if self.0.len() == N {
            self.into()
        } else if self.0.len() < N {
            let mut new = Permutation::identity();
            for i in 0..self.0.len() {
                new.0[i] = self.0[i];
            }
            new
        } else {
            todo!()
        }
    }

    pub fn inverse(self) -> Self {
        let mut identity = Self::identity(self.0.len()).0;
        let len = self.0.len();
        let mut permutation = self.0;

        for i in 0..len {
            if permutation[i] == i {
                continue;
            }

            let pos_i = permutation.iter().position(|e| *e == i).unwrap();

            identity.swap(i, pos_i);
            permutation.swap(i, pos_i);
        }

        Self::from_vec(identity)
    }

    pub fn apply(&self, num: usize) -> usize {
        self.0[num]
    }

    pub fn apply_vec<T>(&self, array: Vec<T>) -> Vec<T>
    where
        T: Copy,
    {
        let permutation = self.as_vec();

        let mut new_array = vec![MaybeUninit::uninit(); self.0.len()];

        for (i, p) in permutation.iter().enumerate() {
            new_array[*p].write(array[i]);
        }

        new_array
            .into_iter()
            .map(|i| unsafe { i.assume_init() })
            .collect()
    }
}

impl<const N: usize> From<&PermutationDyn> for Permutation<N> {
    fn from(value: &PermutationDyn) -> Self {
        debug_assert!(value.0.len() == N);
        let mut vals = [0; N];
        vals.copy_from_slice(&value.0);
        Permutation(vals)
    }
}

#[derive(Debug, Clone)]

pub struct PermutationDynIter {
    indices: Vec<usize>,
    left: usize,
    n: usize,
}

impl PermutationDynIter {
    pub fn new(n: usize) -> Self {
        let mut indices = vec![0; n];

        for (i, index) in indices.iter_mut().enumerate() {
            *index = n - i - 1;
        }

        PermutationDynIter {
            n,
            indices,
            left: factorial(n),
        }
    }
}

impl Iterator for PermutationDynIter {
    type Item = PermutationDyn;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sorted = 1;

        for i in (0..self.n.saturating_sub(1)).rev() {
            if self.indices[i] > self.indices[i + 1] {
                sorted += 1;
            } else {
                break;
            }
        }

        if sorted != self.n {
            let mut next_unsorted = self.n - sorted - 1;

            for i in (self.n - sorted - 1..self.n).rev() {
                if self.indices[i] > self.indices[self.n - sorted - 1] {
                    next_unsorted = i;
                    break;
                }
            }

            self.indices.swap(next_unsorted, self.n - sorted - 1);
        }

        self.indices[self.n - sorted..self.n].reverse();

        if self.left == 0 {
            None
        } else {
            self.left -= 1;
            Some(PermutationDyn(self.indices.clone()))
        }
    }
}

impl ExactSizeIterator for PermutationDynIter {
    fn len(&self) -> usize {
        self.left
    }
}

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

pub const CYCLE_STRUCTURES: [&[&[usize]]; 11] = [
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

    #[test]
    fn inverse_test() {
        let permutation = Permutation::from_array([3, 1, 4, 2, 0]);
        let inverse = permutation.inverse();
        assert_eq!(inverse.into_array(), [4, 1, 3, 0, 2]);
    }

    #[test]
    fn permutation_iter_test() {
        let mut iter = PermutationIter::<3>::new();

        assert_eq!(iter.next(), Some(Permutation([0, 1, 2])));
        assert_eq!(iter.next(), Some(Permutation([0, 2, 1])));
        assert_eq!(iter.next(), Some(Permutation([1, 0, 2])));
        assert_eq!(iter.next(), Some(Permutation([1, 2, 0])));
        assert_eq!(iter.next(), Some(Permutation([2, 0, 1])));
        assert_eq!(iter.next(), Some(Permutation([2, 1, 0])));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn order_test() {
        assert_eq!(Permutation::from_array([1, 0, 3, 2]).order(), 2);
        assert_eq!(Permutation::from_array([1, 2, 0, 3]).order(), 3);
        assert_eq!(Permutation::from_array([1, 2, 3, 0]).order(), 4);
    }

    #[test]
    fn cycle_test() {
        assert_eq!(
            Permutation::from_array([1, 2, 3, 0]).cycles(),
            vec![vec![0, 1, 2, 3]]
        );
        assert_eq!(
            Permutation::from_array([1, 0, 3, 2, 4]).cycles(),
            vec![vec![0, 1], vec![2, 3], vec![4]]
        );
        assert_eq!(
            Permutation::from_array([3, 0, 1, 2]).cycles(),
            vec![vec![0, 3, 2, 1]]
        );
    }

    #[test]
    fn single_iter() {
        let mut iter = PermutationDynIter::new(1);
        assert_eq!(iter.next(), Some(PermutationDyn::from_array([0])));
        assert_eq!(iter.next(), None);
    }
}
