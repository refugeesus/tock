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
