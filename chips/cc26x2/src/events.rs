
pub static mut EVENTS: u64 = 0;

use cortexm::support::{atomic_read, atomic_write};

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EVENT_PRIORITY {
    GPIO = 0,
    UART0 = 2,
    UART1 = 1,
    AON_RTC = 3,
    RTC = 4,
    I2C0 = 6,
    AON_PROG = 7,
}
}

pub fn next_pending() -> Option<EVENT_PRIORITY> {
    let mut event_flags;
    unsafe { event_flags = atomic_read(&EVENTS) }
    

    let mut count = 0;
    // stay in loop until we found the flag
    while event_flags!=0 {
        // if flag is found, return the count
        if (event_flags & 0b1) != 0 {
            return Some( EVENT_PRIORITY::from_u8(count)
                .expect("Unmapped EVENT_PRIORITY"));
        }
        // otherwise increment
        count += 1;
        event_flags >>= 1;
    }
    None
}

pub fn set_event_flag(priority: EVENT_PRIORITY) {
    unsafe { 
        let mut val = atomic_read(&EVENTS);
        val |= 0b1 << (priority as u8) as u64;
        atomic_write(&mut EVENTS, val);
    };
}

pub fn clear_event_flag(priority: EVENT_PRIORITY) {
    unsafe { 
        let mut val = atomic_read(&EVENTS);
        val &= !0b1 << (priority as u8) as u64;
        atomic_write(&mut EVENTS, val);
 };
}

use num_traits::FromPrimitive;

enum_from_primitive!{
#[derive(Debug, PartialEq)]
pub enum NVIC_IRQ {
    GPIO = 0,
    I2C = 1,
    RF_CORE_PE1 = 2,
    //UNASSIGNED 3
    AON_RTC = 4,
    UART0 = 5,
    SSI0 = 7,
    SSI1 = 8,
    RF_CORE_PE2 = 9,
    RF_CORE_HW = 10,
    RF_CMD_ACK = 11,
    I2S = 12,
    //UNASSIGNED 13
    WATCHDOG = 14,
    GPT0A = 15,
    GPT0B = 16,
    GPT1A = 17,
    GPT1B = 18,
    GPT2A = 19,
    GPT2B = 20,
    GPT3A = 21,
    GPT3B = 22,
    CRPYTO = 23,
    DMA_SW = 24,
    DMA_ERROR = 25,
    FLASH = 26,
    SW_EVENT0 = 27,
    AUX_COMBINED = 28,
    AON_PROG = 29,
    DYNAMIC_PROG = 30,
    AUX_COMP_A = 31,
    AUX_ADC = 32,
    TRNG = 33, 
    UART1 = 36
}
}
