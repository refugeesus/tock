use cortexm4::{self, nvic};
use gpio;
use i2c;
use kernel;
use events;
use rtc;
use uart;
use kernel::common::ring_buffer;
use num_traits::FromPrimitive;


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
use events::NVIC_IRQ;

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

        while let Some(interrupt) = nvic::next_pending() {
            let parse_nvic = events::NVIC_IRQ::from_u32(interrupt);

            if let Some(event) = parse_nvic {
                match event {
                    NVIC_IRQ::GPIO => gpio::PORT.handle_interrupt(),
                    NVIC_IRQ::AON_RTC => rtc::RTC.handle_interrupt(),
                    NVIC_IRQ::UART0 => uart::UART0.handle_interrupt(),
                    //NVIC_IRQ::UART1 => uart::UART1.handle_interrupt(),
                    NVIC_IRQ::I2C => i2c::I2C0.handle_interrupt(),
                    // AON Programmable interrupt
                    // We need to ignore JTAG events since some debuggers emit these
                    NVIC_IRQ::AON_PROG => (),
                    _ => panic!("unhandled interrupt {}", interrupt),
                }
            }
            let n = nvic::Nvic::new(interrupt);
            n.clear_pending();
            n.enable();
        }


        while let Some(event) = events::next_pending() {
            match event {
                    EVENT_PRIORITY::UART1 => {
                        uart::UART1.handle_event()
                    },
                     _ => panic!("unhandled event {:?} ", event),
                    // EVENT_PRIORITY::GPIO  => gpio::PORT.handle_interrupt(),
                    // EVENT_PRIORITY::RTC   => rtc::RTC.handle_interrupt(),
                    // EVENT_PRIORITY::AON_RTC => rtc::RTC.handle_interrupt(),
                    // EVENT_PRIORITY::UART0 => uart::UART0.handle_interrupt(),

                    // EVENT_PRIORITY::I2C0 => i2c::I2C0.handle_interrupt(),
                    // EVENT_PRIORITY::AON_PROG => (),
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
