use cortexm4::{self, nvic};
use gpio;
use i2c;
use kernel;
use events;
use rtc;
use uart;

pub struct Cc26X2 {
    mpu: cortexm4::mpu::MPU,
    systick: cortexm4::systick::SysTick,
}


impl Cc26X2 {
    pub unsafe fn new() -> Cc26X2 {
        Cc26X2 {
            mpu: cortexm4::mpu::MPU::new(),
            // The systick clocks with 48MHz by default
            systick: cortexm4::systick::SysTick::new_with_calibration(48 * 1000000),
        }
    }
}


use events::EVENT_PRIORITY;

impl kernel::Chip for Cc26X2 {
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

        while let Some(event) = events::next_pending() {
            match event {
                //TODO
                EVENT_PRIORITY::GPIO => gpio::PORT.handle_interrupt(),
                EVENT_PRIORITY::AON_RTC => rtc::RTC.handle_interrupt(),
                EVENT_PRIORITY::I2C0 => i2c::I2C0.handle_interrupt(),
                //TODO correct these to fit the new model
                EVENT_PRIORITY::UART0 => uart::UART0.handle_event(),
                EVENT_PRIORITY::UART1 => uart::UART1.handle_event(),
                EVENT_PRIORITY::AON_PROG => (),
                 _ => panic!("unhandled event {:?} ", event),
            }
        }
    }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() }
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
