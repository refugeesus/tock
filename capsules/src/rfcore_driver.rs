#![allow(unused_imports)]
use core::cell::Cell;
use self::commands as cmd;
use fixedvec::FixedVec;
use kernel::common::cells::TakeCell;
use kernel::{AppId, Callback, Driver, ReturnCode};
use kernel::hil::radio_client::{self, RadioConfig, RfcOperationStatus};
use cc26x0::{osc, radio::rfc};

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
    cmd_stack: TakeCell<'static, FixedVec<'static, RadioCommands>>,
    state_stack: TakeCell<'static, FixedVec<'static, State>>,
}

impl Radio {
    pub fn new(rfc: &'static rfc::RFCore) -> Radio {
        let rfc_stack =
            unsafe { static_init!(FixedVec<'static, State>, FixedVec::new(&mut RFC_STACK)) };

        let cmd_stack = unsafe {
            static_init!(
                FixedVec<'static, RadioCommands>,
                FixedVec::new(&mut CMD_STACK)
            )
        };
        debug_assert_eq!(rfc_stack.len(), 0);
        rfc_stack
            .push(State::Start)
            .expect("Rfc stack should be empty");
        debug_assert_eq!(cmd_stack.len(), 0);

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

    pub fn transmit(&self, buf: &'static mut [u8], len: usize) -> ReturnCode {
        if !self.rfc.is_enabled() {
            self.power_up();
        }

        // finish some transmit here
    }

    fn pop_state(&self) -> State {
        let state_stack = self
            .state_stack
            .take()
            .expect("self.state_stack must be some here");
        let state = state_stack.pop().expect("self.state_stack is empty");
        self.state_stack.replace(state_stack);
        state
    }

    // Functions for pushing and popping radio commands from the command stack
    fn push_cmd(&self, cmd: RadioCommands) {
        let cmd_stack = self
            .cmd_stack
            .take()
            .expect("self.cmd_stack must be some here");
        cmd_stack.push(cmd).expect("self.cmd_stack is full");
        self.cmd_stack.replace(cmd_stack);
    }

    fn pop_cmd(&self) -> RadioCommands {
        let cmd_stack = self
            .cmd_stack
            .take()
            .expect("self.cmd_stack must be some here");
        let cmd = cmd_stack.pop().expect("self.cmd_stack is empty");
        self.cmd_stack.replace(cmd_stack);
        cmd
    }
}

impl rfc::RFCoreClient for Radio {
    fn command_done(&self) {
        match self.state.get() {

        }
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

impl RadioDriver for Radio {
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

impl From<usize> for RfcOperationStatus {
    fn from(val: usize) -> RfcOperationStatus {
        match val {
            0 => RfcOperationStatus::Idle,
            1 => RfcOperationStatus::Pending,
            2 => RfcOperationStatus::Active,
            3 => RfcOperationStatus::Skipped,
            4 => RfcOperationStatus::SendDone,
            5 => RfcOperationStatus::TxDone,
            6 => RfcOperationStatus::CommandDone,
            7 => RfcOperationStatus::LastCommandDone,
            8 => RfcOperationStatus::RxOk,
            9 => RfcOperationStatus::TxDone,
            val => {
                debug_assert!(false, "{} does not represent a valid command.", val);
                RfcOperationStatus::Invalid
            }
        }
    }
}

pub mod commands {
    use kernel::common::registers::ReadOnly;

    // Radio Commands

    // RFC Immediate commands
    pub const RFC_CMD0: u16 = 0x801;
    pub const RFC_PING: u16 = 0x406;
    pub const RFC_BUS_REQUEST: u16 = 0x40E;
    pub const RFC_START_RAT_TIMER: u16 = 0x0405;
    pub const RFC_STOP_RAT_TIMER: u16 = 0x0809;
    pub const RFC_SETUP: u16 = 0x0802;
    pub const RFC_STOP: u16 = 0x0402;
    pub const RFC_FS_POWERDOWN: u16 = 0x080D;

    /* Basic direct command */
    #[derive(Debug, Clone, Copy)]
    pub struct DirectCommand {
        pub command_id: u16,
        pub parameters: u16,
    }

    impl DirectCommand {
        pub const fn new(command: u16, param: u16) -> DirectCommand {
            DirectCommand {
                command_id: command,
                parameters: param,
            }
        }
    }

    /* This is common between every command. Use this
       In order to decode any arbitrary command! */
    #[allow(unused)]
    #[repr(C)]
    pub struct CommandCommon {
        pub command_no: ReadOnly<u16>,
        pub status: ReadOnly<u16>,
        pub p_next_op: ReadOnly<u32>,
        pub start_time: ReadOnly<u32>,
        pub start_trigger: ReadOnly<u8>,
        pub condition: RfcCondition,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdNop {
        pub command_no: u16,
        pub status: u16,
        pub p_next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
    }

    impl CmdNop {
        pub fn new() -> CmdNop {
            CmdNop {
                command_no: 0x0801,
                status: 0,
                p_next_op: 0,
                start_time: 0,
                start_trigger: 0,
                condition: {
                    let mut cond = RfcCondition(0);
                    cond.set_rule(0x01);
                    cond
                },
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdRadioSetup {
        pub command_no: u16, //0x0802
        pub status: u16,
        pub p_next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub mode: u8,
        pub lo_divider: u8,
        pub config: RfcSetupConfig,
        pub tx_power: u16,
        pub reg_override: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdSyncStartRat {
        pub command_no: u16, // 0x080A 
        pub status: u16,
        pub next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub _reserved: u16,
        pub rat0: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdSyncStopRat {
        pub command_no: u16, // 0x0809 
        pub status: u16,
        pub next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub _reserved: u16,
        pub rat0: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdFS {
        pub command_no: u16, // 0x0803
        pub status: u16,
        pub p_next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub fract_freq: u16,
        pub synth_conf: u8,
        _reserved: [u8; 5],
    }

    // Power up frequency synthesizer
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdFSPowerup {
        pub command_no: u16, //0x080C
        pub status: u16,
        pub p_next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub reg_override: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdFsPowerdown {
        pub command_no: u16, //0x080D
        pub status: u16,
        pub p_nextop: u32,
        pub ratmr: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
    }

    // Continuous TX test, unimplemented
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdTxTest {
        // command_no 0x0808
        pub command_no: u16,
        pub status: u16,
        pub p_next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub config: u8,
        _reserved_a: u8,
        pub tx_word: u16,
        _reserved_b: u8,
        pub end_trigger: RfcTrigger,
        pub sync_word: u32,
        pub end_time: u32,
    }

    // Continuous RX test, unimplemented
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CmdRxTest {
        pub command_no: u16, // 0x0807
        pub status: u16,
        pub p_next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub config: u8,
        pub end_trigger: u8,
        pub sync_word: u32,
        pub end_time: u32,
    }

    /* Bitfields used by many commands */
    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcTrigger(u8);
        impl Debug;
        pub _trigger_type, _set_trigger_type  : 3, 0;
        pub _enable_cmd, _set_enable_cmd      : 4;
        pub _trigger_no, _set_trigger_no      : 6, 5;
        pub _past_trigger, _set_past_trigger  : 7;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcCondition(u8);
        impl Debug;
        pub _rule, set_rule : 3, 0;
        pub _skip, _set_skip : 7, 4;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcSetupConfig(u16);
        impl Debug;
        pub _frontend_mode, set_frontend_mode: 2, 0;
        pub _bias_mode, set_bias_mode: 3;
        pub _analog_cfg_mode, _set_analog_config_mode: 9, 4;
        pub _no_fs_powerup, _set_no_fs_powerup: 10;
    }
}
