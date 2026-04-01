import subprocess
import random

LEAN_CMD = ["lake", "exe", "sumcheck"]
LEAN_DIR = "../../sumcheck-lean4"
CANDIDATE_CMD = ["./target/release/epfl_sumcheck"]
CANDIDATE_DIR = "../candidate/epfl_sumcheck"

# ---------- INPUT GENERATION ----------

def random_input_rust(n, terms, seed):
    """Input for Rust: ends with a seed"""
    lines = []
    lines.append(str(n))
    lines.append(str(len(terms)))
    for t in terms:
        lines.append(" ".join(map(str, t)))
    lines.append(str(seed))
    return "\n".join(lines)

def random_input_lean(n, terms, challenges):
    """Input for Lean: ends with challenges"""
    lines = []
    lines.append(str(n))
    lines.append(str(len(terms)))
    for t in terms:
        lines.append(" ".join(map(str, t)))
    lines.append(" ".join(map(str, challenges)))
    return "\n".join(lines)

def random_poly(n):
    """Generate random polynomial terms"""
    num_terms = random.randint(1, 5)
    terms = []
    for _ in range(num_terms):
        coeff = random.randint(0, 18)
        exponents = [random.randint(0, 1) for _ in range(n)]
        terms.append([coeff] + exponents)
    return terms

# ---------- PARSING ----------

def parse_challenges(output):
    """Extract challenges list from Rust output"""
    for line in output.splitlines():
        if line.startswith("challenges: ["):
            inner = line[len("challenges: ["):-1]
            return [int(x.strip()) for x in inner.split(",")]
    return None

def parse_transcript(output):
    """Extract claims and round polys for comparison"""
    claims = None
    round_polys = []
    for line in output.splitlines():
        if line.startswith("claims: ["):
            inner = line[len("claims: ["):-1]
            claims = [int(x.strip()) for x in inner.split(",")]
        if line.startswith("round_poly_"):
            inner = line.split("[")[1].rstrip("]")
            vals = [int(x.strip()) for x in inner.split(",")]
            round_polys.append(vals)
    return claims, round_polys

# ---------- RUN ----------

def run(cmd, cwd, input_str):
    try:
        result = subprocess.run(
            cmd,
            input=input_str.encode(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            cwd=cwd,
            timeout=10,
        )
        return result.stdout.decode()
    except Exception as e:
        return f"ERROR: {e}"

# ---------- MAIN LOOP ----------

def main():
    iteration = 0
    while True:
        iteration += 1
        print(f"\n=== Test {iteration} ===")

        # generate random polynomial and seed
        n = random.randint(1, 4)
        terms = random_poly(n)
        seed = random.randint(0, 2**32)

        # run Rust candidate first to get challenges
        rust_input = random_input_rust(n, terms, seed)
        rust_out = run(CANDIDATE_CMD, CANDIDATE_DIR, rust_input)

        if "ERROR" in rust_out:
            print(f"Rust error: {rust_out}")
            continue

        # extract challenges from Rust output
        challenges = parse_challenges(rust_out)
        if challenges is None:
            print("Could not parse challenges from Rust output")
            print(rust_out)
            continue

        # run Lean with those same challenges
        lean_input = random_input_lean(n, terms, challenges)
        lean_out = run(LEAN_CMD, LEAN_DIR, lean_input)

        # parse both transcripts
        rust_claims, rust_polys = parse_transcript(rust_out)
        lean_claims, lean_polys = parse_transcript(lean_out)

        print(f"n={n} terms={len(terms)} seed={seed}")
        print(f"Rust:  claims={rust_claims} polys={rust_polys}")
        print(f"Lean:  claims={lean_claims} polys={lean_polys}")

        if rust_claims != lean_claims or rust_polys != lean_polys:
            print("\n❌ MISMATCH FOUND!")
            print("Rust input:")
            print(rust_input)
            print("Lean input:")
            print(lean_input)
            print("Rust output:")
            print(rust_out)
            print("Lean output:")
            print(lean_out)
            break
        else:
            print("✅ OK")

if __name__ == "__main__":
    main()