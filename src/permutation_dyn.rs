use std::mem::MaybeUninit;

use crate::permutation::{factorial, Permutation, FACTORIAL};

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

    pub fn from_rank(mut rank: usize, n: usize) -> Self {
        let mut permutation = vec![0; n];
        let mut elements_left = vec![None; n];

        for i in 0..n {
            elements_left[i] = Some(i);
        }

        for k in 0..n {
            let fac = FACTORIAL[n - k - 1];
            let d = rank / fac;
            permutation[k] = elements_left
                .iter_mut()
                .filter(|i| i.is_some())
                .nth(d)
                .unwrap()
                .take()
                .unwrap();
            rank %= fac;
        }

        PermutationDyn(permutation)
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
            let mut new = [0; N];
            for i in 0..self.0.len() {
                new[i] = self.0[i];
            }
            Permutation::from_array(new)
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
        Permutation::from_array(vals)
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn single_iter() {
        let mut iter = PermutationDynIter::new(1);
        assert_eq!(iter.next(), Some(PermutationDyn::from_array([0])));
        assert_eq!(iter.next(), None);
    }
}
