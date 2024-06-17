use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Clone)]
pub enum Mode {
    PrettyPrint,
    NormalizeParatopy,
    GenerateParatopyClasses,
    FindSCS,
    Testing,
    GenerateLatinSquares,
}
