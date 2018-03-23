//! Interface for direct control of ACIFC
//!
//! Author: Danilo Verhaert <verhaert@stanford.edu>

use returncode::ReturnCode;

/// Enum for selecting which edge to trigger interrupts on.
// #[derive(Debug)]
// pub enum InterruptMode {
//     RisingEdge,
//     FallingEdge,
//     EitherEdge,
// }

pub trait Acifc{
    // Initialize the ACIFC
    fn initialize_acifc(&self) -> ReturnCode;

	// Enable the clock    
    fn enable_clock(&self);

	// Disable the clock
    fn disable_clock(&self);

    // Do a comparison of two inputs. Output will be 1 when Vinp>Vinn (Vin positive > Vin negative) 
    fn normal_comparison(&self, usize) -> u32;

    // Do a comparison of three inputs. Output will be 1 when Vacbn_x+1 < Vcommon < Vacap_x! 
    fn window_comparison(&self, usize) -> u32;

    // Do a basic test to make sure everything is working
    fn test_output(&self) -> ReturnCode;
}

// pub trait Client {
//     /// Called when an interrupt occurs. The `identifier` will
//     /// be the same value that was passed to `enable_interrupt()`
//     /// when the interrupt was configured.
//     fn fired(&self, identifier: usize) -> ReturnCode;
// }