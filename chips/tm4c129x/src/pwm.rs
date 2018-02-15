use kernel::common::VolatileCell;
use nvic;

const PWM0_BASE: usize = 0x40028000;
struct PWM0 {
    ctl: VolatileCell<u32>,
    sync: VolatileCell<u32>,
    enable: VolatileCell<u32>,
    invert: VolatileCell<u32>,
    fault: VolatileCell<u32>,
    inten: VolatileCell<u32>,
    ris: VolatileCell<u32>,
    isc: VolatileCell<u32>,
    status: VolatileCell<u32>,
    faultval: VolatileCell<u32>,
    enupd: VolatileCell<u32>,
    _reserved0: [u32; 5],
    _0_ctl: VolatileCell<u32>,
    _0_inten: VolatileCell<u32>,
    _0_ris: VolatileCell<u32>,
    _0_isc: VolatileCell<u32>,
    _0_load: VolatileCell<u32>,
    _0_count: VolatileCell<u32>,
    _0_cmpa: VolatileCell<u32>,
    _0_cmpb: VolatileCell<u32>,
    _0_gena: VolatileCell<u32>,
    _0_genb: VolatileCell<u32>,
    _0_dbctl: VolatileCell<u32>,
    _0_dbrise: VolatileCell<u32>,
    _0_dbfall: VolatileCell<u32>,
    _0_fltsrc0: VolatileCell<u32>,
    _0_fltsrc1: VolatileCell<u32>,
    _0_minfltper: VolatileCell<u32>,
    _1_ctl: VolatileCell<u32>,
    _1_inten: VolatileCell<u32>,
    _1_ris: VolatileCell<u32>,
    _1_isc: VolatileCell<u32>,
    _1_load: VolatileCell<u32>,
    _1_count: VolatileCell<u32>,
    _1_cmpa: VolatileCell<u32>,
    _1_cmpb: VolatileCell<u32>,
    _1_gena: VolatileCell<u32>,
    _1_genb: VolatileCell<u32>,
    _1_dbctl: VolatileCell<u32>,
    _1_dbrise: VolatileCell<u32>,
    _1_dbfall: VolatileCell<u32>,
    _1_fltsrc0: VolatileCell<u32>,
    _1_fltsrc1: VolatileCell<u32>,
    _1_minfltper: VolatileCell<u32>,
    _2_ctl: VolatileCell<u32>,
    _2_inten: VolatileCell<u32>,
    _2_ris: VolatileCell<u32>,
    _2_isc: VolatileCell<u32>,
    _2_load: VolatileCell<u32>,
    _2_count: VolatileCell<u32>,
    _2_cmpa: VolatileCell<u32>,
    _2_cmpb: VolatileCell<u32>,
    _2_gena: VolatileCell<u32>,
    _2_genb: VolatileCell<u32>,
    _2_dbctl: VolatileCell<u32>,
    _2_dbrise: VolatileCell<u32>,
    _2_dbfall: VolatileCell<u32>,
    _2_fltsrc0: VolatileCell<u32>,
    _2_fltsrc1: VolatileCell<u32>,
    _2_minfltper: VolatileCell<u32>,
    _3_ctl: VolatileCell<u32>,
    _3_inten: VolatileCell<u32>,
    _3_ris: VolatileCell<u32>,
    _3_isc: VolatileCell<u32>,
    _3_load: VolatileCell<u32>,
    _3_count: VolatileCell<u32>,
    _3_cmpa: VolatileCell<u32>,
    _3_cmpb: VolatileCell<u32>,
    _3_gena: VolatileCell<u32>,
    _3_genb: VolatileCell<u32>,
    _3_dbctl: VolatileCell<u32>,
    _3_dbrise: VolatileCell<u32>,
    _3_dbfall: VolatileCell<u32>,
    _3_fltsrc0: VolatileCell<u32>,
    _3_fltsrc1: VolatileCell<u32>,
    _3_minfltper: VolatileCell<u32>,
    _reserved1: [u32; 432],
    _0_fltsen: VolatileCell<u32>,
    _0_fltstat0: VolatileCell<u32>,
    _0_fltstat1: VolatileCell<u32>,
    _reserved2: [u32; 29],
    _1_fltsen: VolatileCell<u32>,
    _1_fltstat0: VolatileCell<u32>,
    _1_fltstat1: VolatileCell<u32>,
    _reserved3: [u32; 29],
    _2_fltsen: VolatileCell<u32>,
    _2_fltstat0: VolatileCell<u32>,
    _2_fltstat1: VolatileCell<u32>,
    _reserved4: [u32; 29],
    _3_fltsen: VolatileCell<u32>,
    _3_fltstat0: VolatileCell<u32>,
    _3_fltstat1: VolatileCell<u32>,
    _reserved5: [u32; 397],
    pp: VolatileCell<u32>,
    _reserved6: [u32; 1],
    cc: VolatileCell<u32>,
}
