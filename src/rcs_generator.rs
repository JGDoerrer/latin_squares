use crate::{
    constraints::Constraints, latin_square::LatinSquare, partial_latin_square::PartialLatinSquare,
};

pub struct RCSGenerator<const N: usize> {
    stack: Vec<StackEntry<N>>,
}

#[derive(Debug)]
struct StackEntry<const N: usize> {
    sq: PartialLatinSquare<N>,
    index: usize,
    triples: Vec<[usize; 3]>,
}

impl<const N: usize> RCSGenerator<N> {
    pub fn new() -> Self {
        RCSGenerator {
            stack: vec![StackEntry {
                sq: PartialLatinSquare::empty(),
                index: 0,
                triples: Vec::new(),
            }],
        }
    }
}

impl<const N: usize> Iterator for RCSGenerator<N> {
    type Item = LatinSquare<N>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            let StackEntry { sq, index, triples } = self.stack.last_mut().unwrap();

            if *index >= N.pow(3) {
                self.stack.pop();
                continue;
            }

            let triple = [*index / (N * N), *index / N % N, *index % N];
            *index += 1;

            if triple[0] > triple[1] || triple[0] > triple[2] {
                continue;
            }

            if triples.contains(&triple)
                || triples.contains(&[triple[2], triple[0], triple[1]])
                || triples.contains(&[triple[1], triple[2], triple[0]])
            {
                continue;
            }

            let [row, col, val] = triple;

            let mut new_sq = *sq;
            let constraints = Constraints::new_partial(sq);

            if constraints.get(row, col).contains(val) {
                new_sq.set(row, col, Some(val));
            } else {
                continue;
            }

            if constraints.get(val, row).contains(col) {
                new_sq.set(val, row, Some(col));
            } else if new_sq.get(val, row) != Some(col) {
                continue;
            }

            if constraints.get(col, val).contains(row) {
                new_sq.set(col, val, Some(row));
            } else if new_sq.get(col, val) != Some(row) {
                continue;
            }

            if new_sq.is_complete() {
                return Some(new_sq.into());
            }

            let mut new_partial = Constraints::new_partial(&new_sq);
            new_partial.find_singles();
            if !new_partial.is_solvable() {
                continue;
            }

            // dbg!(new_sq);
            let index = *index;
            let mut triples = triples.clone();
            triples.push(triple);
            self.stack.push(StackEntry {
                sq: new_sq,
                index,
                triples,
            });
        }

        None
    }
}

fn triple_iter<const N: usize>() -> impl Iterator<Item = [usize; 3]> {
    (0..N).flat_map(|first| {
        ((first + 1)..N)
            .flat_map(move |second| ((second + 1)..N).map(move |third| [first, second, third]))
    })
}
