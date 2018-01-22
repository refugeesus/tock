use kernel::common::VolatileCell;

struct FLASH_CTRL {
    fma: VolatileCell<u32>,
    fmd: VolatileCell<u32>,
    fmc: VolatileCell<u32>,
    fcris: VolatileCell<u32>,
    fcim: VolatileCell<u32>,
    fcmisc: VolatileCell<u32>,
    _reserved0: [u32; 2],
    fmc2: VolatileCell<u32>,
    _reserved1: [u32; 3],
    fwbval: VolatileCell<u32>,
    _reserved2: [u32; 2],
    flpekey: VolatileCell<u32>,
    _reserved3: [u32; 48],
    fwbn: VolatileCell<u32>,
    _reserved4: [u32; 943],
    pp: VolatileCell<u32>,
    ssize: VolatileCell<u32>,
    conf: VolatileCell<u32>,
    romswmap: VolatileCell<u32>,
    dmasz: VolatileCell<u32>,
    dmast: VolatileCell<u32>,
    _reserved5: [u32; 63],
    rvp: VolatileCell<u32>,
    _reserved6: [u32; 62],
    bootcfg: VolatileCell<u32>,
    _reserved7: [u32; 3],
    userreg0: VolatileCell<u32>,
    userreg1: VolatileCell<u32>,
    userreg2: VolatileCell<u32>,
    userreg3: VolatileCell<u32>,
    _reserved8: [u32; 4],
    fmpre0: VolatileCell<u32>,
    fmpre1: VolatileCell<u32>,
    fmpre2: VolatileCell<u32>,
    fmpre3: VolatileCell<u32>,
    fmpre4: VolatileCell<u32>,
    fmpre5: VolatileCell<u32>,
    fmpre6: VolatileCell<u32>,
    fmpre7: VolatileCell<u32>,
    fmpre8: VolatileCell<u32>,
    fmpre9: VolatileCell<u32>,
    fmpre10: VolatileCell<u32>,
    fmpre11: VolatileCell<u32>,
    fmpre12: VolatileCell<u32>,
    fmpre13: VolatileCell<u32>,
    fmpre14: VolatileCell<u32>,
    fmpre15: VolatileCell<u32>,
    _reserved9: [u32; 112],
    fmppe0: VolatileCell<u32>,
    fmppe1: VolatileCell<u32>,
    fmppe2: VolatileCell<u32>,
    fmppe3: VolatileCell<u32>,
    fmppe4: VolatileCell<u32>,
    fmppe5: VolatileCell<u32>,
    fmppe6: VolatileCell<u32>,
    fmppe7: VolatileCell<u32>,
    fmppe8: VolatileCell<u32>,
    fmppe9: VolatileCell<u32>,
    fmppe10: VolatileCell<u32>,
    fmppe11: VolatileCell<u32>,
    fmppe12: VolatileCell<u32>,
    fmppe13: VolatileCell<u32>,
    fmppe14: VolatileCell<u32>,
    fmppe15: VolatileCell<u32>,
}

const FLASH_CTRL_BASE: usize = 0x400fd000;
