use std::mem::MaybeUninit;

pub fn factorial(n: usize) -> usize {
    (2..=n).product()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Permutation<const N: usize>([usize; N]);

impl<const N: usize> Permutation<N> {
    pub fn identity() -> Self {
        let mut elements = [0; N];
        for i in 0..N {
            elements[i] = i;
        }
        Permutation(elements)
    }

    pub fn from_array(elements: [usize; N]) -> Self {
        for i in 0..N {
            debug_assert!(elements.contains(&i));
        }

        Permutation(elements)
    }

    pub fn to_array(self) -> [usize; N] {
        self.0
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.0.swap(i, j);
    }

    pub fn inverse(self) -> Self {
        let mut identity = Self::identity().to_array();
        let mut permutation = self.to_array();

        for i in 0..N {
            if permutation[i] == i {
                continue;
            }

            let pos_i = permutation.iter().position(|e| *e == i).unwrap();

            identity.swap(i, pos_i);
            permutation.swap(i, pos_i);
        }

        let inverse = Self::from_array(identity);

        inverse
    }

    pub fn order(&self) -> usize {
        let mut permutation = *self;

        let mut order = 1;
        while permutation != Permutation::identity() {
            permutation.0 = permutation.0.map(|i| self.apply(i));
            order += 1;
        }
        order
    }

    pub fn apply(self, num: usize) -> usize {
        self.0[num]
    }

    pub fn apply_array<T>(self, array: [T; N]) -> [T; N]
    where
        T: Copy,
    {
        let permutation = self.to_array();

        let mut new_array = [MaybeUninit::uninit(); N];

        for (i, p) in permutation.into_iter().enumerate() {
            new_array[p].write(array[i]);
        }

        new_array.map(|i| unsafe { i.assume_init() })
    }
}

impl<const N: usize> From<[usize; N]> for Permutation<N> {
    fn from(value: [usize; N]) -> Self {
        Permutation::from_array(value)
    }
}

pub struct PermutationIter<const N: usize> {
    indices: [usize; N],
    left: usize,
}

impl<const N: usize> PermutationIter<N> {
    pub fn new() -> Self {
        let mut indices = [0; N];

        for i in 0..N {
            indices[i] = N - i - 1;
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PermutationDyn(Vec<usize>);

impl PermutationDyn {
    pub fn identity(n: usize) -> Self {
        let mut elements = vec![0; n];
        for i in 0..n {
            elements[i] = i;
        }
        PermutationDyn(elements)
    }

    pub fn from_array<const N: usize>(elements: [usize; N]) -> Self {
        for i in 0..N {
            debug_assert!(elements.contains(&i));
        }

        PermutationDyn(elements.to_vec())
    }

    pub fn to_vec(self) -> Vec<usize> {
        self.0
    }

    pub fn as_vec(&self) -> &Vec<usize> {
        &self.0
    }

    pub fn pad_with_id<const N: usize>(&self) -> Permutation<N> {
        if self.0.len() >= N {
            self.into()
        } else {
            let mut new = Permutation::identity();
            for i in 0..self.0.len() {
                new.0[i] = self.0[i];
            }
            new
        }
    }

    // pub fn inverse(self) -> Self {
    //     let mut identity = Self::identity().to_array();
    //     let mut permutation = self.to_array();

    //     for i in 0..N {
    //         if permutation[i] == i {
    //             continue;
    //         }

    //         let pos_i = permutation.iter().position(|e| *e == i).unwrap();

    //         identity.swap(i, pos_i);
    //         permutation.swap(i, pos_i);
    //     }

    //     let inverse = Self::from_array(identity);

    //     inverse
    // }

    pub fn apply(self, num: usize) -> usize {
        self.0[num]
    }

    // pub fn apply_array<T>(self, array: [T; N]) -> [T; N]
    // where
    //     T: Copy,
    // {
    //     let permutation = self.to_array();

    //     let mut new_array = [MaybeUninit::uninit(); N];

    //     for (i, p) in permutation.into_iter().enumerate() {
    //         new_array[p].write(array[i]);
    //     }

    //     new_array.map(|i| unsafe { i.assume_init() })
    // }
}

impl<const N: usize> From<&PermutationDyn> for Permutation<N> {
    fn from(value: &PermutationDyn) -> Self {
        assert!(value.0.len() == N);
        let mut vals = [0; N];
        for i in 0..N {
            vals[i] = value.0[i];
        }
        Permutation(vals)
    }
}

pub struct PermutationDynIter {
    indices: Vec<usize>,
    left: usize,
    n: usize,
}

impl PermutationDynIter {
    pub fn new(n: usize) -> Self {
        let mut indices = vec![0; n];

        for i in 0..n {
            indices[i] = n - i - 1;
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

        for i in (0..self.n - 1).rev() {
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn inverse_test() {
        let permutation = Permutation::from_array([3, 1, 4, 2, 0]);
        let inverse = permutation.inverse();
        assert_eq!(inverse.to_array(), [4, 1, 3, 0, 2]);
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
}
