#include <stdio.h>

#include <acifc.h>
#include <timer.h>
#include <tock.h>

static void comparison_polling(uint8_t ac) {
  uint count = 0;
  while(1){
    count++;
    printf("Try %d. \n", count);
    comparison(ac);
    delay_ms(1000);
  }
}

int main(void) {
  printf("*********************\n");
  printf("ACIFC test application\n");

  // Set mode to which test you want
  uint8_t mode = 1;
  uint8_t ac = 0;

  initialize_acifc();

  switch (mode) {
    case 0: comparison_polling(ac); break;
    case 1: test_output(); break;
  }
  printf("*********************\n");
  return 0;
}
