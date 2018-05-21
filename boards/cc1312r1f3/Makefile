# Makefile for building the tock kernel for the launchxl development kit

TOCK_ARCH=cortex-m4
TOCK_BOARD=cc1312r1f3
TARGET=thumbv7em-none-eabi
PLATFORM=cc1312r1f3

include ../Makefile.common

flash: target/$(TARGET)/release/cc1312r1f3 
	$(call check_defined, UNIFLASH_CLI_BASE, See launchxl README for details)
	@echo "Flashing target/$(TARGET)/release/launchxl"
	@$(UNIFLASH_CLI_BASE)/ccs_base/DebugServer/bin/DSLite flash -c jtag/cc1312r1f3.ccxml -e -f target/$(TARGET)/release/cc1312r1f3

