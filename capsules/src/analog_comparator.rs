//! Provides userspace access to the analog comparators on a board.
//!
//! ## Instantiation
//!
//! ```rust
//! let acifc = static_init!(
//! capsules::analog_comparator::AnalogComparator<'static, sam4l::acifc::Acifc>,
//! capsules::analog_comparator::AnalogComparator::new(&mut sam4l::acifc::ACIFC));
//! ```
//!
//! Author: Danilo Verhaert <verhaert@cs.stanford.edu>

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00007;

use kernel::{AppId, Driver, ReturnCode};
use kernel::hil;

pub struct AnalogComparator<'a, A: hil::analog_comparator::AnalogComparator + 'a> {
    ac: &'a A,
}

impl<'a, A: hil::analog_comparator::AnalogComparator> AnalogComparator<'a, A> {
    pub fn new(ac: &'a A) -> AnalogComparator<'a, A> {
        AnalogComparator { ac: ac }
    }
}

impl<'a, A: hil::analog_comparator::AnalogComparator> Driver for AnalogComparator<'a, A> {
    /// Control the analog comparator.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Initialize the analog comparator by activating the clock and
    ///        the ACIFC itself.
    /// - `2`: Perform a simple comparison.
    ///        Input x chooses the desired comparator ACx (0 or 1 for hail,
    ///        0-3 for imix)
    /// - `3`: Perform a window comparison.
    ///        Input x chooses the desired window Windowx (0 for hail,
    ///        0 or 1 for imix)
    /// - `4`: Test the ACIFC for basic  functionality.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => return ReturnCode::SUCCESS,

            1 => self.ac.initialize_acifc(),

            2 => ReturnCode::SuccessWithValue {
                value: self.ac.comparison(data) as usize,
            },

            3 => ReturnCode::SuccessWithValue {
                value: self.ac.window_comparison(data) as usize,
            },

            _ => return ReturnCode::ENOSUPPORT,
        }
    }
}
