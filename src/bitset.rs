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

        #[allow(dead_code)]
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
            pub const fn insert(&mut self, index: usize) {
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

            pub fn print_sq(&self, size: usize) {
                for i in 0..size {
                    println!("+{}", "---+".repeat(size));
                    print!("|");
                    for j in 0..size {
                        if self.contains(i * size + j) {
                            print!(" X |");
                        } else {
                            print!("   |");
                        }
                    }
                    println!()
                }
                println!("+{}", "---+".repeat(size));
                println!()
            }

            pub fn iter(&self) -> $IterName {
                self.into_iter()
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

        #[derive(Debug, Clone)]
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
make_bitset!(u32, BitSet32, BitSet32Iter);
make_bitset!(u16, BitSet16, BitSet16Iter);
