# Makefile for building the tock kernel for the sensortag development kit

TOCK_ARCH=cortex-m3
TOCK_BOARD=launchxl-cc13x0
TARGET=thumbv7m-none-eabi
PLATFORM=launchxl
OPENOCD ?= openocd
JLINK_EXE ?= JLinkExe
JLINK_DEVICE ?= cc1310f128
JLINK_ARGUMENTS ?= -device $(JLINK_DEVICE) -if jtag -speed 4000 -AutoConnect 1 -jtagconf -1,1

# include ../Makefile.common

TI_BASE="/Applications/ti/ccsv8"
PROG_BASE=0x10000

include ../../tock/boards/Makefile.common

#flash: target/$(TARGET)/release/sensortag
#	$(call check_defined, UNIFLASH_CLI_BASE, See launchxl README for details)
#	@echo "Flashing target/$(TARGET)/release/sensortag"
#	@$(TI_BASE)/ccs_base/DebugServer/bin/DSLite flash -c jtag/cc1310.ccxml -e -f target/$(TARGET)/release/sensortag

flash: target/$(TARGET)/release/launchxl.bin
	$(OPENOCD) -f flash-kernel.openocd

flash-ccfg: target/$(TARGET)/release/launchxlccfg.bin
	$(OPENOCD) -f flash-ccfg.openocd

flash-jlink: target/$(TARGET)/release/launchxl.bin
	$(JLINK_EXE) $(JLINK_ARGUMENTS) -CommanderScript flash-kernel.jlink

flash-ccfg-jlink: target/$(TARGET)/release/launchxlccfg.bin
	$(JLINK_EXE) $(JLINK_ARGUMENTS) -CommanderScript flash-ccfg.jlink
