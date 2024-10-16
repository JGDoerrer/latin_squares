/// A permutation of N elements
#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct PermutationSimd(u8, [u8; 16]);

impl PermutationSimd {
    pub fn from_slice(elements: &[u8]) -> Self {
        let mut array = [0; 16];
        let n = elements.len();

        for i in 0..n {
            debug_assert!(elements.contains(&(i as u8)));
            array[i] = elements[i];
        }

        PermutationSimd::from_array(n as u8, array)
    }

    pub fn from_array(n: u8, elements: [u8; 16]) -> Self {
        for i in 0..n {
            debug_assert!(elements.contains(&i));
        }

        PermutationSimd(n, elements)
    }

    pub fn as_array(&self) -> &[u8; 16] {
        &self.1
    }

    pub fn into_array(self) -> [u8; 16] {
        self.1
    }

    #[inline]
    pub fn apply(&self, num: u8) -> u8 {
        debug_assert!(num < self.0);
        self.1[num as usize]
    }
}
