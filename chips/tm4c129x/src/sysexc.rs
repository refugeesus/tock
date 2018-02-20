#[allow(unused_imports)]
use kernel::common::VolatileCell;

#[allow(dead_code)]
const SYSEXC_BASE: usize = 0x400f9000;

#[allow(dead_code)]
struct SYSEXC {
    ris: VolatileCell<u32>,
    im: VolatileCell<u32>,
    mis: VolatileCell<u32>,
    ic: VolatileCell<u32>,
}
