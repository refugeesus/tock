#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_ACIFC 0x7

// Does the driver exist?
int acifc_exists(void);

// Initialize and enable the DAC.
int initialize_acifc(void);

// Comparing the voltages of two pins (if one is higher than the other)
bool comparison(uint8_t);

// Compare the voltages of three pins (if one is between the other two)
bool window_comparison(uint8_t);

#ifdef __cplusplus
}
#endif
