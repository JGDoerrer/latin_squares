use std::vec;

use crate::{
    bitset::{BitSet128, BitSet128Iter},
    bitvec::BitVec,
    latin_square::{Cell, LatinSquare, PartialLatinSquare},
    partial_square_generator::PartialSquareGenerator,
};

#[derive(Debug)]
pub struct HittingSetGenerator<const N: usize> {
    stack: Vec<StackEntry>,
    sq: LatinSquare<N>,
    unavoidable_sets: Vec<Vec<BitSet128>>,
    max_entries: usize,
    entry_to_set: Vec<BitVec>,
    partial_gen: Option<PartialSquareGenerator<N>>,
}

#[derive(Debug)]
struct StackEntry {
    next_dead: BitSet128,
    current_set_iter: BitSet128Iter,
    hitting_set: BitSet128,
    sets_hit: BitVec,
    dead: BitSet128,
}

impl<const N: usize> HittingSetGenerator<N> {
    pub fn new(
        sq: LatinSquare<N>,
        unavoidable_sets: Vec<Vec<BitSet128>>,
        max_entries: usize,
    ) -> Self {
        let mut entry_to_set = vec![BitVec::empty(); N * N];

        for (i, set) in unavoidable_sets[0].iter().enumerate() {
            for entry in *set {
                entry_to_set[entry].insert(i);
            }
        }

        HittingSetGenerator {
            stack: vec![StackEntry {
                current_set_iter: unavoidable_sets[0][0].into_iter(),
                next_dead: BitSet128::empty(),
                hitting_set: BitSet128::empty(),
                dead: BitSet128::empty(),
                sets_hit: BitVec::empty(),
            }],
            entry_to_set,
            unavoidable_sets,
            sq,
            max_entries,
            partial_gen: None,
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
        if let Some(partial_gen) = &mut self.partial_gen {
            let next = partial_gen.next();
            if next.is_none() {
                self.partial_gen = None;
            } else {
                return next;
            }
        }

        if self.stack.is_empty() {
            return None;
        }

        while let Some(entry) = self.stack.last_mut() {
            let StackEntry {
                current_set_iter: current_set,
                hitting_set,
                dead,
                sets_hit,
                next_dead,
            } = entry;

            let Some(next_entry) = current_set.next() else {
                self.stack.pop();
                continue;
            };
            next_dead.insert(next_entry);

            // if dead.contains(next_entry) {
            //     continue;
            // }

            let mut next_hitting_set = hitting_set.clone();
            next_hitting_set.insert(next_entry);

            if next_hitting_set.len() > self.max_entries {
                self.stack.pop();
                continue;
            }

            let next_sets_hit = sets_hit.union(&self.entry_to_set[next_entry]);

            let next_set_index = next_sets_hit.first_zero();

            if next_set_index.is_none()
                || next_set_index.is_some_and(|index| index >= self.unavoidable_sets[0].len())
            {
                // all sets are hit
                let mut sq = PartialLatinSquare::new();

                for index in next_hitting_set {
                    let Cell(i, j) = Cell::from_index::<N>(index);
                    sq.set(i, j, Some(self.sq.get(i, j)));
                }

                if next_hitting_set.len() < self.max_entries {
                    self.partial_gen = Some(PartialSquareGenerator::new_partial(
                        self.sq,
                        sq,
                        self.max_entries,
                    ));
                    return self.partial_gen.as_mut().unwrap().next();
                } else {
                    return Some(sq);
                }
            }

            let entries_left = self.max_entries - next_hitting_set.len();
            if entries_left == 0 {
                continue;
            }

            if entries_left > 1
                && self
                    .unavoidable_sets
                    .get(entries_left)
                    .is_some_and(|sets| sets.iter().any(|set| set.is_disjoint(*hitting_set)))
            {
                continue;
            }

            let next_dead = dead.union(*next_dead);
            let next_set =
                self.unavoidable_sets[0][next_set_index.unwrap()].intersect(next_dead.complement());

            self.stack.push(StackEntry {
                dead: next_dead,
                next_dead: BitSet128::empty(),
                current_set_iter: next_set.into_iter(),
                hitting_set: next_hitting_set,
                sets_hit: next_sets_hit,
            })
        }

        None
    }
}
