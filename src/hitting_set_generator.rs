use crate::{
    bitset::BitSet128,
    bitvec::BitVec,
    latin_square::{Cell, LatinSquare, PartialLatinSquare},
};

#[derive(Debug)]
pub struct HittingSetGenerator<const N: usize> {
    stack: Vec<StackEntry>,
    sq: LatinSquare<N>,
    unavoidable_sets: Vec<Vec<BitSet128>>,
    max_entries: usize,
}

#[derive(Debug)]
struct StackEntry {
    index: usize,
    current_set: BitSet128,
    used_sets: BitVec,
    hitting_set: BitSet128,
    dead: BitSet128,
}

impl<const N: usize> HittingSetGenerator<N> {
    pub fn new(
        sq: LatinSquare<N>,
        unavoidable_sets: Vec<Vec<BitSet128>>,
        max_entries: usize,
    ) -> Self {
        let mut used_sets = BitVec::empty();
        used_sets.insert(0);

        HittingSetGenerator {
            stack: vec![StackEntry {
                index: 0,
                current_set: unavoidable_sets[0][0],
                used_sets,
                hitting_set: BitSet128::empty(),
                dead: BitSet128::empty(),
            }],
            unavoidable_sets,
            sq,
            max_entries,
        }
    }

    fn get_partial_sq(&self, hitting_set: BitSet128) -> PartialLatinSquare<N> {
        let mut partial_sq = PartialLatinSquare::new();

        for i in hitting_set {
            let Cell(i, j) = Cell::from_index::<N>(i);

            partial_sq.set(i, j, Some(self.sq.get(i, j)));
        }

        partial_sq
    }
}

impl<const N: usize> Iterator for HittingSetGenerator<N> {
    type Item = PartialLatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        while let Some(entry) = self.stack.last_mut() {
            let StackEntry {
                current_set,
                used_sets,
                index,
                hitting_set,
                dead,
            } = entry;

            let Some(next_entry) = current_set.into_iter().nth(*index) else {
                self.stack.pop();
                continue;
            };

            *index += 1;
            let n = *index;

            if dead.contains(next_entry) {
                continue;
            }

            let mut new_hitting_set = *hitting_set;
            new_hitting_set.insert(next_entry);
            // if entries_left == 1
            //     && self
            //         .unavoidable_sets_2
            //         .iter()
            //         .any(|set| set.intersect(new_hitting_set).is_empty())
            // {
            //     continue;
            // }
            // if entries_left == 2
            //     && self
            //         .unavoidable_sets_3
            //         .iter()
            //         .any(|set| set.intersect(new_hitting_set).is_empty())
            // {
            //     continue;
            // }

            let Some(next_unused_set) = used_sets.first_zero() else {
                let hitting_set = *hitting_set;
                self.stack.pop();
                return Some(self.get_partial_sq(hitting_set));
            };

            let entries_left = new_hitting_set.len() - self.max_entries;
            if entries_left == 0 {
                continue;
            }
            if self
                .unavoidable_sets
                .get(entries_left)
                .map(|vec| vec.iter().any(|set| set.is_disjoint(*hitting_set)))
                .is_some_and(|b| b)
            {
                continue;
            }

            let dead = current_set.into_iter().take(n).collect();
            let current_set = self.unavoidable_sets[0][next_unused_set];

            let mut new_used_sets = used_sets.clone();
            new_used_sets.insert(next_unused_set);

            self.stack.push(StackEntry {
                current_set,
                hitting_set: new_hitting_set,
                used_sets: new_used_sets,
                dead,
                index: 0,
            });
        }

        None
    }
}
