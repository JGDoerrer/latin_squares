use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use clap::{self, Parser};



use pairs5::LATIN_PAIRS_5;

use crate::{
    squares5::LATIN_SQUARES_5,
};

mod bitset;
mod compressed_latin_square;
mod constraints;
mod latin_square;
mod latin_square_generator;
mod latin_square_oa_generator;
mod latin_square_pair_generator;
mod latin_square_triple_generator;
mod orthogonal_array;
mod orthogonal_generator;
mod pair_constraints;
mod pairs5;
mod squares5;
mod triple_constraints;

#[derive(Parser)]
struct Args {}

fn main() {
    let _args = Args::parse();

    // LatinSquareOAGenerator::load("0,3,0,0,1,1,2,0,0,2,0,1,1,3,0,1,1,1,1,1,1,1,0,1,1,1".to_string())
    //     .unwrap()
    // LatinSquareOAGenerator::new().for_each(|pair| println!("{pair:?}"));

    // generate_5_graph();
    // generate_7_graph();

    // LatinSquarePairGenerator::<N>::new().for_each(|pair| {
    //     println!("{pair:?}");

    // if let Some(third) = OrthogonalLatinSquareGenerator::new(vec![sq1, sq2]).next() {
    //     println!("{third:?}");
    // } else {
    //     println!("nope");
    // }
    // });

    // let mut threads = vec![];
    // for i in 0..thread::available_parallelism()
    //     .unwrap_or(NonZeroUsize::new(1).unwrap())
    //     .get()
    // threads.push(thread::spawn(move || {
    // let triple = LatinSquareTripleGenerator::<N>::load()
    //     .unwrap_or(LatinSquareTripleGenerator::new())
    //     .next();
    // let triple = LatinSquareTripleGenerator::<N>::new().next();
    // let sq = LatinSquareOAGenerator::new().next();
    // let sq = LatinSquareGenerator::<9>::new().next().unwrap();
    // println!("{sq:?}");

    // let triple = LatinSquareOAGenerator::load(std::env::args().nth(1).unwrap_or(String::new()))
    //     .unwrap_or(LatinSquareOAGenerator::new())
    //     .next();
    // println!("{triple:?}");
    // }));
    // }

    // threads
    //     .into_iter()
    //     .for_each(|thread| thread.join().unwrap());
}

fn generate_5_graph() {
    // let all_pairs: Vec<_> = LatinSquareOAGenerator::new().collect();

    let indices: Vec<_> = LATIN_PAIRS_5
        .iter()
        .map(|sqs| {
            (
                LATIN_SQUARES_5
                    .iter()
                    .position(|sq| sq.is_isotopic_to(&sqs[0]))
                    .unwrap(),
                LATIN_SQUARES_5
                    .iter()
                    .position(|sq| sq.is_isotopic_to(&sqs[1]))
                    .unwrap(),
            )
        })
        .collect();

    let mut connected_parts: Vec<Vec<_>> = Vec::new();

    for (i, j) in indices {
        if let Some(part) = connected_parts.iter_mut().find(|part| {
            part.iter()
                .find(|(a, b)| *a == i || *a == j || *b == i || *b == j)
                .is_some()
        }) {
            part.push((i, j));
        } else {
            connected_parts.push(vec![(i, j)]);
        }
    }

    for (part_index, part) in connected_parts.into_iter().enumerate() {
        let mut nodes: Vec<_> = part.iter().map(|(i, j)| vec![*i, *j]).flatten().collect();
        nodes.sort();
        nodes.dedup();

        let mut nodes_str = String::new();
        let len = nodes.len();

        for (i, index) in nodes.into_iter().enumerate() {
            let angle = i as f64 * 360.0 / len as f64;

            let square = LATIN_SQUARES_5[index];
            let mut rows = String::new();

            for i in 0..5 {
                for j in 0..5 {
                    let entry = square.get(i, j);

                    rows.push_str(format!("{entry}").as_str());
                    if j != 4 {
                        rows.push_str(" & ");
                    }
                }
                rows.push_str("\\\\");
            }

            nodes_str.push_str(
                format!("\\node ({index}) at ({angle}:4cm) {{$\\begin{{pmatrix}}{rows}\\end{{pmatrix}}$}};\n").as_str(),
            );
        }

        let mut path = "\\path[-]\n".to_string();

        for (i, j) in &part {
            path.push_str(format!("({i}) edge ({j})").as_str());
        }

        let string = format!("\\begin{{tikzpicture}}{nodes_str}{path};\\end{{tikzpicture}}");

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("figures/5_{part_index}.tex"))
            .unwrap();

        let mut writer = BufWriter::new(file);
        writer.write(string.as_bytes()).unwrap();
    }

    // println!("{indices:?}");

    // let string = indices
    //     .into_iter()
    //     .map(|(i, j)| format!("{i} - {j}"))
    //     .reduce(|a, b| format!("{a}\n{b}"))
    //     .unwrap();

    // println!("{string}");
}

fn generate_7_graph() {

    // let all_pairs: Vec<[LatinSquare<7>; 2]> = vec![];

    // println!("{indices:?}");

    // let string = indices
    //     .into_iter()
    //     .map(|(i, j)| format!("{i} - {j}"))
    //     .reduce(|a, b| format!("{a}\n{b}"))
    //     .unwrap();

    // println!("{string}");
}
