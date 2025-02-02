use std::{fs::File, io::Read};

use args::Args;
use clap::Parser;
use ebnf2pest::translate;

mod args;

fn main() {
    let args = Args::parse();

    let mut file = File::open(& args.file).expect("Failed to open file");
    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf).unwrap();
    let grammar = match ebnf2pest::parse(&buf) {
        Ok(grammar) => grammar,
        Err(e) => {
            eprintln!("Failed to parse EBNF: {}", e);
            return;
        }
    };
    println!("{}", translate(grammar));
}