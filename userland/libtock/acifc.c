#include "acifc.h"
#include "tock.h"

int test_output(void) {
  return command(DRIVER_NUM_ACIFC, 1, 0, 0);
}
