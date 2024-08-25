use crate::cycles::CYCLE_STRUCTURES;

/// A permutation of N elements
#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct PermutationSimd(u8, [u8; 16]);

impl PermutationSimd {
    pub fn identity(n: u8) -> Self {
        let mut elements = [0; 16];

        let mut i = 0;
        while i < n {
            elements[i as usize] = i as u8;
            i += 1;
        }

        PermutationSimd(n, elements)
    }

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
            debug_assert!(elements.contains(&(i as u8)));
        }

        PermutationSimd(n, elements)
    }

    pub fn into_array(self) -> [u8; 16] {
        self.1
    }

    pub fn as_array(&self) -> &[u8; 16] {
        &self.1
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.1.swap(i, j);
    }

    pub fn inverse(&self) -> Self {
        let mut inverse = [0; 16];

        for i in 0..self.0 {
            inverse[self.1[i as usize] as usize] = i as u8;
        }

        dbg!(self);
        dbg!(Self::from_array(self.0, inverse))
    }

    pub fn order(&self) -> usize {
        let mut permutation = self.clone();

        let mut order = 1;
        while permutation != PermutationSimd::identity(self.0) {
            permutation.1 = permutation.1.map(|i| self.apply(i));
            order += 1;
        }
        order
    }

    pub fn cycles(&self) -> Vec<Vec<u8>> {
        let mut cycles = Vec::new();
        let mut used = [false; 16];

        for start in &self.1[0..self.0 as usize] {
            if used[*start as usize] {
                continue;
            }

            let mut cycle = vec![*start];
            let mut current = self.apply(*start);

            while current != *start {
                used[current as usize] = true;
                cycle.push(current);
                current = self.apply(current);
            }

            cycle.rotate_right(1);
            cycles.push(cycle);
        }

        cycles
    }

    pub fn cycle_lengths(&self) -> Vec<usize> {
        let mut cycles = Vec::with_capacity(self.0 as usize / 2);
        let mut used = [false; 16];

        for start in &self.1[0..self.0 as usize] {
            if used[*start as usize] {
                continue;
            }

            used[*start as usize] = true;
            let mut cycle_len = 1;
            let mut current = self.1[*start as usize];

            while current != *start {
                used[current as usize] = true;
                cycle_len += 1;
                current = self.1[current as usize];
            }

            cycles.push(cycle_len);
        }

        cycles
    }

    pub fn cycle_lengths_index(&self) -> usize {
        let mut cycles = [0; 16];
        let mut cycle_count = 0;
        let mut used = [false; 16];

        for start in &self.1[0..self.0 as usize] {
            if used[*start as usize] {
                continue;
            }

            used[*start as usize] = true;
            let mut cycle_len = 1;
            let mut current = self.1[*start as usize];

            while current != *start {
                used[current as usize] = true;
                cycle_len += 1;
                current = self.1[current as usize];
            }

            cycles[cycle_count] = cycle_len;
            cycle_count += 1
        }

        cycles[0..cycle_count].sort();

        CYCLE_STRUCTURES[self.0 as usize]
            .iter()
            .position(|c| c == &&cycles[0..cycle_count])
            .unwrap()
    }

    #[inline]
    pub fn apply(&self, num: u8) -> u8 {
        assert!(num < self.0);
        self.1[num as usize]
    }
}
