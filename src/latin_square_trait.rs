pub trait LatinSquareTrait: PartialLatinSquareTrait {
    fn get(&self, row: usize, col: usize) -> usize {
        self.get_partial(row, col).unwrap()
    }
}

pub trait PartialLatinSquareTrait {
    fn n(&self) -> usize;

    fn get_partial(&self, row: usize, col: usize) -> Option<usize>;
}
