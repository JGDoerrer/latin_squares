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
    stack_index: usize,
    sets: Vec<BitSet>,
    max_entries: usize,
    entry_to_sets: Vec<BitVec>,
    last_progress: Instant,
    temp_entry: StackEntry,
}

#[derive(Debug, Clone)]
struct StackEntry {
    cand: BitSet,
    hitting_set: BitSet,
    uncovered: BitVec,
    critical: Vec<BitVec>,
    c: BitSetIter,
    c_set: BitSet,
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

        let mut stack = vec![
            StackEntry {
                hitting_set: BitSet::empty(),
                uncovered: BitVec::with_capacity(sets.len()),
                cand: BitSet::empty(),
                critical: vec![BitVec::with_capacity(sets.len()); largest_entry + 1],
                c: BitSet::empty().iter(),
                c_set: BitSet::empty(),
            };
            max_entries + 1
        ];
        stack[0] = StackEntry {
            hitting_set: BitSet::empty(),
            uncovered,
            critical: vec![BitVec::with_capacity(sets.len()); largest_entry + 1],
            c_set: c,
            c: c.into_iter(),
            cand,
        };

        NewHittingSetGenerator {
            stack,
            stack_index: 0,
            entry_to_sets: entry_to_set,
            max_entries,
            last_progress: Instant::now(),
            temp_entry: StackEntry {
                hitting_set: BitSet::empty(),
                uncovered: BitVec::with_capacity(sets.len()),
                cand: BitSet::empty(),
                critical: vec![BitVec::with_capacity(sets.len()); largest_entry + 1],
                c: BitSet::empty().iter(),
                c_set: BitSet::empty(),
            },
            sets,
        }
    }

    fn progress(&self) -> f64 {
        let totals: Vec<_> = self.stack[0..self.stack_index]
            .iter()
            .map(|entry| entry.c_set.len() as f64)
            .collect();

        self.stack[0..self.stack_index]
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                (totals[i] - entry.c.clone().count() as f64)
                    / totals[0..=i].iter().copied().product::<f64>()
            })
            .sum::<f64>()
    }
}

impl Iterator for NewHittingSetGenerator {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }

        'w: while let Some(entry) = self.stack.get_mut(self.stack_index) {
            let StackEntry {
                hitting_set,
                uncovered,
                critical,
                c,
                cand,
                ..
            } = entry;

            for v in c {
                let next_entry = &mut self.temp_entry;
                next_entry.critical.clone_from(critical);
                next_entry.uncovered.clone_from(uncovered);
                next_entry.hitting_set.clone_from(hitting_set);
                next_entry.hitting_set.insert(v);

                for f in self.entry_to_sets[v].iter() {
                    for crit in &mut next_entry.critical {
                        if !crit.is_empty() {
                            crit.remove(f);
                        }
                    }

                    if next_entry.uncovered.contains(f) {
                        next_entry.uncovered.remove(f);
                        next_entry.critical[v].insert(f);
                    }
                }

                if hitting_set.into_iter().all(|f| {
                    next_entry.critical[f].iter().any(|c| {
                        self.sets[c].intersect(next_entry.hitting_set) == BitSet::single(f)
                    })
                }) {
                    cand.insert(v);
                    if next_entry.uncovered.is_empty() {
                        let hitting_set = next_entry.hitting_set;

                        let time_passed = (Instant::now() - self.last_progress).as_secs_f64();
                        if time_passed >= 1.0 {
                            self.last_progress = Instant::now();
                            dbg!(self.progress());
                        }
                        return Some(hitting_set);
                    }
                    if hitting_set.len() + 1 >= self.max_entries {
                        continue;
                    }

                    let uncovered_set_index = next_entry
                        .uncovered
                        .iter()
                        .min_by_key(|index| self.sets[*index].intersect(*cand).len())
                        .unwrap();
                    let uncovered_set = &self.sets[uncovered_set_index];

                    let c = uncovered_set.intersect(*cand);
                    next_entry.cand = cand.intersect(c.complement());
                    next_entry.c = c.into_iter();
                    next_entry.c_set = c;

                    self.stack_index += 1;
                    std::mem::swap(&mut self.stack[self.stack_index], &mut self.temp_entry);

                    continue 'w;
                }
            }

            let other_cand = self.stack[self.stack_index].cand;
            if self.stack_index > 0 {
                self.stack_index -= 1;
            } else {
                self.stack.clear();
            }

            if let Some(cand) = &mut self.stack.get_mut(self.stack_index).map(|e| e.cand) {
                *cand = cand.intersect(other_cand);
            }
        }

        None
    }
}
