#[macro_use]
extern crate afl;

use std::panic;
use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::{Zero, BigInteger, PrimeField};
use ark_std::rand::SeedableRng;
use ark_std::rand::rngs::StdRng;
use efficient_sumcheck::multilinear_sumcheck;
use efficient_sumcheck::transcript::SanityTranscript;

// ─── PRIME FIELDS ─────────────────────────────────────────────────────────────

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

// ─── NON-PRIME FIELDS ─────────────────────────────────────────────────────────
// intentionally non-prime to test robustness
// Rust's MontConfig does not verify primality at compile time

#[derive(MontConfig)]
#[modulus = "15"]
#[generator = "2"]
pub struct F15Config;
pub type F15 = Fp64<MontBackend<F15Config, 1>>;

#[derive(MontConfig)]
#[modulus = "9"]
#[generator = "2"]
pub struct F9Config;
pub type F9 = Fp64<MontBackend<F9Config, 1>>;

#[derive(MontConfig)]
#[modulus = "25"]
#[generator = "2"]
pub struct F25Config;
pub type F25 = Fp64<MontBackend<F25Config, 1>>;

// ─── CONSTANTS ────────────────────────────────────────────────────────────────

// 0-4 = prime fields, 5-7 = non-prime fields
const MODULI: [u64; 8] = [
    19,
    2147483647,
    2013265921,
    2130706433,
    18446744069414584321,
    15,
    9,
    25,
];

// ─── HELPERS ──────────────────────────────────────────────────────────────────

fn all_boolean_points(n: usize) -> Vec<Vec<u64>> {
    let total = 1usize << n;
    (0..total)
        .map(|i| (0..n).map(|bit| ((i >> bit) & 1) as u64).collect())
        .collect()
}

// ─── SUMCHECK MACRO ───────────────────────────────────────────────────────────

macro_rules! run_sumcheck {
    ($F:ty, $evaluations:expr, $seed:expr) => {{
        let mut evals: Vec<$F> = $evaluations;
        panic::catch_unwind(move || {
            let mut rng = StdRng::seed_from_u64($seed);
            let mut transcript = SanityTranscript::new(&mut rng);
            multilinear_sumcheck::<$F, $F>(&mut evals, &mut transcript)
        }).is_err()
    }};
}

macro_rules! build_evals {
    ($F:ty, $terms:expr, $points:expr) => {{
        $points.iter().map(|p| {
            let mut r = <$F>::zero();
            for (coeff, exps) in &$terms {
                let mut v = <$F>::from(*coeff);
                for (i, &e) in exps.iter().enumerate() {
                    if e > 0 && p[i] == 0 {
                        v = <$F>::zero();
                        break;
                    }
                }
                r += v;
            }
            r
        }).collect::<Vec<$F>>()
    }};
}

// ─── MAIN ─────────────────────────────────────────────────────────────────────

fn main() {
    fuzz!(|data: &[u8]| {
        if data.len() < 3 { return; }

        // field_id: 0..7 (0-4 prime, 5-7 non-prime)
        let field_id = (data[0] % 8) as usize;

        // n: 1..16
        let n = ((data[1] as usize) % 4) + 1;

        // num_terms: 1..32
        let num_terms = ((data[2] as usize) % 32) + 1;

        let modulus = MODULI[field_id];

        // parse terms with raw exponents
        let mut pos = 3;
        let mut terms: Vec<(u64, Vec<u64>)> = Vec::new();
        for _ in 0..num_terms {
            if pos >= data.len() { break; }
            let coeff = data[pos] as u64 % modulus;
            pos += 1;
            let mut exps = Vec::new();
            for _ in 0..n {
                exps.push(if pos < data.len() {
                    let e = data[pos] as u64;
                    pos += 1;
                    e
                } else { 0 });
            }
            terms.push((coeff, exps));
        }
        if terms.is_empty() {
            terms.push((1, vec![0; n]));
        }

        let seed = if data.len() >= 8 {
            u64::from_le_bytes(data[data.len()-8..].try_into().unwrap_or([0;8]))
        } else { 42 };

        let points = all_boolean_points(n);

        let panicked = match field_id {
            0 => run_sumcheck!(F19,        build_evals!(F19,        terms, points), seed),
            1 => run_sumcheck!(M31,        build_evals!(M31,        terms, points), seed),
            2 => run_sumcheck!(BabyBear,   build_evals!(BabyBear,   terms, points), seed),
            3 => run_sumcheck!(KoalaBear,  build_evals!(KoalaBear,  terms, points), seed),
            4 => run_sumcheck!(Goldilocks, build_evals!(Goldilocks, terms, points), seed),
            5 => run_sumcheck!(F15,        build_evals!(F15,        terms, points), seed),
            6 => run_sumcheck!(F9,         build_evals!(F9,         terms, points), seed),
            7 => run_sumcheck!(F25,        build_evals!(F25,        terms, points), seed),
            _ => return,
        };

        if panicked {
            std::process::abort();
        }
    });
}
