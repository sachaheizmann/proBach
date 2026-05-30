use std::io::Read;
use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::{Zero, PrimeField};
use ark_std::rand::SeedableRng;
use ark_std::rand::rngs::StdRng;
use effsc::provers::multilinear::MultilinearProver;
use effsc::runner::sumcheck;
use effsc::transcript::SanityTranscript;
use std::panic;

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
const MAX_N: usize = 24;
const MAX_TERMS: usize = 256;

// MSB ordering, matching lib.rs
fn all_boolean_points(n: usize) -> Vec<Vec<u64>> {
    let total = 1usize << n;
    (0..total)
        .map(|i| (0..n).map(|bit| ((i >> (n - 1 - bit)) & 1) as u64).collect())
        .collect()
}

// Same sumcheck call as lib.rs
macro_rules! build_and_run {
    ($F:ty, $terms:expr, $points:expr, $seed:expr) => {{
        let evaluations: Vec<$F> = $points.iter().map(|p| {
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
        let _ = panic::catch_unwind(move || {
            let mut prover = MultilinearProver::new(evaluations.clone());
            let num_rounds = prover.num_variables();
            let mut rng = StdRng::seed_from_u64($seed);
            let mut transcript = SanityTranscript::new(&mut rng);
            sumcheck(&mut prover, num_rounds, &mut transcript, |_, _| {})
        });
    }};
}

fn main() {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).unwrap();
    if data.len() < 18 { return; }

    let field_id = (data[0] % 5) as usize;
    let n = ((data[1] as usize) % MAX_N) + 1;
    let modulus = MODULI[field_id];

    let exp_bytes = (n + 7) / 8;
    let term_size = 8 + exp_bytes;

    let mut pos = 2;
    let remaining = if data.len() >= 8 + pos { data.len() - 8 - pos } else { 0 };
    let mut num_terms = remaining / term_size;
    if num_terms > MAX_TERMS { num_terms = MAX_TERMS; }
    if num_terms == 0 { num_terms = 1; }

    let mut terms: Vec<(u64, Vec<u64>)> = Vec::new();
    for _ in 0..num_terms {
        let mut coeff: u64 = 0;
        for b in 0..8 {
            let byte = if pos < data.len() { data[pos] } else { 0 };
            coeff |= (byte as u64) << (8 * b);
            pos += 1;
        }
        coeff %= modulus;

        let mut exps = vec![0u64; n];
        for v in 0..n {
            let byte_idx = v / 8;
            let bit_idx = v % 8;
            let byte = if pos + byte_idx < data.len() { data[pos + byte_idx] } else { 0 };
            exps[v] = ((byte >> bit_idx) & 1) as u64;
        }
        pos += exp_bytes;
        terms.push((coeff, exps));
    }

    let seed = if data.len() >= 8 {
        u64::from_le_bytes(data[data.len()-8..].try_into().unwrap_or([0; 8]))
    } else { 42 };

    let points = all_boolean_points(n);
    match field_id {
        0 => build_and_run!(F19, terms, points, seed),
        1 => build_and_run!(M31, terms, points, seed),
        2 => build_and_run!(BabyBear, terms, points, seed),
        3 => build_and_run!(KoalaBear, terms, points, seed),
        4 => build_and_run!(Goldilocks, terms, points, seed),
        _ => {}
    }
}