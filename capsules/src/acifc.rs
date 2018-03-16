//! Provides an ACIFC interface for userspace.
//!
//! Usage
//! -----
//!
//! ```rust
//! let acifc = static_init!(
//!     capsules::acifc::Acifc<'static>,
//!     capsules::acifc::Acifc::new(&mut sam4l::acifc::ACIFC));
//! ```

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000007;

use kernel::{AppId, Driver, ReturnCode};
use kernel::hil;

pub struct Acifc<'a, A: hil::acifc::Acifc + 'a> {
    acifc: &'a A,
}

impl<'a, A: hil::acifc::Acifc> Acifc<'a, A> {
    pub fn new(acifc: &'a A) -> Acifc<'a, A> {
        Acifc { acifc: acifc }
    }
}

impl<'a, A: hil::acifc::Acifc> Driver for Acifc<'a, A> {
    /// Control the ACIFC.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Test the ACIFC for basic  functionality.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* Check if exists */ => return ReturnCode::SUCCESS,

            // test the acifc
            1 => self.acifc.test_output(),

           // 2 => self.acifc.fired(data),

            _ => return ReturnCode::ENOSUPPORT,
        }
    }
}
