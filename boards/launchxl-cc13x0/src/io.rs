use core::fmt::{write, Arguments, Write};
use kernel::hil::uart::{self, UART};
use kernel::hil::gpio::Pin;
use kernel::hil::led;
use kernel::debug;
use cc26xx;
use cc26x0;
use core::panic::PanicInfo;

pub struct Writer {
    initialized: bool,
}

pub static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let uart = unsafe { &mut cc26x0::uart::UART0 };
        if !self.initialized {
            self.initialized = true;
            uart.init(uart::UARTParams {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
            });
        }
        for c in s.bytes() {
            uart.send_byte(c);
            while !uart.tx_ready() {}
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
        ($($arg:tt)*) => (
            {
                use core::fmt::write;
                let writer = &mut $crate::io::WRITER;
                let _ = write(writer, format_args!($($arg)*));
            }
        );
}

#[macro_export]
macro_rules! println {
        ($fmt:expr) => (print!(concat!($fmt, "\n")));
            ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[cfg(not(test))]
#[panic_implementation]
#[no_mangle]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    // 6 = Red led, 7 = Green led
    const LED_PIN: usize = 6;

    let led0 = &cc26xx::gpio::PORT[10]; // Red led
    let led1 = &cc26xx::gpio::PORT[15]; // Green led

    led0.make_output();
    led1.make_output();
    loop {
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..100000 {
            led0.set();
            led1.set();
        }
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..500000 {
            led0.set();
            led1.set();
        }
    }
}
