#include "acifc.h"
#include "tock.h"

int initialize_acifc(void) {
  return command(DRIVER_NUM_ACIFC, 1, 0, 0);
}

int comparison(uint8_t ac){
  return command(DRIVER_NUM_ACIFC, 2, ac, 0);
}

int test_output(void) {
  return command(DRIVER_NUM_ACIFC, 3, 0, 0);
}
