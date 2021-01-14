#include "stdio.h"
#include "stdint.h"

uint64_t square(uint64_t value) {
    return value * value;
}

void square_all(uint64_t length, double* input, double* output) {
    for (uint64_t i=0; i<length; ++i) {
        output[i] = input[i] * input[i];
    }
}
