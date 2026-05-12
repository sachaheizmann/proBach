// supports old API only

use std::io::Read;
use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::{Zero, BigInteger, PrimeField};
use ark_std::rand::SeedableRng;
use ark_std::rand::rngs::StdRng;
use effsc::runner::sumcheck;
use effsc::provers::multilinear::MultilinearProver;
use effsc::transcript::SanityTranscript;

#[derive(MontConfig)]
#[modulus = "19"]
#[generator = "2"]
pub struct F19Config;
pub type F19 = Fp64<MontBackend<F19Config, 1>>;

#[derive(MontConfig)]
#[modulus = "2147483647"]
#[generator = "7"]
pub struct M31Config;
pub type M31 = Fp64<MontBackend<M31Config, 1>>;

#[derive(MontConfig)]
#[modulus = "2013265921"]
#[generator = "31"]
pub struct BabyBearConfig;
pub type BabyBear = Fp64<MontBackend<BabyBearConfig, 1>>;

#[derive(MontConfig)]
#[modulus = "2130706433"]
#[generator = "3"]
pub struct KoalaBearConfig;
pub type KoalaBear = Fp64<MontBackend<KoalaBearConfig, 1>>;

#[derive(MontConfig)]
#[modulus = "18446744069414584321"]
#[generator = "7"]
pub struct GoldilocksConfig;
pub type Goldilocks = Fp64<MontBackend<GoldilocksConfig, 1>>;

const MODULI: [u64; 5] = [19, 2147483647, 2013265921, 2130706433, 18446744069414584321];

fn all_boolean_points(n: usize) -> Vec<Vec<u64>> {
    let total = 1usize << n;
    (0..total)
        .map(|i| (0..n).map(|bit| ((i >> bit) & 1) as u64).collect())
        .collect()
}

fn run<F: ark_ff::Field + ark_ff::Zero + ark_ff::One + Copy + std::iter::Sum>(
    evaluations: Vec<F>, seed: u64
) where F: ark_ff::PrimeField {
    let mut evals = evaluations;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut transcript = SanityTranscript::new(&mut rng);
    let _ = multilinear_sumcheck::<F, F>(&mut evals, &mut transcript);
}

macro_rules! build_and_run {
    ($F:ty, $terms:expr, $points:expr, $seed:expr) => {{
        let evals: Vec<$F> = $points.iter().map(|p| {
            let mut r = <$F>::zero();
            for (coeff, exps) in &$terms {
                let mut v = <$F>::from(*coeff);
                for (i, &e) in exps.iter().enumerate() {
                    if e > 0 && p[i] == 0 { v = <$F>::zero(); break; }
                }
                r += v;
            }
            r
        }).collect();
        let mut evals2 = evals;
        let mut rng = StdRng::seed_from_u64($seed);
        let mut transcript = SanityTranscript::new(&mut rng);
        let _ = multilinear_sumcheck::<$F, $F>(&mut evals2, &mut transcript);
    }};
}

fn main() {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).unwrap();
    if data.len() < 3 { return; }

    let field_id = (data[0] % 5) as usize;
    let n        = ((data[1] as usize) % 4) + 1;
    let num_terms = ((data[2] as usize) % 8) + 1;
    let modulus  = MODULI[field_id];

    let mut pos = 3;
    let mut terms: Vec<(u64, Vec<u64>)> = Vec::new();
    for _ in 0..num_terms {
        if pos >= data.len() { break; }
        let coeff = data[pos] as u64 % modulus;
        pos += 1;
        let mut exps = Vec::new();
        for _ in 0..n {
            exps.push(if pos < data.len() { let e = (data[pos] & 1) as u64; pos += 1; e } else { 0 });
        }
        terms.push((coeff, exps));
    }
    if terms.is_empty() { terms.push((1, vec![0; n])); }

    let seed = if data.len() >= 8 {
        u64::from_le_bytes(data[data.len()-8..].try_into().unwrap_or([0;8]))
    } else { 42 };

    let points = all_boolean_points(n);

    match field_id {
        0 => build_and_run!(F19,        terms, points, seed),
        1 => build_and_run!(M31,        terms, points, seed),
        2 => build_and_run!(BabyBear,   terms, points, seed),
        3 => build_and_run!(KoalaBear,  terms, points, seed),
        4 => build_and_run!(Goldilocks, terms, points, seed),
        _ => {}
    }
}
