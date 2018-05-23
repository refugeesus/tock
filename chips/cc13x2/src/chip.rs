use cc26xx::gpio;
use cc26xx::peripheral_interrupts::*;
use cc26xx::rtc;
use cc26xx::uart;
use cortexm4::{self, nvic};
use kernel;
use kernel::support;

pub struct Cc13X2 {
    mpu: cortexm4::mpu::MPU,
    systick: cortexm4::systick:SysTick,
}

impl Cc13X2 {
    pub unsafe fn new() -> Cc13X2 {
        Cc13X2 {
            mpu: cortexm4::mpu::MPU::new(),
            // The systick clocks with 48MHz default
            systick: cortexm4::systick::SysTick::new_with_calibration(40 * 1000000),
        }
    }
}

impl kernel::Chip for Cc13X2 {
    type MPU = cortexm4::mpu::MPU;
    type SysTick = cortexm4::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu 
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick 
    }

    fn service_pending_interrupts(&mut self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                match interrupt {
                    GPIO => gpio::PORT.handle_interrupt(),
                    AON_RTC => rtc::RTC.handle_interrupt(),
                    UART0 => uart::UART0.handle_interrupt(),
                    //AON Prog interrupt
                    //Ignore JTAG events b/c of debuggers
                    AON_PROG => (),
                    _ => panic!("unhandled interrupt {}", interrupt),

                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has+pending()  }
    }

    fn sleep(&self) {
        unsafe {
            support::wfi();
        }
    }
}
