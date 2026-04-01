use std::io::{self, Read};
use efficient_sumcheck::{multilinear_sumcheck, Sumcheck};
use efficient_sumcheck::transcript::SanityTranscript;
use ark_std::rand::SeedableRng;
use ark_std::rand::rngs::StdRng;
use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::{Field, Zero, BigInteger, PrimeField};


// ZMod 19, same field as in Lean (hardcoded for now)
#[derive(MontConfig)]
#[modulus = "19"]
#[generator = "2"]
pub struct F19Config;
pub type F19 = Fp64<MontBackend<F19Config, 1>>;

// Convert field element to its canonical integer 0..18
fn to_u64(x: F19) -> u64 {
    x.into_bigint().as_ref()[0]
}

fn parse_u64_list(s: &str) -> Vec<u64> {
    s.split_whitespace()
        .filter_map(|x| x.parse().ok())
        .collect()
}

// terms: list of (coeff, exponents)
// point: boolean assignment e.g. [0, 1, 0]
fn eval_poly(terms: &[(u64, Vec<u64>)], point: &[u64]) -> F19 {
    let mut result = F19::zero();
    for (coeff, exponents) in terms {
        let mut term_val = F19::from(*coeff);
        for (i, &exp) in exponents.iter().enumerate() {
            // x^exp where x in {0,1}:
            // if x=0 and exp>0 → whole term is 0
            if exp > 0 && point[i] == 0 {
                term_val = F19::zero();
                break;
            }
            // if x=1 or exp=0 → factor is 1, nothing to do
        }
        result += term_val;
    }
    result
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


fn main() {
    // --- parse stdin ---
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let lines: Vec<&str> = input
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    // line 0: n
    let n: usize = lines[0].trim().parse().unwrap();

    // line 1: number of terms
    let num_terms: usize = lines[1].trim().parse().unwrap();

    // lines 2..2+num_terms: "coeff exp0 exp1 ... exp_{n-1}"
    let mut terms: Vec<(u64, Vec<u64>)> = Vec::new();
    for i in 0..num_terms {
        let nums = parse_u64_list(lines[2 + i]);
        let coeff = nums[0];
        let exponents = nums[1..].to_vec();
        terms.push((coeff, exponents));
    }

    // line 2+num_terms: seed
    let seed: u64 = lines[2 + num_terms].trim().parse().unwrap();

    // --- build evaluation table ---
    // evaluate polynomial at every boolean point in {0,1}^n
    let points = all_boolean_points(n);
    let mut evaluations: Vec<F19> = points
        .iter()
        .map(|p| eval_poly(&terms, p))
        .collect();

    // claim_first = sum over all boolean points
    let claim_first: F19 = evaluations.iter().copied().sum();

    // --- run EPFL sumcheck ---
    let mut rng = StdRng::seed_from_u64(seed);
    let mut transcript = SanityTranscript::new(&mut rng);
    let result = multilinear_sumcheck::<F19, F19>(&mut evaluations, &mut transcript);

    // --- print transcript in Lean-compatible format ---
    println!("=== SUMCHECK TRANSCRIPT ===");

    // reconstruct claims list
    // claims[0] = claim_first (sum over hypercube)
    // claims[i+1] = round_poly_i evaluated at challenge_i
    //             = p0 + (p1 - p0) * r
    let mut claims: Vec<u64> = vec![to_u64(claim_first)];
    for (i, (p0, p1)) in result.prover_messages.iter().enumerate() {
        let r = result.verifier_messages[i];
        let next_claim: F19 = *p0 + (*p1 - *p0) * r;
        claims.push(to_u64(next_claim));
    }

    // print claims
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
}