use std::mem::MaybeUninit;

use crate::bitset::BitSet16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Permutation<const N: usize>([usize; N]);

pub fn factorial(n: usize) -> usize {
    (2..=n).product()
}

impl<const N: usize> Permutation<N> {
    // pub const fn identity() -> Self {
    //     Self::from_rank(0)
    // }

    // pub const fn from_rank(rank: u32) -> Self {
    //     Self(rank)
    // }

    // pub fn to_rank(self) -> u32 {
    //     self.0
    // }

    // pub fn from_array(elements: [usize; N]) -> Self {
    //     debug_assert!(N <= 12);
    //     for i in 0..N {
    //         debug_assert!(elements.contains(&i));
    //     }

    //     let len = elements.len();
    //     let mut elements_left: Vec<_> = (0..len).collect();

    //     let mut rank = 0;

    //     for i in 0..len {
    //         let element = elements[i];
    //         let index = elements_left.iter().position(|e| *e == element).unwrap();
    //         elements_left.remove(index);
    //         rank += index * factorial(len - i - 1);
    //     }

    //     Permutation(rank as u32)
    // }

    // pub fn to_array(self) -> [usize; N] {
    //     let mut permutation = [0; N];
    //     let mut elements_left = [None; N];

    //     for i in 0..N {
    //         elements_left[i] = Some(i);
    //     }

    //     let mut rank = self.0 as usize;

    //     for k in 0..N {
    //         let fac = factorial(N - k - 1);
    //         let d = rank / fac;
    //         permutation[k] = elements_left
    //             .iter_mut()
    //             .filter(|i| i.is_some())
    //             .nth(d)
    //             .unwrap()
    //             .take()
    //             .unwrap();
    //         rank %= fac;
    //     }

    //     permutation
    // }
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

    pub fn to_rank(self) -> usize {
        let mut elements_left = BitSet16::all_less_than(N);

        let mut rank = 0;

        for i in 0..N {
            let element = self.0[i];
            let index = elements_left
                .into_iter()
                .position(|e| e == element)
                .unwrap();
            elements_left.remove(element);
            rank += index * factorial(N - i - 1);
        }

        rank
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

    pub fn apply(self, num: usize) -> usize {
        let permutation = self.to_array();
        permutation[num]
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
}
