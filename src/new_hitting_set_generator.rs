use std::{time::Instant, vec};

use crate::{
    bitset::{BitSet128, BitSet128Iter},
    bitvec::BitVec,
};

type BitSet = BitSet128;
type BitSetIter = BitSet128Iter;

#[derive(Debug)]
pub struct NewHittingSetGenerator {
    stack: Vec<StackEntry>,
    sets: Vec<BitSet>,
    max_entries: usize,
    entry_to_sets: Vec<BitVec>,
    last_progress: Instant,
}

#[derive(Debug)]
struct StackEntry {
    cand: BitSet,
    hitting_set: BitSet,
    uncovered: BitVec,
    critical: Vec<BitVec>,
    c: BitSetIter,
}

impl NewHittingSetGenerator {
    pub fn new(mut sets: Vec<Vec<BitSet>>, max_entries: usize) -> Self {
        let sets = sets.remove(0);
        let largest_entry = sets
            .iter()
            .map(|set| set.into_iter().last().unwrap())
            .max()
            .unwrap();
        let mut entry_to_set = vec![BitVec::empty(); largest_entry + 1];

        for (i, set) in sets.iter().enumerate() {
            for entry in *set {
                entry_to_set[entry].insert(i);
            }
        }

        let mut cand = BitSet::all_less_than(largest_entry + 1);
        let uncovered = BitVec::all_less_than(sets.len());
        let uncovered_set_index = uncovered
            .iter()
            .min_by_key(|index| sets[*index].intersect(cand).len())
            .unwrap();
        let uncovered_set = &sets[uncovered_set_index];

        let c = uncovered_set.intersect(cand);
        cand = cand.intersect(c.complement());

        let stack = vec![StackEntry {
            hitting_set: BitSet::empty(),
            uncovered,
            critical: vec![BitVec::empty(); largest_entry + 1],
            c: c.into_iter(),
            cand,
        }];

        NewHittingSetGenerator {
            stack,
            entry_to_sets: entry_to_set,
            sets,
            max_entries,
            last_progress: Instant::now(),
        }
    }
}

impl Iterator for NewHittingSetGenerator {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        'w: while let Some(entry) = self.stack.last_mut() {
            let StackEntry {
                hitting_set,
                uncovered,
                critical,
                c,
                cand,
            } = entry;

            if hitting_set.len() >= self.max_entries {
                self.stack.pop();
                continue;
            }

            for v in c {
                let mut new_critical = critical.clone();
                let mut new_uncovered = uncovered.clone();
                let mut new_hitting_set = *hitting_set;
                new_hitting_set.insert(v);

                for f in self.entry_to_sets[v].iter() {
                    for crit in &mut new_critical {
                        crit.remove(f);
                    }

                    if new_uncovered.contains(f) {
                        new_uncovered.remove(f);
                        new_critical[v].insert(f);
                    }
                }

                if hitting_set.into_iter().all(|f| {
                    new_critical[f]
                        .iter()
                        .any(|c| self.sets[c].intersect(new_hitting_set) == BitSet::single(f))
                }) {
                    cand.insert(v);
                    if new_uncovered.is_empty() {
                        let hitting_set = new_hitting_set;
                        return Some(hitting_set);
                    }

                    let uncovered_set_index = new_uncovered
                        .iter()
                        .min_by_key(|index| self.sets[*index].intersect(*cand).len())
                        .unwrap();
                    let uncovered_set = &self.sets[uncovered_set_index];

                    let c = uncovered_set.intersect(*cand);
                    let cand = cand.intersect(c.complement());

                    let new_entry = StackEntry {
                        hitting_set: new_hitting_set,
                        uncovered: new_uncovered,
                        critical: new_critical,
                        c: c.into_iter(),
                        cand,
                    };
                    self.stack.push(new_entry);
                    continue 'w;
                }
            }

            let other_cand = self.stack.pop().unwrap().cand;
            if let Some(cand) = &mut self.stack.last_mut().map(|e| e.cand) {
                *cand = cand.intersect(other_cand);
            }
        }

        None
    }
}
