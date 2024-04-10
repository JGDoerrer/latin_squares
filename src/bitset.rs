use std::{fmt::Debug, ops::Range};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BitSet {
    bits: BitType,
}

type BitType = u128;

impl BitSet {
    #[inline]
    pub const fn empty() -> Self {
        BitSet { bits: 0 }
    }

    #[inline]
    pub const fn full() -> Self {
        Self::all_less_than(BitType::BITS as usize)
    }

    #[inline]
    pub const fn all_less_than(n: usize) -> Self {
        BitSet {
            bits: (((1 as BitType) << n) - 1) as BitType,
        }
    }

    #[inline]
    pub const fn from_bits(bits: BitType) -> Self {
        BitSet { bits }
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
        let mut bitset = BitSet::empty();
        for item in iter {
            bitset.insert(item);
        }
        bitset
    }

    #[inline]
    pub const fn from_slice(slice: &[usize]) -> Self {
        let mut bits = 0 as BitType;
        let mut i = 0;
        while i < slice.len() {
            bits |= (1 as BitType) << slice[i];
            i += 1;
        }
        BitSet::from_bits(bits)
    }

    #[inline]
    pub const fn bits(&self) -> BitType {
        self.bits
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[inline]
    pub const fn single(i: usize) -> Self {
        BitSet { bits: 1 << i }
    }

    #[inline]
    pub fn insert(&mut self, index: usize) {
        debug_assert!(index < BitType::BITS as usize);
        let bit_mask = 1 << index;

        self.bits |= bit_mask;
    }

    #[inline]
    pub fn remove(&mut self, index: usize) {
        debug_assert!(index < BitType::BITS as usize);
        let bit_mask = 1 << index;

        self.bits &= !bit_mask;
    }

    #[inline]
    pub const fn contains(&self, index: usize) -> bool {
        debug_assert!(index < BitType::BITS as usize);
        let bit_mask = 1 << index;

        (self.bits & bit_mask) != 0
    }

    #[inline]
    pub const fn union(&self, other: Self) -> Self {
        BitSet {
            bits: self.bits | other.bits,
        }
    }

    #[inline]
    pub const fn intersect(&self, other: Self) -> Self {
        BitSet {
            bits: self.bits & other.bits,
        }
    }

    #[inline]
    pub const fn complement(&self) -> Self {
        BitSet { bits: !self.bits }
    }

    #[inline]
    pub const fn is_disjoint(&self, other: Self) -> bool {
        self.bits & other.bits == 0
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

impl IntoIterator for BitSet {
    type IntoIter = BitSetIter;
    type Item = usize;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitSetIter { bitset: self }
    }
}

impl From<BitType> for BitSet {
    #[inline]
    fn from(bits: BitType) -> Self {
        Self::from_bits(bits)
    }
}

pub struct BitSetIter {
    bitset: BitSet,
}

impl Iterator for BitSetIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bitset.bits.trailing_zeros() as usize;

        if next < BitType::BITS as usize {
            // remove first set bit
            self.bitset.bits = (self.bitset.bits - 1) & self.bitset.bits;
            Some(next)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for BitSetIter {
    #[inline]
    fn len(&self) -> usize {
        self.bitset.len()
    }
}

impl Debug for BitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitSet")
            .field(
                "bits",
                &format!("{:0128b}", self.bits).chars().collect::<String>(),
            )
            .field(
                "set_bits",
                &(0..BitType::BITS as usize)
                    .filter(|i| self.contains(*i))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
