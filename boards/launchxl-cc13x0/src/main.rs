#![no_std]
#![no_main]
#![feature(lang_items, compiler_builtins_lib, asm, panic_implementation)]

extern crate capsules;
extern crate cortexm3;

extern crate cc26x0;

#[allow(unused_imports)]
#[macro_use(create_capability, debug, debug_gpio, static_init)]
extern crate kernel;

use capsules::virtual_uart::{UartDevice, UartMux};
use cc26x0::{aon, gpio, peripherals, power, radio, rtc, uart, trng};
use kernel::capabilities;
use kernel::hil;

#[macro_use]
pub mod io;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 10000] = [0; 10000];

pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        radio::ble::Ble,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
    >,
    gpio: &'static capsules::gpio::GPIO<'static, gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, uart::UART>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
    >,
    rng: &'static capsules::rng::SimpleRng<'static, trng::Trng>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x0::init();
    
    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    
    // Setup AON event defaults
    aon::AON.setup();

    // Setup power management and register all resources to be used
    power::init();
    peripherals::init();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    
    // Power on peripheral domain and gpio clocks
    gpio::power_on_gpio();    
    
    // LEDs
    let led_pins = static_init!(
        [(
            &'static cc26x0::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 2],
        [
            (
                &cc26x0::gpio::PORT[6],
                capsules::led::ActivationMode::ActiveHigh
            ), // Red
            (
                &cc26x0::gpio::PORT[7],
                capsules::led::ActivationMode::ActiveHigh
            ), // Green
        ]
    );
    
    let led = static_init!(
        capsules::led::LED<'static, cc26x0::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONS
    let button_pins = static_init!(
        [(&'static cc26x0::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc26x0::gpio::PORT[13],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
            (
                &cc26x0::gpio::PORT[14],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 1
        ]
    );
    
    let button = static_init!(
        capsules::button::Button<'static, cc26x0::gpio::GPIOPin>,
        capsules::button::Button::new(
            button_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }
    
    // UART
    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        UartMux<'static>,
        UartMux::new(
            &cc26x0::uart::UART0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );

    hil::uart::UART::set_client(&cc26x0::uart::UART0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();

    cc26x0::uart::UART0.initialize_and_set_pins(3, 2);

    let console = static_init!(
        capsules::console::Console<uart::UART>,
        capsules::console::Console::new(
            &uart::UART0,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::uart::UART::set_client(&uart::UART0, console);
    console.initialize();
    
    // Create virtual device for kernel debug.
    let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    debugger_uart.setup();
    let debugger = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(
            debugger_uart,
            &mut kernel::debug::OUTPUT_BUF,
            &mut kernel::debug::INTERNAL_BUF,
        )
    );
    hil::uart::UART::set_client(debugger_uart, debugger);

    let debug_wrapper = static_init!(
        kernel::debug::DebugWriterWrapper,
        kernel::debug::DebugWriterWrapper::new(debugger)
    );
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static gpio::GPIOPin; 26],
        [
            &gpio::PORT[1],
            &gpio::PORT[2],
            &gpio::PORT[3],
            &gpio::PORT[5],
            &gpio::PORT[6],
            &gpio::PORT[7],
            &gpio::PORT[8],
            &gpio::PORT[9],
            &gpio::PORT[11],
            &gpio::PORT[12],
            &gpio::PORT[13],
            &gpio::PORT[14],
            &gpio::PORT[16],
            &gpio::PORT[17],
            &gpio::PORT[18],
            &gpio::PORT[19],
            &gpio::PORT[20],
            &gpio::PORT[21],
            &gpio::PORT[22],
            &gpio::PORT[23],
            &gpio::PORT[24],
            &gpio::PORT[25],
            &gpio::PORT[26],
            &gpio::PORT[27],
            &gpio::PORT[30],
            &gpio::PORT[31]
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(virtual_alarm1, 
                                          board_kernel.create_grant(&memory_allocation_capability)
                                          )
    );
    virtual_alarm1.set_client(alarm);
    
    let ble_radio_virtual_alarm = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );

    //trng::TRNG.enable();
    let rng = static_init!(
        capsules::rng::SimpleRng<'static, trng::Trng>,
        capsules::rng::SimpleRng::new(&trng::TRNG,
                                      board_kernel.create_grant(&memory_allocation_capability)
                                      )
    );
    trng::TRNG.set_client(rng);

    // Use BLE
    radio::RFC.set_client(&radio::BLE);
    let ble_radio = static_init!(
        capsules::ble_advertising_driver::BLE<
            'static,
            radio::ble::Ble,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, rtc::Rtc>,
        >,
        capsules::ble_advertising_driver::BLE::new(
            &mut radio::BLE,
            board_kernel.create_grant(&memory_allocation_capability),
            &mut capsules::ble_advertising_driver::BUF,
            ble_radio_virtual_alarm
        )
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
        &radio::BLE,
        ble_radio,
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
        &radio::BLE,
        ble_radio,
    );
    ble_radio_virtual_alarm.set_client(ble_radio);

    let launchxl_cc13x0 = Platform {
        ble_radio,
        gpio,
        led,
        button,
        console,
        alarm,
        rng,
    };

    let mut chip = cc26x0::chip::Cc26x0::new();

    debug!("Initialization complete. Entering main loop\r");
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    let ipc = &kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

    kernel::procs::load_processes(
        board_kernel,
        &cortexm3::syscall::SysCall::new(),
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    board_kernel.kernel_loop(&launchxl_cc13x0, &mut chip, Some(&ipc), &main_loop_capability);
}
