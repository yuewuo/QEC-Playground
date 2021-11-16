#include "stdio.h"
#include "stdint.h"

extern "C" {
    uint64_t square(uint64_t value);
    void square_all(uint64_t length, double* input, double* output);
}

uint64_t square(uint64_t value) {
    return value * value;
}

void square_all(uint64_t length, double* input, double* output) {
    for (uint64_t i=0; i<length; ++i) {
        output[i] = input[i] * input[i];
    }
}
