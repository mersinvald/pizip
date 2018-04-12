#![feature(inclusive_range_syntax)]

extern crate num;
extern crate rayon;
extern crate gmp;

use rayon::prelude::*;

mod pi;
mod seeker;

use std::io::{stdout, Read, Write};
use std::fs::File;

fn read_file(path: &str) -> Vec<u8> {
    let mut vec = Vec::new();
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut vec).unwrap();
    vec
}

fn main() {
    print!("Preparing pi buffer... ");
    stdout().flush().unwrap();
    let seeker = seeker::PiSeeker::precalculate();
    println!("done.");

    let file = read_file("Cargo.toml");
    for found_block in seeker.seek(&file) {
        println!("{}", found_block);
    }
}
