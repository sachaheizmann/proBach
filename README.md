# Sumcheck Differential Fuzzer

A differential fuzzing framework that tests the [EPFL sumcheck library](https://github.com/compsec-epfl/efficient-sumcheck) against a formally verified [Lean4 oracle](https://github.com/z-tech/z-lean).

## Overview

The sumcheck protocol is a fundamental building block of modern zero-knowledge proof systems. This project uses AFL++ to generate randomized polynomial inputs and verifies that the EPFL Rust implementation produces identical transcripts to the Lean4 oracle which has machine-checked proofs of completeness and soundness.

Any semantic disagreement between the two implementations is reported as a crash,the input is saved for analysis.

## Fields Tested

The fuzzer tests 5 prime fields for the moment, all using Montgomery arithmetic via `ark-ff`:

| Field | Modulus |
|---|---|
| ZMod 19 | 19 |
| M31 | 2^31 - 1 |
| BabyBear | 2^31 - 2^27 + 1 |
| KoalaBear | 2^31 - 2^24 + 1 |
| Goldilocks | 2^64 - 2^32 + 1 |

ZMod 19 is for testing purpose only.

Each field has a dedicated Lean oracle (`computeTranscriptZ19`, `computeTranscriptM31`, etc.) and is tested differentially against the same EPFL Rust implementation.

## Known Limitations

**Input size:**
- Polynomials are capped at `MAX_N=24` variables and `MAX_TERMS=256` monomials
- At `n=24`, each test case takes ~13 seconds, AFL++ speed drops significantly for large `n`

**Protocol coverage:**
- Only `MultilinearProver` is tested, `InnerProductProver`, `CoefficientProver`, `SpaceProver` and `BlendyProver` are not covered
- Only the honest prover is tested, no malicious prover scenarios
- Fiat-Shamir is not tested yet,  challenges come from a seeded RNG (`SanityTranscript`)

**Lean oracle:**
- Only ZMod 19 has a formal proof of primality (`by decide`), the other 4 fields use `axiom` to assert primality without proof
- The Lean oracle is only tested against ZMod 19, M31, BabyBear, KoalaBear, and Goldilock

## Architecture

The fuzzer consists of three components working together:

**C Harness (`fuzzer/afl_harness/harness.c`)**
The central orchestrator. Parses AFL++ byte inputs into sparse polynomial descriptions, calls both oracles via direct FFI, and compares their outputs. Uses AFL++ persistent mode — the Lean runtime is initialized once and reused across 100,000 iterations before restarting.

**Rust Candidate (`candidate/epfl_sumcheck/`)**
Wraps the EPFL sumcheck library behind a C-callable FFI function `rust_run_sumcheck`. Converts the sparse polynomial to an evaluation table, runs the protocol with a seeded RNG, and returns `g_j(0)` per round (EvalsInfty wire format), challenges, and the final value in a flat `uint64_t` buffer.

**Lean Oracle (`lean/SumcheckFFI.lean`)**
Exposes 5 formally verified sumcheck implementations (one per prime field) via `@[export]` functions. Takes the same polynomial and challenges as input and returns `g_j(0)` per round and the final value for comparison.

## Repository Structure

```
.
├── candidate/
│   └── epfl_sumcheck/          ← Rust wrapper around the EPFL sumcheck library
│       ├── src/
│       │   ├── lib.rs          ← C FFI export: rust_run_sumcheck()
│       │   ├── main.rs         ← standalone binary for manual testing
│       │   ├── fuzz_target.rs  ← AFL++ fuzz target for Rust-only campaign
│       │   └── cov_target.rs   ← LLVM coverage binary
│       └── Cargo.toml
├── fuzzer/
│   └── afl_harness/
│       ├── harness.c           ← combined differential fuzzing harness
│       └── seeds/              ← AFL++ seed inputs (one per field)
└── lean/
    ├── SumcheckFFI.lean        ← FFI exports for all 5 prime fields
    ├── Main.lean               ← ZMod 19 standalone binary
    ├── MainM31.lean            ← M31 standalone binary
    ├── MainBabyBear.lean       ← BabyBear standalone binary
    ├── MainKoalaBear.lean      ← KoalaBear standalone binary
    ├── MainGoldilocks.lean     ← Goldilocks standalone binary
    └── lakefile.lean           ← Lake build configuration
```

## Dependencies

**Differential fuzzing harness:**
```bash
sudo apt install afl++ clang lld
```

**Rust-only fuzzing campaign:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install cargo-afl
```

## Build Instructions (to be tested)

**1. Build Lean shared library**
```bash
cd lean/
lake build MainShared
```
This compiles all 5 prime field oracles into a single shared library `libdifferential__testing_MainShared.so`. Takes 1-2 hours on first build — Mathlib is downloaded and compiled from source. Subsequent builds are incremental.

**2. Build Rust shared library**
```bash
cd candidate/epfl_sumcheck/
cargo build --release --lib
```

**3. Compile the harness**
```bash
cd lean/
export LEAN_PREFIX=$(lean --print-prefix)
RUST_LIB=../candidate/epfl_sumcheck/target/release

L_FLAGS=$(find .lake -name "*.so" | xargs -I{} dirname {} | sort -u | sed 's/^/-L/' | tr '\n' ' ')
l_FLAGS=$(find .lake -name "*.so" | xargs -I{} basename {} | sed 's/^lib//' | sed 's/\.so$//' | sed 's/^/-l/' | tr '\n' ' ')
RPATH_FLAGS=$(find $(pwd)/.lake -name "*.so" | xargs -I{} dirname {} | sort -u | sed 's/^/-Wl,-rpath,/' | tr '\n' ' ')

afl-clang-fast ../fuzzer/afl_harness/harness.c \
  -I $LEAN_PREFIX/include \
  -fuse-ld=lld \
  $L_FLAGS $l_FLAGS \
  -L $LEAN_PREFIX/lib/lean -L $LEAN_PREFIX/lib \
  -L $RUST_LIB -lepfl_sumcheck \
  -Wl,--start-group -lleancpp -lLean -lStd -lInit -lleanrt -lLake -Wl,--end-group \
  -Wl,-Bstatic -lgmp -lunwind -luv -lc++ -lc++abi -Wl,-Bdynamic \
  -lpthread -ldl -lrt -lm \
  $RPATH_FLAGS \
  -Wl,-rpath,$LEAN_PREFIX/lib \
  -Wl,-rpath,$LEAN_PREFIX/lib/lean \
  -Wl,-rpath,$RUST_LIB \
  -o ../fuzzer/afl_harness/harness_afl
```

## Running the Fuzzer

**Before every run (might be required after reboot):**
```bash
echo core | sudo tee /proc/sys/kernel/core_pattern
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

**Start AFL++ with N instances (1 main + N-1 workers):**
```bash
cd fuzzer/afl_harness/
rm -rf afl_output/

AFL_NO_UI=1 afl-fuzz -i seeds/ -o afl_output/ -M main -t 999999 -- ./harness_afl > /tmp/afl_main.log 2>&1 &
for i in $(seq 1 $((N-1))); do
    AFL_NO_UI=1 afl-fuzz -i seeds/ -o afl_output/ -S worker$i -t 999999 -- ./harness_afl > /tmp/afl_worker$i.log 2>&1 &
done
```

The `-t 999999` timeout is required — the Lean runtime takes 1-2 minutes to initialize on first execution.

**Check status:**
```bash
afl-whatsup fuzzer/afl_harness/afl_output/ | tail -15
```

**Kill all instances:**
```bash
kill $(pgrep -f afl-fuzz)
```

**Analyse a crash:**
```bash
./harness_afl < afl_output/main/crashes/<crash_id>
cat /tmp/afl_mismatch.txt
```

## Results
After 24 hours with 4 parallel workers on a 16-core AMD EPYC server (32 GB RAM):
- 1.5 million test executions
- 0 semantic mismatches
- 83% region coverage of the EPFL sumcheck source (all uncovered regions are dead code for power-of-two evaluation tables)
## Acknowledgements

- [EPFL compsec lab](https://github.com/compsec-epfl/efficient-sumcheck) for the Rust sumcheck implementation
- [z-tech/sumcheck-lean4](https://github.com/z-tech/z-lean) for the formally verified Lean4 oracle with machine-checked proofs of completeness and soundness
