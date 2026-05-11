use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::{Zero, BigInteger, PrimeField};
use ark_std::rand::SeedableRng;
use ark_std::rand::rngs::StdRng;
use efficient_sumcheck::multilinear_sumcheck;
use efficient_sumcheck::transcript::SanityTranscript;
use std::panic;

// ─── FIELD DEFINITIONS ────────────────────────────────────────────────────────

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

// ─── CONSTANTS ────────────────────────────────────────────────────────────────

const MODULI: [u64; 5] = [
    19,
    2147483647,
    2013265921,
    2130706433,
    18446744069414584321,
];

// ─── HELPERS ──────────────────────────────────────────────────────────────────

fn all_boolean_points(n: usize) -> Vec<Vec<u64>> {
    let total = 1usize << n;
    (0..total)
        .map(|i| (0..n).map(|bit| ((i >> bit) & 1) as u64).collect())
        .collect()
}

// ─── CORE SUMCHECK ────────────────────────────────────────────────────────────
// returns: claims (n+1) then round_polys (n*2) flat into out_buf
// return value: number of u64s written, or 0 on panic

macro_rules! run_sumcheck_ffi {
    ($F:ty, $terms:expr, $n:expr, $seed:expr, $out:expr) => {{
        let to_u64 = |x: $F| -> u64 { x.into_bigint().as_ref()[0] };
        let points = all_boolean_points($n);

        let evaluations: Vec<$F> = points.iter().map(|p| {
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

        let claim_first: $F = evaluations.iter().copied().sum();

        let result = panic::catch_unwind(move || {
            let mut evals = evaluations.clone();
            let mut rng = StdRng::seed_from_u64($seed);
            let mut transcript = SanityTranscript::new(&mut rng);
            multilinear_sumcheck::<$F, $F>(&mut evals, &mut transcript)
        });

        match result {
            Err(_) => 0u32,
            Ok(transcript) => {
                let mut idx = 0usize;
                // claims[0] = initial sum
                $out[idx] = to_u64(claim_first); idx += 1;
                // claims[1..n+1] = round claims
                for (i, (p0, p1)) in transcript.prover_messages.iter().enumerate() {
                    let r = transcript.verifier_messages[i];
                    let next: $F = *p0 + (*p1 - *p0) * r;
                    $out[idx] = to_u64(next); idx += 1;
                }
                // round polys: p0, p1 per round
                for (p0, p1) in &transcript.prover_messages {
                    $out[idx] = to_u64(*p0); idx += 1;
                    $out[idx] = to_u64(*p1); idx += 1;
                }
                // challenges
                for r in &transcript.verifier_messages {
                    $out[idx] = to_u64(*r); idx += 1;
                }
                idx as u32
            }
        }
    }};
}

// ─── C-CALLABLE EXPORT ────────────────────────────────────────────────────────
//
// Input:
//   field_id   : 0=Z19, 1=M31, 2=BabyBear, 3=KoalaBear, 4=Goldilocks
//   n          : number of variables
//   num_terms  : number of polynomial terms
//   coeffs     : array of num_terms coefficients
//   exps       : flat array of num_terms*n exponents
//   seed       : RNG seed
//   out_buf    : output buffer (caller allocates, size >= (3*n+1) u64s)
//
// Output layout in out_buf:
//   [0..n]       : claims (n+1 values)
//   [n+1..3n]    : round polys (p0,p1 per round = 2n values)
//   [3n+1..4n]   : challenges (n values)
//
// Returns: number of u64s written, or 0 on panic

#[no_mangle]
pub extern "C" fn rust_run_sumcheck(
    field_id: u8,
    n: u32,
    num_terms: u32,
    coeffs: *const u64,
    exps: *const u8,
    seed: u64,
    out_buf: *mut u64,
) -> u32 {
    let n = n as usize;
    let num_terms = num_terms as usize;
    let modulus = MODULI[field_id as usize % 5];

    // safety: caller guarantees valid pointers
    let coeffs_slice = unsafe { std::slice::from_raw_parts(coeffs, num_terms) };
    let exps_slice   = unsafe { std::slice::from_raw_parts(exps, num_terms * n) };
    let out_slice    = unsafe { std::slice::from_raw_parts_mut(out_buf, 4 * n + 1) };

    let terms: Vec<(u64, Vec<u64>)> = (0..num_terms).map(|t| {
        let coeff = coeffs_slice[t] % modulus;
        let exponents = (0..n).map(|v| exps_slice[t * n + v] as u64).collect();
        (coeff, exponents)
    }).collect();

    match field_id % 5 {
        0 => run_sumcheck_ffi!(F19,        terms, n, seed, out_slice),
        1 => run_sumcheck_ffi!(M31,        terms, n, seed, out_slice),
        2 => run_sumcheck_ffi!(BabyBear,   terms, n, seed, out_slice),
        3 => run_sumcheck_ffi!(KoalaBear,  terms, n, seed, out_slice),
        4 => run_sumcheck_ffi!(Goldilocks, terms, n, seed, out_slice),
        _ => 0,
    }
}
