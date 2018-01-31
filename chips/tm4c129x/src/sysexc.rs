use nvic;
use kernel::common::VolatileCell;

const SYSEXC_BASE: usize = 0x400f9000;
struct SYSEXC {
    ris: VolatileCell<u32>,
    im: VolatileCell<u32>,
    mis: VolatileCell<u32>,
    ic: VolatileCell<u32>,
}
