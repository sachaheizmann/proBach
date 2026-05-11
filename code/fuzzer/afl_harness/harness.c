#include <lean/lean.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>

// ─── Constants ───────────────────────────────────────────────────────────────

static const uint64_t MODULI[5] = {
    19ULL,
    2147483647ULL,
    2013265921ULL,
    2130706433ULL,
    18446744069414584321ULL,
};

#define MAX_N     24
#define MAX_TERMS 8
#define MAX_OUT   65536
#define MAX_BUF   4096

#ifndef __AFL_INIT
#define __AFL_INIT() do {} while(0)
#endif

#ifndef __AFL_LOOP
#define __AFL_LOOP(x) (1)
#endif

// ─── Lean runtime ────────────────────────────────────────────────────────────

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

static lean_object* make_terms_list(uint64_t* coeffs, uint8_t exps[][MAX_N], int num_terms, int n) {
    lean_object* list = lean_box(0);
    for (int i = num_terms - 1; i >= 0; i--) {
        lean_object* term = make_term(coeffs[i], exps[i], n);
        lean_object* cell = lean_alloc_ctor(1, 2, 0);
        lean_ctor_set(cell, 0, term);
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
    if (len < 11) return -1;
    size_t pos = 0;
    tc->field_id  = buf[pos++] % 5;
    tc->n         = (buf[pos++] % MAX_N) + 1;
    tc->num_terms = (buf[pos++] % MAX_TERMS) + 1;
    uint64_t modulus = MODULI[tc->field_id];
    for (int t = 0; t < tc->num_terms; t++) {
        if (pos >= len) {
            tc->num_terms = t == 0 ? 1 : t;
            if (t == 0) { tc->coeffs[0] = 1; memset(tc->exps[0], 0, MAX_N); }
            break;
        }
        tc->coeffs[t] = buf[pos++] % modulus;
        for (int v = 0; v < tc->n; v++)
            tc->exps[t][v] = (pos < len) ? (buf[pos++] & 1) : 0;
    }
    if (len >= 8) memcpy(&tc->seed, buf + len - 8, 8);
    else tc->seed = 42;
    tc->num_terms = dedup_terms(tc->coeffs, tc->exps, tc->num_terms, tc->n, modulus);
    return 0;
}

// ─── Lean call ───────────────────────────────────────────────────────────────

typedef lean_object* (*lean_transcript_fn)(lean_object*, lean_object*, lean_object*);

static lean_transcript_fn LEAN_FNS[5] = {
    lean_compute_transcript_z19,
    lean_compute_transcript_m31,
    lean_compute_transcript_babybear,
    lean_compute_transcript_koalabear,
    lean_compute_transcript_goldilocks,
};

static int run_lean(const TestCase* tc, const uint64_t* rust_out,
                    uint64_t* lean_claims, uint64_t* lean_rounds) {
    int n = tc->n;

    // challenges are at offset n+1+2n = 3n+1 in rust_out
    uint64_t chals[MAX_N];
    for (int i = 0; i < n; i++)
        chals[i] = rust_out[n + 1 + 2*n + i];

    lean_object* ln     = lean_unsigned_to_nat(n);
    lean_object* lterms = make_terms_list((uint64_t*)tc->coeffs,
                                          (uint8_t(*)[MAX_N])tc->exps,
                                          tc->num_terms, n);
    lean_object* lchals = make_nat_list(chals, n);

    lean_object* result = LEAN_FNS[tc->field_id](ln, lterms, lchals);

    lean_object* claims_list = lean_ctor_get(result, 0);
    lean_object* rounds_list = lean_ctor_get(result, 1);
    lean_inc(claims_list);
    lean_inc(rounds_list);

    lean_object* cur = claims_list;
    int idx = 0;
    while (lean_obj_tag(cur) != 0) {
        lean_object* head = lean_ctor_get(cur, 0);
        lean_claims[idx++] = lean_unbox_uint64(head);
        cur = lean_ctor_get(cur, 1);
    }

    cur = rounds_list;
    idx = 0;
    while (lean_obj_tag(cur) != 0) {
        lean_object* pair = lean_ctor_get(cur, 0);
        // UInt64 x UInt64 pair: 2 pointer fields (boxed UInt64)
        lean_rounds[idx++] = lean_unbox_uint64(lean_ctor_get(pair, 0));
        lean_rounds[idx++] = lean_unbox_uint64(lean_ctor_get(pair, 1));
        cur = lean_ctor_get(cur, 1);
    }

    lean_dec_ref(result);
    lean_dec_ref(claims_list);
    lean_dec_ref(rounds_list);
    return 0;
}

// ─── Compare ─────────────────────────────────────────────────────────────────

static int compare_outputs(const uint64_t* rust_out, uint32_t rust_len,
                            const uint64_t* lean_claims,
                            const uint64_t* lean_rounds, int n) {
    for (int i = 0; i <= n; i++) {
        if (rust_out[i] != lean_claims[i]) return 0;
    }
    for (int i = 0; i < 2*n; i++) {
        if (rust_out[n+1+i] != lean_rounds[i]) return 0;
    }
    return 1;
}

// ─── Main ────────────────────────────────────────────────────────────────────

int main(void) {
    // initialize Lean runtime once
    lean_initialize_runtime_module();
    lean_object* res = initialize_differential__testing_SumcheckFFI(1);
    if (lean_io_result_is_ok(res)) { lean_dec_ref(res); }
    else { lean_io_result_show_error(res); lean_dec(res); return 1; }
    lean_io_mark_end_initialization();

    // defer AFL++ fork until after Lean init
    __AFL_INIT();

    // AFL++ persistent loop
    while (__AFL_LOOP(100000)) {
        uint8_t buf[MAX_BUF];
        size_t len = fread(buf, 1, sizeof(buf), stdin);
        if (len == 0) break;

        TestCase tc;
        if (parse_bytes(buf, len, &tc) != 0) continue;

        // flat exps array for Rust FFI
        uint8_t flat_exps[MAX_TERMS * MAX_N];
        for (int t = 0; t < tc.num_terms; t++)
            for (int v = 0; v < tc.n; v++)
                flat_exps[t * tc.n + v] = tc.exps[t][v];

        // run Rust via direct FFI call
        uint64_t rust_out[4 * MAX_N + 1] = {0};
        uint32_t rust_len = rust_run_sumcheck(
            tc.field_id, tc.n, tc.num_terms,
            tc.coeffs, flat_exps, tc.seed,
            rust_out
        );
        if (rust_len == 0) continue;

        // run Lean via FFI
        uint64_t lean_claims[MAX_N + 1] = {0};
        uint64_t lean_rounds[2 * MAX_N] = {0};
        if (run_lean(&tc, rust_out, lean_claims, lean_rounds) != 0) continue;

        // compare
        if (!compare_outputs(rust_out, rust_len, lean_claims, lean_rounds, tc.n)) {
            FILE* f = fopen("/tmp/afl_mismatch.txt", "w");
            if (f) {
                fprintf(f, "FIELD: %d N: %d\n", tc.field_id, tc.n);
                fprintf(f, "RUST claims: ");
                for (int i = 0; i <= tc.n; i++) fprintf(f, "%lu ", rust_out[i]);
                fprintf(f, "\nLEAN claims: ");
                for (int i = 0; i <= tc.n; i++) fprintf(f, "%lu ", lean_claims[i]);
                fprintf(f, "\n");
                fclose(f);
            }
            abort();
        }
    }
    return 0;
}
