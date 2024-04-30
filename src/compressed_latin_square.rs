use crate::latin_square::LatinSquare;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CompressedLatinSquare<const N: usize> {
    pub values: [u32; N], // u32 works up to 12
}

impl<const N: usize> From<LatinSquare<N>> for CompressedLatinSquare<N> {
    fn from(sq: LatinSquare<N>) -> Self {
        let mut ranks = [0; N];
        for i in 0..N {
            let row = sq.values[i];

            let rank = rank_of_permutation(&row);
            ranks[i] = rank;
        }
        CompressedLatinSquare { values: ranks }
    }
}

impl<const N: usize> From<CompressedLatinSquare<N>> for LatinSquare<N> {
    fn from(sq: CompressedLatinSquare<N>) -> Self {
        let mut values = [[0; N]; N];
        for i in 0..N {
            let rank = sq.values[i];

            let permutation = permutation_from_rank(rank);
            values[i] = permutation;
        }
        LatinSquare { values }
    }
}

fn rank_of_permutation(elements: &[u8]) -> u32 {
    assert!(elements.len() <= 12);

    let len = elements.len();
    let mut elements_left: Vec<_> = (0..len as u8).collect();

    let mut rank = 0;

    for i in 0..len {
        let element = elements[i];
        let index = elements_left.iter().position(|e| *e == element).unwrap();
        elements_left.remove(index);
        rank += index * factorial(len - i - 1);
    }

    rank as u32
}

fn permutation_from_rank<const N: usize>(mut rank: u32) -> [u8; N] {
    let mut permutation = [0; N];
    let mut elements_left: Vec<_> = (0..N as u8).collect();

    for k in 0..N {
        let fac = factorial(N - k - 1);
        let d = rank as usize / fac;
        permutation[k] = elements_left[d];
        elements_left.remove(d);
        rank %= fac as u32;
    }

    permutation
}

fn factorial(n: usize) -> usize {
    (2..=n).product()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(rank_of_permutation(&[0, 1, 2, 3, 4]), 0);
        assert_eq!(
            rank_of_permutation(&[4, 3, 2, 1, 0]),
            factorial(5) as u32 - 1
        );
        assert_eq!(
            permutation_from_rank(rank_of_permutation(&[0, 3, 1, 4, 2])),
            [0, 3, 1, 4, 2]
        )
    }
}
