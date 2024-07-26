pub trait Permutable<T> {
    /// Returns an iterator over all permutations of the items, withot duplicates
    /// ```
    /// let items = vec![0, 1, 2];
    /// let mut permutations = items.permutations();
    ///
    /// assert_eq!(permutations.next(), Some(vec![0, 1, 2]));
    /// assert_eq!(permutations.next(), Some(vec![0, 2, 1]));
    /// assert_eq!(permutations.next(), Some(vec![1, 0, 2]));
    /// assert_eq!(permutations.next(), Some(vec![1, 2, 0]));
    /// assert_eq!(permutations.next(), Some(vec![2, 0, 1]));
    /// assert_eq!(permutations.next(), Some(vec![2, 1, 0]));
    /// assert_eq!(permutations.next(), None);
    /// ```
    fn permutations(&self) -> Permutations<T>;
}

impl<T> Permutable<T> for Vec<T>
where
    T: Clone,
{
    fn permutations(&self) -> Permutations<T> {
        Permutations::new(self.clone())
    }
}

pub struct Permutations<T> {
    is_first: bool,
    start: Vec<T>,
    items: Vec<T>,
    n: usize,
}

impl<T> Permutations<T>
where
    T: Clone,
{
    fn new(items: Vec<T>) -> Self {
        let n = items.len();
        Permutations {
            is_first: true,
            n,
            start: items.clone(),
            items,
        }
    }
}

impl<T> Iterator for Permutations<T>
where
    T: Clone + PartialOrd,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let old_items = self.items.clone();
        let mut sorted = 1;

        for i in (0..self.n.saturating_sub(1)).rev() {
            if self.items[i] >= self.items[i + 1] {
                sorted += 1;
            } else {
                break;
            }
        }

        if sorted != self.n {
            let mut next_unsorted = self.n - sorted - 1;

            for i in (self.n - sorted - 1..self.n).rev() {
                if self.items[i] > self.items[self.n - sorted - 1] {
                    next_unsorted = i;
                    break;
                }
            }

            self.items.swap(next_unsorted, self.n - sorted - 1);
        }

        self.items[self.n - sorted..self.n].reverse();

        if old_items == self.start && !self.is_first {
            None
        } else {
            self.is_first = false;
            Some(old_items)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn duplicates() {
        let items = vec![0, 1, 1];
        let mut permutations = items.permutations();

        assert_eq!(permutations.next(), Some(vec![0, 1, 1]));
        assert_eq!(permutations.next(), Some(vec![1, 0, 1]));
        assert_eq!(permutations.next(), Some(vec![1, 1, 0]));
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn unordered_start() {
        let items = vec![2, 0, 1];
        let mut permutations = items.permutations();

        assert_eq!(permutations.next(), Some(vec![2, 0, 1]));
        assert_eq!(permutations.next(), Some(vec![2, 1, 0]));
        assert_eq!(permutations.next(), Some(vec![0, 1, 2]));
        assert_eq!(permutations.next(), Some(vec![0, 2, 1]));
        assert_eq!(permutations.next(), Some(vec![1, 0, 2]));
        assert_eq!(permutations.next(), Some(vec![1, 2, 0]));
        assert_eq!(permutations.next(), None);
    }
}
