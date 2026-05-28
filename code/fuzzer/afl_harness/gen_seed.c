#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

int main(int argc, char** argv) {
    int n        = argc > 1 ? atoi(argv[1]) : 16;
    int field_id = argc > 2 ? atoi(argv[2]) : 2;  // babybear default

    uint8_t buf[256] = {0};
    int pos = 0;

    buf[pos++] = (uint8_t)field_id;
    buf[pos++] = (uint8_t)(n - 1);  // parse does % MAX_N + 1
    buf[pos++] = 0;                 // num_terms - 1 = 0 → 1 term

    // coefficient = 1 (8 bytes LE)
    uint64_t coeff = 1;
    memcpy(buf + pos, &coeff, 8); pos += 8;

    // exponents: all zero, bit-packed
    int exp_bytes = (n + 7) / 8;
    memset(buf + pos, 0, exp_bytes); pos += exp_bytes;

    // seed (last 8 bytes)
    uint64_t seed = 42;
    memcpy(buf + pos, &seed, 8); pos += 8;

    fwrite(buf, 1, pos, stdout);
    return 0;
}
