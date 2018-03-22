//! Provides userspace access to the ACIFC interface.
//!
// !## Instantiation
//! -----
//!
//! ```rust
//! let acifc = static_init!(
//! capsules::acifc::Acifc<'static, sam4l::acifc::Acifc>, 
//! capsules::acifc::Acifc::new(&mut sam4l::acifc::ACIFC);
//! ```
//!
//! Author: Danilo Verhaert <verhaert@stanford.edu>

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x07;

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
    /// - `1`: Initialize the ACIFC by activating the clock and the ACIFC itself.
    /// - `2`: Perform a simple comparison. Input is the desired comparator (0 or 1)
    /// - `3`: Test the ACIFC for basic  functionality.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => return ReturnCode::SUCCESS,

            1 => self.acifc.initialize_acifc(),

            2 => self.acifc.comparison(data),

            3 => self.acifc.test_output(),

            _ => return ReturnCode::ENOSUPPORT,
        }
    }
}
