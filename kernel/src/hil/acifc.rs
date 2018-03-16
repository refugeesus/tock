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
    // Test outputs on the ACIFC
    fn test_output(&self) -> ReturnCode;
}
// pub trait Client {
//     /// Called when an interrupt occurs. The `identifier` will
//     /// be the same value that was passed to `enable_interrupt()`
//     /// when the interrupt was configured.
//     fn fired(&self, identifier: usize) -> ReturnCode;
// }