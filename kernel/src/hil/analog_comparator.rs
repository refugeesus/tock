//! Interface for direct control of the analog comparators.
//!
//! Author: Danilo Verhaert <verhaert@cs.stanford.edu>

use returncode::ReturnCode;

pub trait AnalogComparator {
    /// Initialize the ACIFC by enabling the clock, activating the ACs (Analog
    /// Comparators). Currently in initialization always-on mode is enabled,
    /// allowing a measurement on an AC to be made quickly after a measurement
    /// is triggered, without waiting for the AC startup time. The drawback is
    /// that when the AC is always on the power dissipation will be higher.
    fn initialize_acifc(&self) -> ReturnCode;

    /// Do a comparison of two inputs. Output will be 1 when one is higher than
    /// the other, and 0 otherwise. Specifically, the output is 1 when Vinp>Vinn
    /// (Vin positive > Vin negative), and 0 if Vinp < Vinn.
    fn comparison(&self, usize) -> bool;

    /// Do a comparison of three input voltages. Two ACs, ACx and ACx+1 are
    /// grouped for this comparison. The sources of the negative input of ACx
    /// (Vn_x) and the positive input of ACx+1 (Vp_x+1) must be connected
    /// together externally. These form the common voltage Vcommon. The other
    /// two remaining sources, being the positive input of ACx (Vp_x) and
    /// negative input of ACx+1 (Vn_x+1) then define an upper and a lower bound
    /// of a window. The result then depends on Vcommon lying inside of outside
    /// of this window. When the voltage of Vcommon lies inside the window
    /// defined by the positive input of ACx and the negative input of ACx+1,
    /// the output will be 1; it will be 0 if it is outside of the window.
    /// Specifically, the output will be 1 when Vn_x+1 < Vcommon < Vp_x, and 0
    /// if Vcommon < Vn_x+1 or Vcommon > Vp_x.
    fn window_comparison(&self, usize) -> bool;
}
