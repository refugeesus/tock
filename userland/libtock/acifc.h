#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_ACIFC 0x7

// Initialize and enable the DAC.
int initialize_acifc(void);

// Comparing the voltages of two pins
int comparison(uint8_t);

// Basic test function
int test_output(void);

#ifdef __cplusplus
}
#endif
