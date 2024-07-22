pub trait LatinSquareTrait: PartialLatinSquareTrait {
    fn get(&self, row: usize, col: usize) -> usize;
}

pub trait PartialLatinSquareTrait {
    fn n(&self) -> usize;

    fn get_partial(&self, row: usize, col: usize) -> Option<usize>;
}

pub trait MOLSTrait: PartialMOLSTrait {
    fn squares(&self) -> &[impl LatinSquareTrait];
}

pub trait PartialMOLSTrait {
    fn n(&self) -> usize;
    fn mols(&self) -> usize;

    fn partial_squares(&self) -> &[impl PartialLatinSquareTrait];
}
