#[derive(Debug, Clone)]
pub struct BitVec {
    words: Vec<usize>,
}

impl BitVec {
    #[inline]
    pub fn empty() -> Self {
        BitVec { words: Vec::new() }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        BitVec {
            words: Vec::with_capacity(capacity.div_ceil(usize::BITS as usize)),
        }
    }

    #[inline]
    pub fn all_less_than(n: usize) -> Self {
        Self::from_iter(0..n)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|word| *word == 0)
    }

    #[inline]
    pub fn insert(&mut self, index: usize) {
        let word = index / usize::BITS as usize;
        let bit = index % usize::BITS as usize;
        let bit_mask = 1 << bit;

        if self.words.len() <= word {
            self.words.resize(word + 1, 0);
        }

        self.words[word] |= bit_mask;
    }

    #[inline]
    pub fn remove(&mut self, index: usize) {
        let word = index / usize::BITS as usize;
        let bit = index % usize::BITS as usize;
        let bit_mask = 1 << bit;

        if self.words.len() <= word {
            return;
        }

        self.words[word] &= !bit_mask;
    }

    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        let word = index / usize::BITS as usize;
        let bit = index % usize::BITS as usize;
        let bit_mask = 1 << bit;

        if self.words.len() <= word {
            false
        } else {
            (self.words[word] & bit_mask) != 0
        }
    }

    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        let new_len = self.words.len().max(other.words.len());
        let mut words = Vec::with_capacity(new_len);

        for i in 0..new_len {
            words.push(self.words[i] | other.words[i]);
        }

        BitVec { words }
    }

    #[inline]
    pub fn intersect(&self, other: &Self) -> Self {
        let new_len = self.words.len().min(other.words.len());
        let mut words = Vec::with_capacity(new_len);

        for i in 0..new_len {
            words.push(self.words[i] & other.words[i]);
        }

        BitVec { words }
    }

    #[inline]
    pub fn minus(&self, other: &Self) -> Self {
        let new_len = self.words.len();
        let mut words = Vec::with_capacity(new_len);

        for i in 0..new_len {
            words.push(self.words[i] & !other.words.get(i).unwrap_or(&0));
        }

        BitVec { words }
    }

    #[inline]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.words
            .iter()
            .zip(other.words.iter())
            .all(|(a, b)| a & b == 0)
    }

    #[inline]
    pub fn is_subset_of(&self, other: Self) -> bool {
        self.words
            .iter()
            .zip(other.words.iter())
            .all(|(a, b)| a & b == *a)
    }

    #[inline]
    pub fn count_ones(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .into_iter()
            .sum()
    }

    pub fn first_one(&self) -> Option<usize> {
        let index = self.words.iter().position(|word| *word != 0)?;

        Some(self.words[index].trailing_zeros() as usize + index * usize::BITS as usize)
    }

    pub fn first_zero(&self) -> Option<usize> {
        let index = self.words.iter().position(|word| *word != usize::MAX)?;

        Some(self.words[index].trailing_ones() as usize + index * usize::BITS as usize)
    }
}

impl FromIterator<usize> for BitVec {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut new = Self::empty();
        for item in iter {
            new.insert(item);
        }
        new
    }
}
