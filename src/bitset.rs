use std::{fmt::Debug, ops::Range};

macro_rules! make_bitset {
    (
        $BitType:ty,
        $Name:ident,
        $IterName:ident
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $Name {
            bits: $BitType,
        }

        impl $Name {
            #[inline]
            pub const fn empty() -> Self {
                $Name { bits: 0 }
            }

            #[inline]
            pub const fn full() -> Self {
                Self::all_less_than(<$BitType>::BITS as usize)
            }

            #[inline]
            pub const fn all_less_than(n: usize) -> Self {
                $Name {
                    bits: (((1 as $BitType) << n) - 1) as $BitType,
                }
            }

            #[inline]
            pub const fn from_bits(bits: $BitType) -> Self {
                $Name { bits }
            }

            #[inline]
            pub const fn from_range(range: Range<usize>) -> Self {
                let start = range.start;
                let end = range.end;

                Self::all_less_than(end).intersect(Self::all_less_than(start).complement())
            }

            #[inline]
            pub fn from_iter<T>(iter: T) -> Self
            where
                T: IntoIterator<Item = usize>,
            {
                let mut bitset = $Name::empty();
                for item in iter {
                    bitset.insert(item);
                }
                bitset
            }

            #[inline]
            pub const fn from_slice(slice: &[usize]) -> Self {
                let mut bits = 0 as $BitType;
                let mut i = 0;
                while i < slice.len() {
                    bits |= (1 as $BitType) << slice[i];
                    i += 1;
                }
                $Name::from_bits(bits)
            }

            #[inline]
            pub const fn bits(&self) -> $BitType {
                self.bits
            }

            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.bits == 0
            }

            #[inline]
            pub const fn single(i: usize) -> Self {
                $Name { bits: 1 << i }
            }

            #[inline]
            pub fn insert(&mut self, index: usize) {
                debug_assert!(index < <$BitType>::BITS as usize);
                let bit_mask = 1 << index;

                self.bits |= bit_mask;
            }

            #[inline]
            pub fn remove(&mut self, index: usize) {
                debug_assert!(index < <$BitType>::BITS as usize);
                let bit_mask = 1 << index;

                self.bits &= !bit_mask;
            }

            #[inline]
            pub const fn contains(&self, index: usize) -> bool {
                debug_assert!(index < <$BitType>::BITS as usize);
                let bit_mask = 1 << index;

                (self.bits & bit_mask) != 0
            }

            #[inline]
            pub const fn union(&self, other: Self) -> Self {
                $Name {
                    bits: self.bits | other.bits,
                }
            }

            #[inline]
            pub const fn intersect(&self, other: Self) -> Self {
                $Name {
                    bits: self.bits & other.bits,
                }
            }

            #[inline]
            pub const fn complement(&self) -> Self {
                $Name { bits: !self.bits }
            }

            #[inline]
            pub const fn is_disjoint(&self, other: Self) -> bool {
                self.intersect(other).is_empty()
            }

            #[inline]
            pub const fn is_subset_of(&self, other: Self) -> bool {
                self.bits & other.bits == self.bits
            }

            #[inline]
            pub const fn len(&self) -> usize {
                self.bits.count_ones() as usize
            }

            #[inline]
            pub const fn is_single(&self) -> bool {
                self.bits != 0 && self.bits & (self.bits - 1) == 0
                // self.bits.is_power_of_two()
            }
        }

        impl IntoIterator for $Name {
            type IntoIter = $IterName;
            type Item = usize;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                $IterName { bitset: self }
            }
        }

        impl From<$BitType> for $Name {
            #[inline]
            fn from(bits: $BitType) -> Self {
                Self::from_bits(bits)
            }
        }

        #[derive(Debug)]
        pub struct $IterName {
            bitset: $Name,
        }

        impl Iterator for $IterName {
            type Item = usize;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let next = self.bitset.bits.trailing_zeros() as usize;

                if next < <$BitType>::BITS as usize {
                    // remove first set bit
                    self.bitset.bits = (self.bitset.bits - 1) & self.bitset.bits;
                    Some(next)
                } else {
                    None
                }
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                for _ in 0..n {
                    self.bitset.bits = (self.bitset.bits - 1) & self.bitset.bits;
                }

                self.next()
            }
        }

        impl ExactSizeIterator for $IterName {
            #[inline]
            fn len(&self) -> usize {
                self.bitset.len()
            }
        }

        impl FromIterator<usize> for $Name {
            fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
                $Name::from_iter(iter)
            }
        }

        impl Debug for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("BitSet")
                    .field(
                        "bits",
                        &format!("{1:00$b}", <$BitType>::BITS as usize, self.bits)
                            .chars()
                            .collect::<String>(),
                    )
                    // .field(
                    //     "set_bits",
                    //     &(0..<$BitType>::BITS as usize)
                    //         .filter(|i| self.contains(*i))
                    //         .collect::<Vec<_>>(),
                    // )
                    .finish()
            }
        }
    };
}

make_bitset!(u128, BitSet128, BitSet128Iter);
make_bitset!(u64, BitSet64, BitSet64Iter);
make_bitset!(u16, BitSet16, BitSet16Iter);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct BitSet192 {
    words: [u64; 3],
}

impl BitSet192 {
    #[inline]
    pub const fn empty() -> Self {
        BitSet192 { words: [0; 3] }
    }

    #[inline]
    pub fn full() -> Self {
        Self::all_less_than(192)
    }

    #[inline]
    pub fn all_less_than(n: usize) -> Self {
        Self::from_iter(0..n)
    }

    #[inline]
    pub const fn from_bits(bits: [u64; 3]) -> Self {
        BitSet192 { words: bits }
    }

    #[inline]
    pub fn from_range(range: Range<usize>) -> Self {
        let start = range.start;
        let end = range.end;

        Self::all_less_than(end).intersect(Self::all_less_than(start).complement())
    }

    #[inline]
    pub fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = usize>,
    {
        let mut bitset = BitSet192::empty();
        for item in iter {
            bitset.insert(item);
        }
        bitset
    }

    // #[inline]
    // pub const fn from_slice(slice: &[usize]) -> Self {
    //     let mut bits = 0 as $BitType;
    //     let mut i = 0;
    //     while i < slice.len() {
    //         bits |= (1 as $BitType) << slice[i];
    //         i += 1;
    //     }
    //     BitSet192::from_bits(bits)
    // }

    #[inline]
    pub const fn bits(&self) -> [u64; 3] {
        self.words
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|word| *word == 0)
    }

    #[inline]
    pub const fn single(_i: usize) -> Self {
        todo!()
    }

    #[inline]
    pub fn insert(&mut self, index: usize) {
        debug_assert!(index < 192);
        let word = index / 64;
        let bit = index % 64;
        let bit_mask = 1 << bit;

        self.words[word] |= bit_mask;
    }

    #[inline]
    pub fn remove(&mut self, index: usize) {
        debug_assert!(index < 192);
        let word = index / 64;
        let bit = index % 64;
        let bit_mask = 1 << bit;

        self.words[word] &= !bit_mask;
    }

    #[inline]
    pub const fn contains(&self, index: usize) -> bool {
        debug_assert!(index < 192);
        let word = index / 64;
        let bit = index % 64;
        let bit_mask = 1 << bit;

        (self.words[word] & bit_mask) != 0
    }

    #[inline]
    pub const fn union(&self, other: Self) -> Self {
        let mut words = [0; 3];
        let mut i = 0;

        while i < 3 {
            words[i] = self.words[i] | other.words[i];
            i += 1;
        }

        BitSet192 { words }
    }

    #[inline]
    pub const fn intersect(&self, other: Self) -> Self {
        let mut words = [0; 3];
        let mut i = 0;

        while i < 3 {
            words[i] = self.words[i] & other.words[i];
            i += 1;
        }

        BitSet192 { words }
    }

    #[inline]
    pub fn complement(&self) -> Self {
        BitSet192 {
            words: self.words.map(|word| !word),
        }
    }

    #[inline]
    pub fn is_disjoint(&self, other: Self) -> bool {
        self.intersect(other).is_empty()
    }

    #[inline]
    pub fn is_subset_of(&self, other: Self) -> bool {
        self.intersect(other) == *self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.words
            .map(|word| word.count_ones() as usize)
            .into_iter()
            .sum()
    }

    #[inline]
    pub fn is_single(&self) -> bool {
        self.len() == 1
    }
}

impl IntoIterator for BitSet192 {
    type IntoIter = BitSet192Iter;
    type Item = usize;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitSet192Iter { bitset: self }
    }
}

impl From<[u64; 3]> for BitSet192 {
    #[inline]
    fn from(bits: [u64; 3]) -> Self {
        Self::from_bits(bits)
    }
}

pub struct BitSet192Iter {
    bitset: BitSet192,
}

impl Iterator for BitSet192Iter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = 0;

        for i in 0..3 {
            let word = &mut self.bitset.words[i];
            if *word == 0 {
                next += 64;
            } else {
                next += word.trailing_zeros() as usize;
                *word = (*word - 1) & *word;
                return Some(next);
            }
        }

        None
    }
}

impl ExactSizeIterator for BitSet192Iter {
    #[inline]
    fn len(&self) -> usize {
        self.bitset.len()
    }
}

impl FromIterator<usize> for BitSet192 {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        BitSet192::from_iter(iter)
    }
}
