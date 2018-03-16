#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_ACIFC 0x7

// Initialize and enable the DAC.
int test_output(void);

#ifdef __cplusplus
}
#endif
