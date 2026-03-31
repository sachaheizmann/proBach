use std::io::{self, Read};
use ark_ff::{Field, PrimeField};
use ark_bls12_381::Fr;

fn main() {
    // read stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let lines: Vec<&str> = input.lines().collect();

    let n: usize = lines[0].parse().unwrap();
    let num_terms: usize = lines[1].parse().unwrap();

    println!("n = {}", n);
    println!("num_terms = {}", num_terms);

    for i in 0..num_terms {
        let parts: Vec<u64> = lines[2 + i]
            .split_whitespace()
            .map(|x| x.parse().unwrap())
            .collect();

        let coeff = Fr::from(parts[0]);

        println!("coeff = {}", coeff);

        let exps = &parts[1..];
        println!("exponents = {:?}", exps);
    }

    let challenge_line = lines[2 + num_terms];
    let challenges: Vec<u64> = challenge_line
        .split_whitespace()
        .map(|x| x.parse().unwrap())
        .collect();

    println!("challenges = {:?}", challenges);
}