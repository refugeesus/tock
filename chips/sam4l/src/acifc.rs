//! Implementation of the ACIFC controller.
//!
//! See datasheet section "37. Analog Comparator Interface (ACIFC)".
//!
//! The Analog Comparator Interface (ACIFC) controls a number of Analog
//! Comparators (ACs) with identical behavior. Each Analog Comparator compares
//! two voltages and gives an output depending on this comparison. A specific AC
//! is referred to as ACx where x is any number from 0 to n and n is the index
//! of last AC module. The ACIFC on the SAM4L supports a total of 8 ACs (and
//! therefore 4 possible ACWs). However, note that the 64 pin SAM4L (e.g. on the
//! Hail) has 2 ACs (ACA0 and ACB0), and the 100 pin SAM4L (e.g. on the Imix)
//! has 4 ACs (ACA0, ACB0, ACA1, ACB1). Currently, no version of the SAM4L
//! exists with all the 8 ACs implemented. Therefore a lot of the defined
//! bitfields remain unused, but are initialized for a possible future scenario.
//! The ACIFC can be configured in normal mode using each comparator
//! independently or in window mode using defined comparator pairs (ACx and
//! ACx+1) to observe a window.
//!
//! Author: Danilo Verhaert <verhaert@cs.stanford.edu>

// TODO:
// - Implement handling of interrupts
// - Implement other modes, e.g. user and peripheral triggered comparison

use kernel::{ReturnCode, StaticRef};
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::hil;
use pm;

#[repr(C)]
pub struct AcifcRegisters {
    ctrl: ReadWrite<u32, Control::Register>,
    sr: ReadOnly<u32, Status::Register>,
    _reserved0: [ReadOnly<u32>; 2],
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
    icr: WriteOnly<u32, Interrupt::Register>,
    tr: ReadWrite<u32, Test::Register>,
    _reserved1: [ReadOnly<u32>; 2],
    parameter: ReadOnly<u32, Parameter::Register>,
    version: ReadOnly<u32>,
    _reserved2: [ReadOnly<u32>; 18],
    confw: [ReadWrite<u32, WindowConfiguration::Register>; 4],
    _reserved3: [ReadOnly<u32>; 16],
    conf: [ReadWrite<u32, ACConfiguration::Register>; 8],
}

register_bitfields![u32,
	Control [
		/// Analog comparator test mode. Equal to 1 means AC outputs will be
		/// bypassed with values in AC test register.
		ACTEST 7,
		/// This bit is set when an enabled peripheral event is received (called
		/// by EVENTEN), and starts a single comparison.
		ESTART 5,
		/// This bit can be set by the user and starts a single comparison.
		USTART 4,
		/// This bit sets ESTART to 1 on receiving a peripheral event from
		/// another hardware module.
		EVENTEN 1,
		/// Enables or disables the ACIFC.
		EN 0
	],

	Status [
		/// This bit represents an output for the window mode, and reads one
		/// when the common input voltage is inside the window of the two
		/// non-common inputs.
		WFCS3 27,
		WFCS2 26,
		WFCS1 25,
		WFCS0 24,
		/// ACRDY is set when the AC output is ready. ACCS is set when the
		/// positive input voltage V_{INP} is greater than the negative input
		/// voltage V_{INN}.
		ACRDY7 15,
		ACCS7 14,
		ACRDY6 13,
		ACCS6 12,
		ACRDY5 11,
		ACCS5 10,
		ACRDY4 9,
		ACCS4 8,
		ACRDY3 7,
		ACCS3 6,
		ACRDY2 5,
		ACCS2 4,
		ACRDY1 3,
		ACCS1 2,
		ACRDY0 1,
		ACCS0 0
	],

	/// - IER: Writing a one to a bit in this register will set the
	///   corresponding bit in IMR.
	/// - IDR: Writing a one to a bit in this register will clear the
	///   corresponding bit in IMR.
	/// - IMR: Writing a one in any of these bits will enable the corresponding
	///   interrupt.
	/// - ISR: WFINTx shows if a window mode interrupt is pending. SUTINTx shows
	///   if a startup time interrupt is pending. ACINTx shows if a normal mode
	///   interrupt is pending.
	/// - ICR: Writing a one to a bit in this register will clear the
	///   corresponding bit in ISR and the corresponding interrupt request.
	Interrupt [
		WFINT3 27,
		WFINT2 26,
		WFINT1 25,
		WFINT0 24,
		SUTINT7 15,
		ACINT7 14,
		SUTINT6 13,
		ACINT6 12,
		SUTINT5 11,
		ACINT5 10,
		SUTINT4 9,
		ACINT4 8,
		SUTINT3 7,
		ACINT3 6,
		SUTINT2 5,
		ACINT2 4,
		SUTINT1 3,
		ACINT1 2,
		SUTINT0 1,
		ACINT0 0
	],

	Test [
		/// If equal to one, overrides ACx output with the value of ACTESTx.
		ACTEST7 7,
		ACTEST6 6,
		ACTEST5 5,
		ACTEST4 4,
		ACTEST3 3,
		ACTEST2 2,
		ACTEST1 1,
		ACTEST0 0
	],

	Parameter [
		/// If equal to one, window mode x is implemented.
		WIMPL3 19,
		WIMPL2 18,
		WIMPL1 17,
		WIMPL0 16,
		/// If equal to one, analog comparator x is implemented.
		ACIMPL7 7,
		ACIMPL6 6,
		ACIMPL5 5,
		ACIMPL4 4,
		ACIMPL3 3,
		ACIMPL2 2,
		ACIMPL1 1,
		ACIMPL0 0
		],

	WindowConfiguration [
		/// If equal to one, window mode is enabled.
		WFEN OFFSET(16) NUMBITS(1) [],
		/// If equal to one, peripheral event from ACWOUT is enabled.
		WEVEN OFFSET(11) NUMBITS(1) [],
		/// Peripheral Event Source Selection for Window Mode
		WEVSRC OFFSET (8) NUMBITS(3) [
			AcwoutRisingEdge = 0,
			AcwoutFallingEdge = 1,
			AcwoutRisingOrFallingEdge = 2,
			InsideWindow = 3,
			OutsideWindow = 4,
			MeasureDone = 5
		],
			/// Window Mode Interrupt Settings
		WIS OFFSET(0) NUMBITS (3)[
			/// Window interrupt as soon as the common input voltage is inside
			/// the window
			InterruptInsideWindow = 0,
			/// Window interrupt as soon as the common input voltage is outside
			/// the window
			InterruptOutsideWindow = 1,
			/// Window interrupt on toggle of ACWOUT
			InterruptToggleAcwout = 2,
			/// Window interrupt when evaluation of common input voltage is done
			InterruptAfterEvaluation = 3,
			/// Window interrupt when the common input voltage enters the window
			/// (i.e., rising-edge of ACWOUT)
			InterruptEnterWindow = 4,
			/// Window interrupt when the common input voltage leaves the window
			/// (i.e., falling-edge of ACWOUT)
			InterruptLeaveWindow = 5
	]
	],

	ACConfiguration [
		/// If equal to one, AC is always enabled.
		ALWAYSON OFFSET(27) NUMBITS(1) [],
		/// 0: Low-power mode. 1: Fastm ode.
		FAST OFFSET(26) NUMBITS(1) [],
		/// Hysteresis voltage value: 0/25/50/75 mV
		HYS OFFSET(24) NUMBITS(2) [
			HysteresisVoltage0mV = 0,
			HysteresisVoltage25mV = 1,
			HysteresisVoltage50mV = 2,
			HysteresisVoltage75mV = 3
		],
		/// Setting this to one will output peripheral event when ACOUT is zero.
		EVENP OFFSET(17) NUMBITS(1) [],
		/// Setting this to one will output peripheral event when ACOUT is one.
		EVENN OFFSET(16) NUMBITS(1) [],
		/// Negative input select. 00: ACANx pint selected, others reserved.
		INSELN OFFSET(8) NUMBITS(2) [],
		/// Choose between analog comparator mode.
		MODE OFFSET(4) NUMBITS(2) [
			Off = 0,
			ContinuousMeasurementMode = 1,
			/// User Triggered Single Measurement Mode
			UserMode = 2,
			/// Peripheral Event Single Measurement Mode
			PeripheralMode = 3
		],
		/// Interrupt settings
		IS OFFSET(0) NUMBITS(2) [
			/// When Vinp > Vinn
			WhenVinpGtVinn = 0,
			/// When Vinp < Vinn
			WhenVinpLtVinn = 1,
			/// On toggle of ACOUT
			OnToggleOfACOUT = 2,
			/// When comparison of Vinp and Vinn is done
			WhenComparisonDone = 3
		]
	]
];

const BASE_ADDRESS: *mut AcifcRegisters = 0x40040000 as *mut AcifcRegisters;
const ACIFC_REGS: StaticRef<AcifcRegisters> =
    unsafe { StaticRef::new(BASE_ADDRESS as *const AcifcRegisters) };

pub struct Acifc {
    //registers: *mut AcifcRegisters,
    //client: Option<&'a acifc::Client>,
}

pub static mut ACIFC: Acifc = Acifc::new();

/// Implement constructor for struct Acifc
impl Acifc {
    const fn new() -> Self {
        Acifc {
            //registers: BASE_ADDRESS,
            //client: Cell::new(None),
        }
    }

    fn enable_clock(&self) {
        pm::enable_clock(pm::Clock::PBA(pm::PBAClock::ACIFC));
    }

    pub fn set_client(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    /// Functions which (should) enable interrupts for the window or startup modes
    //fn enable_interrupts(&self) {
	// 	unimplemented!("ACIFC enabling interrupts");
    //     let regs = ACIFC_REGS;
    //     regs.ier
    //         .write(Interrupt::ACINT0::SET + Interrupt::ACINT1::SET);
    // }

    //fn disable_interrupts(&self) {
	// 	unimplemented!("ACIFC enabling interrupts");
    //     let regs = ACIFC_REGS;
    //     regs.idr
    //         .write(Interrupt::ACINT0::SET + Interrupt::ACINT1::SET);
    // }

    /// Handling interrupts not yet implemented.
    pub fn handle_interrupt(&mut self) {
        unimplemented!("ACIFC handling of interrupts");
    }

    fn initialize_acifc(&self) {
        let regs = ACIFC_REGS;
        self.enable_clock();
        regs.ctrl.write(Control::EN::SET);

        // Enable continuous measurement mode and always-on mode for AC0-3
        regs.conf[0].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );
        regs.conf[1].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );
        regs.conf[2].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );
        regs.conf[3].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );

        // Enable interrupts? Not yet used.
        // self.enable_interrupts();
    }

    fn comparison(&self, ac: usize) -> bool {
        let regs = ACIFC_REGS;
        let result;
        if ac == 0 {
            result = regs.sr.is_set(Status::ACCS0);
        } else if ac == 1 {
            result = regs.sr.is_set(Status::ACCS1);
        } else if ac == 2 {
            result = regs.sr.is_set(Status::ACCS2);
        } else {
            result = regs.sr.is_set(Status::ACCS3);
        }
        result
    }

    fn window_comparison(&self, window: usize) -> bool {
        let regs = ACIFC_REGS;
        let result;
        if window == 0 {
            regs.confw[0].write(WindowConfiguration::WFEN::SET);
            result = regs.sr.is_set(Status::WFCS0);
        } else {
            regs.confw[1].write(WindowConfiguration::WFEN::SET);
            result = regs.sr.is_set(Status::WFCS1);
        }
        return result;
    }
}

impl hil::analog_comparator::AnalogComparator for Acifc {
    fn initialize_acifc(&self) -> ReturnCode {
        self.initialize_acifc();
        return ReturnCode::SUCCESS;
    }

    fn comparison(&self, data: usize) -> bool {
        self.comparison(data)
    }

    fn window_comparison(&self, data: usize) -> bool {
        self.window_comparison(data)
    }
}
