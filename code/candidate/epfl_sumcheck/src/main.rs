use std::io::{self, Read};
use efficient_sumcheck::multilinear_sumcheck;
use efficient_sumcheck::transcript::SanityTranscript;
use ark_std::rand::SeedableRng;
use ark_std::rand::rngs::StdRng;
use ark_ff::fields::{Fp64, Fp128, MontBackend, MontConfig};
use ark_ff::{Zero, BigInteger, PrimeField};

// ---------- FIELD DEFINITIONS ----------

// ZMod 19
#[derive(MontConfig)]
#[modulus = "19"]
#[generator = "2"]
pub struct F19Config;
pub type F19 = Fp64<MontBackend<F19Config, 1>>;

// Mersenne 31: 2^31 - 1
#[derive(MontConfig)]
#[modulus = "2147483647"]
#[generator = "7"]
pub struct M31Config;
pub type M31 = Fp64<MontBackend<M31Config, 1>>;

// BabyBear: 2^31 - 2^27 + 1
#[derive(MontConfig)]
#[modulus = "2013265921"]
#[generator = "31"]
pub struct BabyBearConfig;
pub type BabyBear = Fp64<MontBackend<BabyBearConfig, 1>>;

// KoalaBear: 2^31 - 2^24 + 1
#[derive(MontConfig)]
#[modulus = "2130706433"]
#[generator = "3"]
pub struct KoalaBearConfig;
pub type KoalaBear = Fp64<MontBackend<KoalaBearConfig, 1>>;

// Goldilocks: 2^64 - 2^32 + 1
#[derive(MontConfig)]
#[modulus = "18446744069414584321"]
#[generator = "7"]
pub struct GoldilocksConfig;
pub type Goldilocks = Fp64<MontBackend<GoldilocksConfig, 1>>;

// ---------- HELPERS ----------

fn parse_u64_list(s: &str) -> Vec<u64> {
    s.split_whitespace()
        .filter_map(|x| x.parse().ok())
        .collect()
}

fn all_boolean_points(n: usize) -> Vec<Vec<u64>> {
    let total = 1usize << n;
    (0..total)
        .map(|i| {
            // least significant bit first (bit 0 = variable 0)
            (0..n)
                .map(|bit| ((i >> bit) & 1) as u64)
                .collect()
        })
        .collect()
}

// ---------- CORE LOGIC AS MACRO ----------
// We use a macro to avoid duplicating the entire sumcheck logic
// for each field type. The macro takes a field type F and runs
// the full protocol, printing the transcript.

macro_rules! run_sumcheck {
    ($F:ty, $terms:expr, $n:expr, $seed:expr) => {{
        use ark_ff::Zero;

        // convert field element to canonical integer
        let to_u64 = |x: $F| -> u64 {
            x.into_bigint().as_ref()[0]
        };

        // evaluate polynomial at a single boolean point
        let eval_poly = |terms: &[(u64, Vec<u64>)], point: &[u64]| -> $F {
            let mut result = <$F>::zero();
            for (coeff, exponents) in terms {
                let mut term_val = <$F>::from(*coeff);
                for (i, &exp) in exponents.iter().enumerate() {
                    // if x=0 and exp>0 → term is 0
                    if exp > 0 && point[i] == 0 {
                        term_val = <$F>::zero();
                        break;
                    }
                    // if x=1 or exp=0 → factor is 1, nothing to do
                }
                result += term_val;
            }
            result
        };

        // build evaluation table over all 2^n boolean points
        let points = all_boolean_points($n);
        let mut evaluations: Vec<$F> = points
            .iter()
            .map(|p| eval_poly($terms, p))
            .collect();

        // claim_first = sum over all boolean points
        let claim_first: $F = evaluations.iter().copied().sum();

        // run EPFL sumcheck with seeded RNG
        let mut rng = StdRng::seed_from_u64($seed);
        let mut transcript = SanityTranscript::new(&mut rng);
        let result = multilinear_sumcheck::<$F, $F>(&mut evaluations, &mut transcript);

        // print transcript
        println!("=== SUMCHECK TRANSCRIPT ===");

        // reconstruct and print claims
        // claims[0] = initial sum
        // claims[i+1] = round_poly_i(challenge_i) = p0 + (p1-p0)*r
        let mut claims: Vec<u64> = vec![to_u64(claim_first)];
        for (i, (p0, p1)) in result.prover_messages.iter().enumerate() {
            let r = result.verifier_messages[i];
            let next_claim: $F = *p0 + (*p1 - *p0) * r;
            claims.push(to_u64(next_claim));
        }

        print!("claims: [");
        for (i, c) in claims.iter().enumerate() {
            if i > 0 { print!(", "); }
            print!("{}", c);
        }
        println!("]");

        // print challenges
        print!("challenges: [");
        for (i, r) in result.verifier_messages.iter().enumerate() {
            if i > 0 { print!(", "); }
            print!("{}", to_u64(*r));
        }
        println!("]");

        // print round polynomials
        for (i, (p0, p1)) in result.prover_messages.iter().enumerate() {
            println!(
                "round_poly_{}: [{}, {}]",
                i,
                to_u64(*p0),
                to_u64(*p1)
            );
        }

        println!("=== END ===");
    }};
}

// ---------- MAIN ----------

fn main() {
    use std::io::BufRead;
    let stdin = io::stdin();
    let mut lines_iter = stdin.lock().lines();
    
    loop {
        // collect lines until "---" or EOF
        let mut block: Vec<String> = Vec::new();
        let mut got_input = false;
        
        loop {
            match lines_iter.next() {
                None => {
                    // EOF — exit cleanly
                    return;
                }
                Some(Ok(line)) => {
                    if line.trim() == "---" {
                        break; // end of this test case
                    }
                    if !line.trim().is_empty() {
                        block.push(line);
                        got_input = true;
                    }
                }
                Some(Err(_)) => return,
            }
        }
        
        if !got_input { continue; }
        
        // parse exactly like before
        let field_id: u8 = block[0].trim().parse().unwrap();
        let n: usize     = block[1].trim().parse().unwrap();
        let num_terms: usize = block[2].trim().parse().unwrap();
        
        let mut terms: Vec<(u64, Vec<u64>)> = Vec::new();
        for i in 0..num_terms {
            let nums = parse_u64_list(&block[3 + i]);
            let coeff = nums[0];
            let exponents = nums[1..].to_vec();
            terms.push((coeff, exponents));
        }
        let seed: u64 = block[3 + num_terms].trim().parse().unwrap();
        
        // run sumcheck — same as before
        match field_id {
            0 => run_sumcheck!(F19,        &terms, n, seed),
            1 => run_sumcheck!(M31,        &terms, n, seed),
            2 => run_sumcheck!(BabyBear,   &terms, n, seed),
            3 => run_sumcheck!(KoalaBear,  &terms, n, seed),
            4 => run_sumcheck!(Goldilocks, &terms, n, seed),
            _ => eprintln!("Unknown field_id: {}", field_id),
        }
        
        // signal to harness that output is complete
        println!("---");
        
        // flush immediately so harness doesn't wait forever
        use std::io::Write;
        io::stdout().flush().unwrap();
    }
}