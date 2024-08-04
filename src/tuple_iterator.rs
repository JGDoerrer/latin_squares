pub struct TupleIterator<const K: usize> {
    n: usize,
    current: Option<[usize; K]>,
}

impl<const K: usize> TupleIterator<K> {
    pub fn new(n: usize) -> Self {
        if n >= K {
            let mut first = [0; K];

            for (i, element) in first.iter_mut().enumerate() {
                *element = i;
            }

            TupleIterator {
                n,
                current: Some(first),
            }
        } else {
            TupleIterator { n, current: None }
        }
    }
}

impl<const K: usize> Iterator for TupleIterator<K> {
    type Item = [usize; K];

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.as_mut()?;

        let prev = *current;

        if current.first().is_some_and(|v| *v == self.n - K) {
            self.current = None;
        } else {
            for i in (0..K).rev() {
                if current[i] < self.n + i - K {
                    current[i] += 1;
                    for j in i + 1..K {
                        current[j] = current[i] + j - i;
                    }

                    break;
                }
            }
        }

        Some(prev)
    }
}

pub struct TupleIteratorDyn {
    n: usize,
    k: usize,
    current: Option<Box<[usize]>>,
}

impl TupleIteratorDyn {
    pub fn new(n: usize, k: usize) -> Self {
        assert!(n >= k);
        let mut first = vec![0; k].into_boxed_slice();

        for i in 0..k {
            first[i] = i;
        }

        TupleIteratorDyn {
            n,
            k,
            current: Some(first),
        }
    }
}

impl Iterator for TupleIteratorDyn {
    type Item = Box<[usize]>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.as_mut()?;

        let prev = current.clone();

        if current.first().is_some_and(|v| *v == self.n - self.k) {
            self.current = None;
        } else {
            for i in (0..self.k).rev() {
                if current[i] < self.n + i - self.k {
                    current[i] += 1;
                    for j in i + 1..self.k {
                        current[j] = current[i] + j - i;
                    }

                    break;
                }
            }
        }

        Some(prev)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_4_2() {
        let mut iter = TupleIterator::new(4);

        assert_eq!(iter.next(), Some([0, 1]));
        assert_eq!(iter.next(), Some([0, 2]));
        assert_eq!(iter.next(), Some([0, 3]));
        assert_eq!(iter.next(), Some([1, 2]));
        assert_eq!(iter.next(), Some([1, 3]));
        assert_eq!(iter.next(), Some([2, 3]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_3_3() {
        let mut iter = TupleIterator::new(3);

        assert_eq!(iter.next(), Some([0, 1, 2]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_3_2() {
        let mut iter = TupleIterator::new(3);

        assert_eq!(iter.next(), Some([0, 1]));
        assert_eq!(iter.next(), Some([0, 2]));
        assert_eq!(iter.next(), Some([1, 2]));
        assert_eq!(iter.next(), None);
    }
}
