import subprocess
import random
from multiprocessing.pool import ThreadPool

LEAN_CMD = ["lake", "exe", "sumcheck"]
LEAN_DIR = "../sumcheck-lean4"
CANDIDATE_CMD = ["./target/release/epfl_sumcheck"]
CANDIDATE_DIR = "../candidate/epfl_sumcheck"
BATCH_SIZE = 16

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

def deduplicate_terms(terms, mod=19):
    """combine terms with identical exponents by summing coefficients mod 19"""
    merged = {}
    for t in terms:
        coeff = t[0]
        exps = tuple(t[1:])
        merged[exps] = (merged.get(exps, 0) + coeff) % mod
    return [[coeff] + list(exps) for exps, coeff in merged.items() if coeff != 0]

def random_poly(n):
    """Generate random polynomial terms with no duplicate monomials"""
    num_terms = random.randint(1, 5)
    terms = []
    for _ in range(num_terms):
        coeff = random.randint(0, 18)
        exponents = [random.randint(0, 1) for _ in range(n)]
        terms.append([coeff] + exponents)
    return deduplicate_terms(terms)

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
            timeout=30,
        )
        return result.stdout.decode()
    except Exception as e:
        return f"ERROR: {e}"

# ---------- SINGLE TEST ----------

def run_single_test(args):
    """run one full test: rust + lean, return comparison result"""
    n, terms, seed, test_num = args

    rust_input = random_input_rust(n, terms, seed)
    rust_out = run(CANDIDATE_CMD, CANDIDATE_DIR, rust_input)

    if "ERROR" in rust_out:
        return {"ok": True, "skipped": True}

    challenges = parse_challenges(rust_out)
    if challenges is None:
        return {"ok": True, "skipped": True}

    lean_input = random_input_lean(n, terms, challenges)
    lean_out = run(LEAN_CMD, LEAN_DIR, lean_input)

    rust_claims, rust_polys = parse_transcript(rust_out)
    lean_claims, lean_polys = parse_transcript(lean_out)

    match = rust_claims == lean_claims and rust_polys == lean_polys

    return {
        "ok": match,
        "skipped": False,
        "test_num": test_num,
        "n": n,
        "terms": terms,
        "seed": seed,
        "rust_claims": rust_claims,
        "lean_claims": lean_claims,
        "rust_polys": rust_polys,
        "lean_polys": lean_polys,
        "rust_input": rust_input,
        "lean_input": lean_input,
    }

# ---------- MAIN LOOP ----------

def main():
    total = 0
    pool = ThreadPool(BATCH_SIZE)

    while True:
        # generate a batch of random test cases
        batch = []
        for _ in range(BATCH_SIZE):
            n = random.randint(1, 4)
            terms = random_poly(n)
            seed = random.randint(0, 2**32)
            total += 1
            batch.append((n, terms, seed, total))

        # run all tests in parallel
        results = pool.map(run_single_test, batch)

        # print results in order
        mismatch_found = False
        for r in results:
            if r.get("skipped"):
                continue

            print(f"=== Test {r['test_num']} ===")
            print(f"n={r['n']} terms={len(r['terms'])} seed={r['seed']}")
            print(f"Rust:  claims={r['rust_claims']} polys={r['rust_polys']}")
            print(f"Lean:  claims={r['lean_claims']} polys={r['lean_polys']}")

            if r["ok"]:
                print("✅ OK")
            else:
                print("❌ MISMATCH FOUND!")
                print(f"\nRust input:\n{r['rust_input']}")
                print(f"\nLean input:\n{r['lean_input']}")
                mismatch_found = True
                break

        if mismatch_found:
            pool.terminate()
            break

if __name__ == "__main__":
    main()