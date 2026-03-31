import subprocess
import random
import tempfile
import os

# ---------- CONFIG ----------
LEAN_CMD = ["lake", "exe", "sumcheck"]
LEAN_DIR = "../sumcheck-lean4"

CANDIDATE_CMD = LEAN_CMD #["../candidate/sumcheck_impl"]  # change this later


# ---------- INPUT GENERATION ----------
def random_input():
    n = random.randint(1, 4)  # keep small for now
    num_terms = random.randint(1, 5)

    terms = []
    for _ in range(num_terms):
        coeff = random.randint(0, 18)
        exponents = [random.randint(0, 1) for _ in range(n)]
        terms.append([coeff] + exponents)

    challenges = [random.randint(0, 18) for _ in range(n)]

    # build text input
    lines = []
    lines.append(str(n))
    lines.append(str(num_terms))

    for t in terms:
        lines.append(" ".join(map(str, t)))

    lines.append(" ".join(map(str, challenges)))

    return "\n".join(lines)


# ---------- RUN PROGRAM ----------
def run(cmd, cwd, input_str):
    try:
        result = subprocess.run(
            cmd,
            input=input_str.encode(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            cwd=cwd,
            timeout=5,
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

        inp = random_input()
        print("Input:")
        print(inp)

        lean_out = run(LEAN_CMD, LEAN_DIR, inp)
        cand_out = run(CANDIDATE_CMD, LEAN_DIR, inp) # modify later

        print("\nLean output:")
        print(lean_out)

        print("\nCandidate output:")
        print(cand_out)

        if lean_out != cand_out:
            print("\n❌ MISMATCH FOUND!")
            print("Input was:")
            print(inp)
            break
        else:
            print("✅ OK")


if __name__ == "__main__":
    main()