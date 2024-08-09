use std::{
    time::{Duration, Instant},
    vec,
};

use crate::{bitset::BitSet128, bitvec::BitVec};

type BitSet = BitSet128;

/// Generates all critical sets for a hitting set problem
#[derive(Debug)]
pub struct HittingSetGenerator {
    stack: Vec<StackEntry>,
    sets: Vec<Vec<BitSet>>,
    max_entries: usize,
    entry_to_set: Vec<BitVec>,
    temp: Option<BitVec>,
    start: Instant,
}

#[derive(Debug)]
struct StackEntry {
    next_dead: BitSet,
    current_set: BitSet,
    current_set_index: usize,
    hitting_set: BitSet,
    sets_hit: BitVec,
    dead: BitSet,
}

impl HittingSetGenerator {
    pub fn new(sets: Vec<Vec<BitSet>>, max_entries: usize) -> Self {
        let largest_entry = sets[0]
            .iter()
            .map(|set| set.into_iter().last().unwrap())
            .max()
            .unwrap();
        let mut entry_to_set = vec![BitVec::empty(); largest_entry + 1];

        for (i, set) in sets[0].iter().enumerate() {
            for entry in *set {
                entry_to_set[entry].insert(i);
            }
        }

        let stack = vec![StackEntry {
            current_set: sets[0][0],
            current_set_index: 0,
            next_dead: BitSet::empty(),
            hitting_set: BitSet::empty(),
            dead: BitSet::empty(),
            sets_hit: BitVec::empty(),
        }];

        HittingSetGenerator {
            stack,
            entry_to_set,
            sets,
            max_entries,
            temp: Some(BitVec::empty()),
            start: Instant::now(),
        }
    }

    fn progress(&self) -> f64 {
        let totals: Vec<_> = self
            .stack
            .iter()
            .map(|entry| entry.current_set.len() as f64)
            .collect();

        self.stack
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                entry.current_set_index.saturating_sub(1) as f64
                    / totals[0..=i]
                        .iter()
                        .copied()
                        .reduce(|a, b| a * b)
                        .unwrap_or(1.0)
            })
            .reduce(|a, b| a + b)
            .unwrap()
    }

    fn estimated_time_left(&self) -> Duration {
        let time_passed = Instant::now() - self.start;
        let progress = self.progress();
        let total_time = time_passed.div_f64(progress);

        total_time - time_passed
    }
}

impl Iterator for HittingSetGenerator {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        let mut last_progress = Instant::now();

        while let Some(entry) = self.stack.last_mut() {
            let StackEntry {
                current_set,
                current_set_index,
                hitting_set,
                dead,
                sets_hit,
                next_dead,
            } = entry;

            let mut entries: Vec<_> = current_set.into_iter().collect();
            entries.sort_by_key(|s| self.entry_to_set[*s].count_ones());
            entries.reverse();

            let Some(next_entry) = entries.get(*current_set_index).copied() else {
                let entry = self.stack.pop().unwrap();
                self.temp = Some(entry.sets_hit);

                let time_passed = (Instant::now() - last_progress).as_secs_f64();
                if time_passed >= 1.0 {
                    dbg!(self.progress(), self.estimated_time_left());

                    last_progress = Instant::now();
                }
                continue;
            };
            next_dead.insert(next_entry);
            *current_set_index += 1;

            // if dead.contains(next_entry) {
            //     continue;
            // }

            let mut next_hitting_set = *hitting_set;
            next_hitting_set.insert(next_entry);

            if next_hitting_set.len() > self.max_entries {
                let entry = self.stack.pop().unwrap();
                self.temp = Some(entry.sets_hit);
                continue;
            }

            let next_sets_hit = self.temp.get_or_insert_with(BitVec::empty);

            sets_hit.union_into(&self.entry_to_set[next_entry], next_sets_hit);

            let next_set_index = next_sets_hit.first_zero();

            if next_set_index.is_none()
                || next_set_index.is_some_and(|index| index >= self.sets[0].len())
            {
                // all sets are hit
                return Some(next_hitting_set);
            }

            let entries_left = self.max_entries - next_hitting_set.len();
            if entries_left == 0 {
                continue;
            }

            if entries_left > 1
                && self
                    .sets
                    .get(entries_left)
                    .is_some_and(|sets| sets.iter().any(|set| set.is_disjoint(*hitting_set)))
            {
                continue;
            }

            let next_dead = dead.union(*next_dead);
            let next_set = self.sets[0][next_set_index.unwrap()].intersect(next_dead.complement());

            self.stack.push(StackEntry {
                dead: next_dead,
                next_dead: BitSet::empty(),
                current_set_index: 0,
                current_set: next_set,
                hitting_set: next_hitting_set,
                sets_hit: self.temp.take().unwrap(),
            });
        }

        None
    }
}
