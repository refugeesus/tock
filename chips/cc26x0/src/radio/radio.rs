#![allow(unused_imports)]
use core::cell::Cell;
use radio::commands as cmd;
use fixedvec::FixedVec;
use kernel::common::cells::TakeCell;
use kernel::{AppId, Callback, Driver, ReturnCode};
use kernel::hil::radio_client::{self, RadioConfig, RfcOperationStatus};
use osc;
use radio::rfc;

static mut RFPARAMS: [u32; 18] = [
    // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x02400403,
    // Synth: Set minimum RTRIM to 6
    0x00068793,
    // Synth: Configure extra PLL filtering
    0x001C8473,
    // Synth: Configure extra PLL filtering
    0x00088433,
    // Synth: Set Fref to 4 MHz
    0x000684A3,
    // Synth: Configure faster calibration
    // HW32_ARRAY_OVERRIDE(0x4004,1),
    // Synth: Configure faster calibration
    0x180C0618,
    // Synth: Configure faster calibration
    0xC00401A1,
    // Synth: Configure faster calibration
    0x00010101,
    // Synth: Configure faster calibration
    0xC0040141,
    // Synth: Configure faster calibration
    0x00214AD3,
    // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us)
    0x02980243,
    // Synth: Set loop bandwidth after lock to 20 kHz
    0x0A480583,
    // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603,
    // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623,
    // Tx: Configure PA ramping, set wait time before turning off (0x1F ticks of 16/24 us = 20.7 us).
    // HW_REG_OVERRIDE(0x6028,0x001F),
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[3]=1)
    // ADI_HALFREG_OVERRIDE(0,16,0x8,0x8),
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[4]=1)
    // ADI_HALFREG_OVERRIDE(0,17,0x1,0x1),
    // Rx: Set AGC reference level to 0x1A (default: 0x2E)
    // HW_REG_OVERRIDE(0x609C,0x001A),
    // Rx: Set LNA bias current offset to adjust +1 (default: 0)
    0x00018883,
    // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    0x000288A3,
    // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD)
    // ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    // TX power override
    // DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0xFFFC08C3,
    // Tx: Set PA trim to max to maximize its output power (in ADI0, set PACTL0=0xF8)
    // ADI_REG_OVERRIDE(0,12,0xF8),
    0xFFFFFFFF,
];

// static mut PAYLOAD: [u8; 256] = [0; 256];
#[derive(Debug, Clone, Copy)]
pub enum State {
    Start,
    Pending,
    CommandStatus(RfcOperationStatus),
    Command(RadioCommands),
    Done,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
pub enum RadioCommands {
    Direct(cmd::DirectCommand),
    RadioSetup(cmd::CmdRadioSetup),
    Common(cmd::CmdNop),
    FSPowerup(cmd::CmdFSPowerup),
    FSPowerdown(cmd::CmdFsPowerdown),
    StartRat(cmd::CmdSyncStartRat),
    StopRat(cmd::CmdSyncStopRat),
    NotSupported,
}

impl Default for RadioCommands {
    fn default() -> RadioCommands {
        RadioCommands::Common(cmd::CmdNop::new())
    }
}

pub static mut RFC_STACK: [State; 6] = [State::Start; 6];

pub struct Radio {
    rfc: &'static rfc::RFCore,
    state: Cell<State>,
    callback: Cell<Option<Callback>>,
    tx_radio_client: Cell<Option<&'static radio_client::TxClient>>,
    rx_radio_client: Cell<Option<&'static radio_client::RxClient>>,
    schedule_powerdown: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
}

impl Radio {
    pub fn new(rfc: &'static rfc::RFCore) -> Radio {
        
        Radio {
            rfc,
            state: Cell::new(State::Start),
            callback: Cell::new(None),
            tx_radio_client: Cell::new(None),
            rx_radio_client: Cell::new(None),
            schedule_powerdown: Cell::new(false),
            tx_buf: TakeCell::empty(),
        }
    }

    pub fn power_up(&self) {
        self.rfc.set_mode(rfc::RfcMode::IEEE);

        // osc::OSC.config_hf_osc(osc::HF_XOSC);
        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();
        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();
        // osc::OSC.config_hf_osc(osc::HF_XOSC);
        
        unsafe {
            let reg_overrides: u32 = RFPARAMS.as_mut_ptr() as u32;
            self.rfc.setup(reg_overrides)
        }
    }

    pub fn power_down(&self) {
        self.rfc.disable();
    }

}

impl rfc::RFCoreClient for Radio {
    fn command_done(&self) {
    }

    fn tx_done(&self) {
        
        if self.schedule_powerdown.get() {
            self.power_down();
            osc::OSC.switch_to_hf_rcosc();

            self.schedule_powerdown.set(false);
        }
        
        let buf = self.tx_buf.take();
        self.tx_radio_client
            .get()
            .map(|client| client.send_done(buf.unwrap(), ReturnCode::SUCCESS));

    }
}
/*
impl Driver for Radio {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _appid: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Callback for RFC Interrupt ready
            0 => {
                self.callback.set(callback);
                return ReturnCode::SUCCESS;
            }
            // Default
            _ => return ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, minor_num: usize, _r2: usize, _r3: usize, _caller_id: AppId) -> ReturnCode {
        let command_status: RfcOperationStatus = minor_num.into();

        match command_status {
            // Handle callback for CMDSTA after write to CMDR
            RfcOperationStatus::SendDone => {
                let current_command = self.pop_cmd();
                self.push_state(State::CommandStatus(command_status));
                match self.rfc.cmdsta() {
                    ReturnCode::SUCCESS => {
                        self.push_cmd(current_command);
                        ReturnCode::SUCCESS
                    }
                    ReturnCode::EBUSY => {
                        self.push_cmd(current_command);
                        ReturnCode::EBUSY
                    }
                    ReturnCode::EINVAL => {
                        self.pop_state();
                        ReturnCode::EINVAL
                    }
                    _ => {
                        self.pop_state();
                        self.pop_cmd();
                        ReturnCode::ENOSUPPORT
                    }
                }
            }
            // Handle callback for command status after command is finished
            RfcOperationStatus::CommandDone => {
                // let current_command = self.rfc.command.as_ptr() as u32;
                let current_command = self.pop_cmd();
                self.push_state(State::CommandStatus(command_status));
                match self.rfc.wait(&current_command) {
                    // match self.rfc.wait_cmdr(current_command) {
                    ReturnCode::SUCCESS => {
                        self.pop_state();
                        ReturnCode::SUCCESS
                    }
                    ReturnCode::EBUSY => {
                        self.push_cmd(current_command);
                        ReturnCode::EBUSY
                    }
                    ReturnCode::ECANCEL => {
                        self.pop_state();
                        ReturnCode::ECANCEL
                    }
                    ReturnCode::FAIL => {
                        self.pop_state();
                        ReturnCode::FAIL
                    }
                    _ => {
                        self.pop_state();
                        ReturnCode::ENOSUPPORT
                    }
                }
            }
            RfcOperationStatus::Invalid => panic!("Invalid command status"),
            _ => panic!("Unimplemented!"),
        }
    }
}
*/
impl RadioConfig for Radio {
    fn set_tx_client(&self, tx_client: &'static radio_client::TxClient) {
        self.tx_radio_client.set(Some(tx_client));
    }

    fn set_rx_client(&self, rx_client: &'static radio_client::RxClient, rx_buf: &'static mut [u8]) {
        self.rx_radio_client.set(Some(rx_client));
    }

    fn set_receive_buffer(&self, rx_buf: &'static mut [u8]){
        // maybe make a rx buf only when needed? 
    }
}
