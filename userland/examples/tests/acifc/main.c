#include <stdio.h>

#include <acifc.h>
#include <timer.h>
#include <tock.h>

int main(void) {
  int ret;

  printf("*********************\n");
  printf("ACIFC test application\n");

  // Set mode to which test you want
  uint8_t mode = 1;

  switch (mode) {
    case 1: test_output(); break;
  }

  return 0;
}
