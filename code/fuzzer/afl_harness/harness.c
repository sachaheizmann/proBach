#include <lean/lean.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <sys/resource.h>

struct timespec t0, t1, t2, t3;
#include <time.h>


// ─── Constants ───────────────────────────────────────────────────────────────

static const uint64_t MODULI[5] = {
    19ULL,
    2147483647ULL,
    2013265921ULL,
    2130706433ULL,
    18446744069414584321ULL,
};

#define MAX_N     24 // maximum number of variables
#define MAX_TERMS 256  // maximum number of monomials in a polynomial

#define MAX_BUF   4096  // 2 + 256*(8 + 3) + 8 = 2826 bytes max at n=24

// Macros for manual testing/compiling
#ifndef __AFL_INIT
#define __AFL_INIT() do {} while(0)
#endif

#ifndef __AFL_LOOP
#define __AFL_LOOP(x) (1)
#endif

// ─── Init Lean runtime + functions ────────────────────────────────────────────────────────────

void lean_initialize_runtime_module();
void lean_io_mark_end_initialization();
lean_object* initialize_differential__testing_SumcheckFFI(uint8_t builtin);

lean_object* lean_compute_transcript_z19(lean_object* n, lean_object* terms, lean_object* challenges);
lean_object* lean_compute_transcript_m31(lean_object* n, lean_object* terms, lean_object* challenges);
lean_object* lean_compute_transcript_babybear(lean_object* n, lean_object* terms, lean_object* challenges);
lean_object* lean_compute_transcript_koalabear(lean_object* n, lean_object* terms, lean_object* challenges);
lean_object* lean_compute_transcript_goldilocks(lean_object* n, lean_object* terms, lean_object* challenges);

// ─── Rust FFI ────────────────────────────────────────────────────────────────

uint32_t rust_run_sumcheck(
    uint8_t  field_id,
    uint32_t n,
    uint32_t num_terms,
    const uint64_t* coeffs,
    const uint8_t*  exps,
    uint64_t seed,
    uint64_t* out_buf
);

// ─── Lean helpers ────────────────────────────────────────────────────────────


// builds a Lean List Nat from a C array of uint64_t values, used to pass challenges to Lean
static lean_object* make_nat_list(uint64_t* vals, int len) {
    lean_object* list = lean_box(0);
    for (int i = len - 1; i >= 0; i--) {
        lean_object* cell = lean_alloc_ctor(1, 2, 0);
        lean_ctor_set(cell, 0, lean_uint64_to_nat(vals[i]));
        lean_ctor_set(cell, 1, list);
        list = cell;
    }
    return list;
}

// builds a single Lean term, (Nat x List Nat) pair representing one monomial
static lean_object* make_term(uint64_t coeff, uint8_t* exps, int n) {
    lean_object* exp_list = lean_box(0);
    for (int i = n - 1; i >= 0; i--) {
        lean_object* cell = lean_alloc_ctor(1, 2, 0);
        lean_ctor_set(cell, 0, lean_unsigned_to_nat(exps[i]));
        lean_ctor_set(cell, 1, exp_list);
        exp_list = cell;
    }
    lean_object* pair = lean_alloc_ctor(0, 2, 0);
    lean_ctor_set(pair, 0, lean_unsigned_to_nat(coeff));
    lean_ctor_set(pair, 1, exp_list);
    return pair;
}
static lean_object* make_uint64_list(uint64_t* vals, int len) {
        lean_object* list = lean_box(0);
        for (int i = len - 1; i >= 0; i--) {
            lean_object* cell = lean_alloc_ctor(1, 2, 0);
            lean_ctor_set(cell, 0, lean_box_uint64(vals[i]));
            lean_ctor_set(cell, 1, list);
            list = cell;
        }
        return list;
    }

// Build flat eval table: 2^n entries, one per boolean point (MSB order)
// Mirrors Rust's all_boolean_points + evaluation loop in the macro
static lean_object* make_eval_table(
    uint64_t* coeffs, uint8_t exps[][MAX_N],
    int num_terms, int n, uint64_t modulus)
{
    int table_size = 1 << n;  // 2^n
    lean_object* list = lean_box(0);  // start with empty list, build backwards

    for (int i = table_size - 1; i >= 0; i--) {
        // boolean point for index i under MSB convention
        // bit j of the point = bit (n-1-j) of i
        uint64_t val = 0;
        for (int t = 0; t < num_terms; t++) {
            uint64_t term_val = coeffs[t] % modulus;
            for (int v = 0; v < n; v++) {
                int bit = (i >> (n - 1 - v)) & 1;  // MSB convention
                if (exps[t][v] > 0 && bit == 0) {
                    term_val = 0;
                    break;
                }
            }
            val = ((__uint128_t)val + term_val) % modulus;
        }

        // prepend to Lean list
        lean_object* cell = lean_alloc_ctor(1, 2, 0);
        lean_ctor_set(cell, 0, lean_box_uint64(val));
        lean_ctor_set(cell, 1, list);
        list = cell;
    }
    return list;
}

// ─── Test case ───────────────────────────────────────────────────────────────

typedef struct {
    uint8_t  field_id;
    uint8_t  n;
    uint8_t  num_terms;
    uint64_t coeffs[MAX_TERMS];
    uint8_t  exps[MAX_TERMS][MAX_N];
    uint64_t seed;
} TestCase;

// ─── Dedup ───────────────────────────────────────────────────────────────────

// sums monomials with identical exponents 
static int dedup_terms(uint64_t *coeffs, uint8_t exps[][MAX_N], int num_terms, int n, uint64_t modulus) {
    int new_count = 0;
    for (int i = 0; i < num_terms; i++) {
        int found = -1;
        for (int j = 0; j < new_count; j++) {
            int match = 1;
            for (int v = 0; v < n; v++) {
                if (exps[j][v] != exps[i][v]) { match = 0; break; }
            }
            if (match) { found = j; break; }
        }
        if (found >= 0) {
            coeffs[found] = (coeffs[found] + coeffs[i]) % modulus;
        } else {
            coeffs[new_count] = coeffs[i] % modulus;
            memcpy(exps[new_count], exps[i], MAX_N);
            new_count++;
        }
    }
    int final_count = 0;
    for (int i = 0; i < new_count; i++) {
        if (coeffs[i] != 0) {
            coeffs[final_count] = coeffs[i];
            memcpy(exps[final_count], exps[i], MAX_N);
            final_count++;
        }
    }
    if (final_count == 0) {
        coeffs[0] = 1;
        memset(exps[0], 0, MAX_N);
        final_count = 1;
    }
    return final_count;
}

// ─── Parse binary input ──────────────────────────────────────────────────────

static int parse_bytes(const uint8_t *buf, size_t len, TestCase *tc) {
    if (len < 18) return -1;
    size_t pos = 0;

    tc->field_id = buf[pos++] % 5;
    tc->n        = (buf[pos++] % MAX_N) + 1;

    uint64_t modulus = MODULI[tc->field_id];
    int exp_bytes    = (tc->n + 7) / 8;
    int term_size    = 8 + exp_bytes;

    // parse as many complete monomials as fit, up to MAX_TERMS
    size_t remaining = (len >= 8 + pos) ? len - 8 - pos : 0;
    int num_terms    = (int)(remaining / term_size);
    if (num_terms > MAX_TERMS) num_terms = MAX_TERMS;
    if (num_terms == 0) num_terms = 1;

    for (int t = 0; t < num_terms; t++) {
        uint64_t coeff = 0;
        for (int b = 0; b < 8; b++)
            coeff |= ((uint64_t)(pos < len ? buf[pos++] : 0)) << (8 * b);
        tc->coeffs[t] = coeff % modulus;

        for (int v = 0; v < tc->n; v++) {
            int byte_idx = v / 8;
            int bit_idx  = v % 8;
            uint8_t byte = (pos + byte_idx < len - 8) ? buf[pos + byte_idx] : 0;
            tc->exps[t][v] = (byte >> bit_idx) & 1;
        }
        pos += exp_bytes;
    }

    tc->num_terms = num_terms;

    if (len >= 8) memcpy(&tc->seed, buf + len - 8, 8);
    else tc->seed = 42;

    tc->num_terms = dedup_terms(tc->coeffs, tc->exps, tc->num_terms, tc->n, modulus);
    return 0;
}

// ─── Lean call ───────────────────────────────────────────────────────────────

// array of function pointers depending on field_id
typedef lean_object* (*lean_transcript_fn)(lean_object*, lean_object*, lean_object*);
static lean_transcript_fn LEAN_FNS[5] = {
    lean_compute_transcript_z19,
    lean_compute_transcript_m31,
    lean_compute_transcript_babybear,
    lean_compute_transcript_koalabear,
    lean_compute_transcript_goldilocks,
};

static int run_lean(const TestCase* tc, const uint64_t* rust_out,
                    uint64_t* lean_s0, uint64_t* lean_final) {
    int n = tc->n;

    // extract challenges from rust output ,they are at offset n -> 2n 
    // in rust_out
    uint64_t challenges[MAX_N];
    for (int i = 0; i < n; i++)
        challenges[i] = rust_out[n + i];
    // time our wrapper (building Lean objects)
    clock_gettime(CLOCK_MONOTONIC, &t0);

    lean_object* ln     = lean_unsigned_to_nat(n);
    // for each term: build (coeff, [exp0, exp1, ...]) pair
    lean_object* lterms = make_eval_table(
        (uint64_t*)tc->coeffs,
        (uint8_t(*)[MAX_N])tc->exps,
        tc->num_terms, n, MODULI[tc->field_id]);

    
    // challenges extracted from rust_out[n..2n]                                      
    lean_object* lchallenges = make_uint64_list(challenges, n);
     
    clock_gettime(CLOCK_MONOTONIC, &t1);
    // call filed_id corresponding function
    // returns a Lean pair (List UInt64, List (UInt64 × UInt64))
    clock_gettime(CLOCK_MONOTONIC, &t2);
    lean_object* result = LEAN_FNS[tc->field_id](ln, lterms, lchallenges);
    clock_gettime(CLOCK_MONOTONIC, &t3);
    double wrapper_time = (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec) / 1e9;
    double library_time = (t3.tv_sec - t2.tv_sec) + (t3.tv_nsec - t2.tv_nsec) / 1e9;
    fprintf(stderr, "Lean wrapper:  %.6fs\n", wrapper_time);
    fprintf(stderr, "Lean library:  %.6fs\n", library_time);

    // extract s0 list (first element of pair)
    lean_object* s0_list = lean_ctor_get(result, 0);
    lean_inc(s0_list);

    // walk s0 list
    lean_object* cur = s0_list;
    int idx = 0;
    while (lean_obj_tag(cur) != 0) {
        lean_object* head = lean_ctor_get(cur, 0);
        lean_s0[idx++] = lean_unbox_uint64(head);
        cur = lean_ctor_get(cur, 1);
    }

    // extract final value (second element of pair) - it's a boxed UInt64
    lean_object* final_obj = lean_ctor_get(result, 1);
    *lean_final = lean_unbox_uint64(final_obj);

    lean_dec_ref(result);
    lean_dec_ref(s0_list);
    return 0;

}

// ─── Compare ─────────────────────────────────────────────────────────────────

static int compare_outputs(const uint64_t* rust_out, uint32_t rust_len,
                            const uint64_t* lean_s0,
                            const uint64_t lean_final, int n) {
    // compare s0 per round
    for (int i = 0; i < n; i++) {
        if (rust_out[i] != lean_s0[i]) return 0;
    }

    // compare final value
    // rust_out[2n] = final value
    if (rust_out[2*n] != lean_final) return 0;

    return 1;
}

// ─── Main ────────────────────────────────────────────────────────────────────

int main(void) {
    // increase stack size to unlimited to prevent stack overflow at n >= 17
    struct rlimit rl;
    getrlimit(RLIMIT_STACK, &rl);
    rl.rlim_cur = RLIM_INFINITY;
    setrlimit(RLIMIT_STACK, &rl);

    // initialize Lean runtime once
    lean_initialize_runtime_module();
    lean_object* res = initialize_differential__testing_SumcheckFFI(1);
    if (lean_io_result_is_ok(res)) { 
        lean_dec_ref(res);
    }
    else { 
        lean_io_result_show_error(res);
        lean_dec(res);
        return 1; 
    }
    lean_io_mark_end_initialization();

    // defer AFL++ fork until after Lean init
    __AFL_INIT();

    // AFL++ persistent loop
    while (__AFL_LOOP(100000)) {
        // read input
        uint8_t buf[MAX_BUF];
        size_t len = fread(buf, 1, sizeof(buf), stdin);
        if (len == 0) break;

        TestCase tc;
        if (parse_bytes(buf, len, &tc) != 0) continue;

        // flatten exps array for Rust FFI
        uint8_t flat_exps[MAX_TERMS * MAX_N];
        for (int t = 0; t < tc.num_terms; t++)
            for (int v = 0; v < tc.n; v++)
                flat_exps[t * tc.n + v] = tc.exps[t][v];

        // run Rust via direct FFI call
        uint64_t rust_out[2 * MAX_N + 1] = {0};
        uint32_t rust_len = rust_run_sumcheck(
            tc.field_id, tc.n, tc.num_terms,
            tc.coeffs, flat_exps, tc.seed,
            rust_out
        );
        if (rust_len == 0) abort();

        // run Lean via FFI
        uint64_t lean_s0[MAX_N] = {0};
        uint64_t lean_final = 0;
        run_lean(&tc, rust_out, lean_s0, &lean_final);

        // compare
        if (!compare_outputs(rust_out, rust_len, lean_s0, lean_final, tc.n)) {
            FILE* f = fopen("/tmp/afl_mismatch.txt", "w");
            if (f) {
                fprintf(f, "FIELD: %d N: %d\n", tc.field_id, tc.n);
                fprintf(f, "RUST s0: ");
                for (int i = 0; i < tc.n; i++) fprintf(f, "%lu ", rust_out[i]);
                fprintf(f, "\nLEAN s0: ");
                for (int i = 0; i < tc.n; i++) fprintf(f, "%lu ", lean_s0[i]);
                fprintf(f, "\nRUST final: %lu\n", rust_out[2*tc.n]);
                fprintf(f, "LEAN final: %lu\n", lean_final);
                fclose(f);
            }
            abort();
        }
    }
    return 0;
}
