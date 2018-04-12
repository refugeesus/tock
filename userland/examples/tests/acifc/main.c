#include <stdio.h>
#include <stdlib.h>

#include <analog_comparator.h>
#include <timer.h>
#include <tock.h>

static void comparison_polling(uint8_t ac) {
  uint count = 0;
  if (ac > 3) {
    printf("Please choose either ac 0 or 1 for hail, and 2 or 3 for imix.");
  }else {
    while (1) {
      count++;
      bool result = comparison(ac);
      printf("Try %d. Result = %d.\n", count, result);
      if (result == 1) {
        printf("This implies Vinp > Vinn!\n\n");
      }else {
        printf("This implies Vinp < Vinn!\n\n");
      }
      delay_ms(1000);
    }
  }
}

static void window_comparison_polling(uint8_t window) {
  uint count = 0;
  if (window > 1) {
    printf("Please choose window 0 for hail and 1 for imix.");
  }else {
    while (1) {
      count++;
      bool result = window_comparison(window);
      printf("Try %d. Result = %d.\n", count, result);
      if (result == 1) {
        printf("This implies Vacbn_x+1 < Vcommon < Vacap_x!\n\n");
      }else {
        printf("This implies Vcommon < Vacan_x+1 or Vcommon > Vacap_x\n\n");
      }
      delay_ms(1000);
    }
  }
}

int main(void) {
  printf("\nACIFC test application\n");

  if (!acifc_exists()) {
    printf("ACIFC driver does not exist\n");
    exit(1);
  }

  // Set mode according to which implementation you want
  uint8_t mode = 0;

  // Choose your comparator. AC = 0 corresponds to PA06 and PA07, whereas ac = 1
  // corresponds to PB02 and PB03. On the hail these are the pins DAC and WKP,
  // and AC2 and AC3 respectively.
  uint8_t ac = 1;

  // Choose your window. For the hail, there is only one window. For imix, there
  // are two (0 and 1).
  uint8_t window = 1;

  // Initialize the ACIFC by enabling some basic necessities
  initialize_acifc();

  switch (mode) {
    // Poll for a normal comparison every second and print the result
    case 0: comparison_polling(ac); break;

    // Poll for a window comparison every second and print the result
    case 1: window_comparison_polling(window); break;

  }
  printf("\n");
  return 0;
}

