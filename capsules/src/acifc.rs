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
//! Author: Danilo Verhaert <verhaert@cs.stanford.edu>

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
    /// Creat an `ACIFC` driver
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Initialize the ACIFC by activating the clock and the ACIFC itself.
    /// - `2`: Perform a simple comparison. Input x chooses the desired comparator ACx (0 or 1 for hail, 0-3 for imix)
    /// - `3`: Perform a window comparison. Input x chooses the desired window Windowx (0 for hail, 0 or 1 for imix)
    /// - `4`: Test the ACIFC for basic  functionality.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => return ReturnCode::SUCCESS,

            1 => self.acifc.initialize_acifc(),

            2 => ReturnCode::SuccessWithValue {
                value: self.acifc.normal_comparison(data) as usize},

            3 => ReturnCode::SuccessWithValue {
                value: self.acifc.window_comparison(data) as usize},

            4 => self.acifc.test_output(),

            _ => return ReturnCode::ENOSUPPORT,
        }
    }
}
