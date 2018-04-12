#include "analog_comparator.h"
#include "tock.h"

int acifc_exists(void) {
  return command(DRIVER_NUM_ACIFC, 0, 0, 0) >= 0;
}

int initialize_acifc(void) {
  return command(DRIVER_NUM_ACIFC, 1, 0, 0);
}

bool comparison(uint8_t ac){
  return command(DRIVER_NUM_ACIFC, 2, ac, 0);
}

bool window_comparison(uint8_t window){
  return command(DRIVER_NUM_ACIFC, 3, window, 0);
}
