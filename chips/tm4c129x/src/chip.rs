//! Interrupt mapping and DMA channel setup.

use cortexm4;
use gpio;
use kernel::Chip;
use helpers::{DeferredCall, Task};
use gpt;
use uart;


pub struct Tm4c129x {
    pub mpu: cortexm4::mpu::MPU,
    pub systick: cortexm4::systick::SysTick,
}

impl Tm4c129x {
    pub unsafe fn new() -> Tm4c129x {
      
        Tm4c129x {
            mpu: cortexm4::mpu::MPU::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Tm4c129x {
    type MPU = cortexm4::mpu::MPU;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&mut self) {
        use nvic::*;
		
		unsafe {
            while let Some(interrupt) = cortexm4::nvic::next_pending() {
                match interrupt {
		                     
		                    UART7 => uart::UART7.handle_interrupt(),
		                    TIMER0A => gpt::TIMER0.handle_interrupt(),
		                    _ => {
				                   //panic!("unhandled interrupt {}", interrupt); 
		                    }
		                }
			            let n = cortexm4::nvic::Nvic::new(interrupt);
		                n.clear_pending();
		                n.enable();
		                
		            } 
		}
    }
    
    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4::nvic::has_pending()}
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn systick(&self) -> &cortexm4::systick::SysTick {
        &self.systick
    }
    
}
