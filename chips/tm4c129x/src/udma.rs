use nvic;
use kernel::common::VolatileCell;

const UDMA_BASE: usize = 0x400ff000;
struct UDMA {
    stat: VolatileCell<u32>,
    cfg: VolatileCell<u32>,
    ctlbase: VolatileCell<u32>,
    altbase: VolatileCell<u32>,
    waitstat: VolatileCell<u32>,
    swreq: VolatileCell<u32>,
    useburstset: VolatileCell<u32>,
    useburstclr: VolatileCell<u32>,
    reqmaskset: VolatileCell<u32>,
    reqmaskclr: VolatileCell<u32>,
    enaset: VolatileCell<u32>,
    enaclr: VolatileCell<u32>,
    altset: VolatileCell<u32>,
    altclr: VolatileCell<u32>,
    prioset: VolatileCell<u32>,
    prioclr: VolatileCell<u32>,
    _reserved0: [u32; 3],
    errclr: VolatileCell<u32>,
    _reserved1: [u32; 300],
    chasgn: VolatileCell<u32>,
    _reserved2: [u32; 3],
    chmap0: VolatileCell<u32>,
    chmap1: VolatileCell<u32>,
    chmap2: VolatileCell<u32>,
    chmap3: VolatileCell<u32>,
}
