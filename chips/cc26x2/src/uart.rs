//! UART driver, cc26x2 family
use kernel;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::uart;
use kernel::ReturnCode;
use prcm;

const MCU_CLOCK: u32 = 48_000_000;

#[repr(C)]
struct UartRegisters {
    dr: ReadWrite<u32>,                                 // data
    rsr_ecr: ReadWrite<u32>,                            // status
    _reserved0: [u32; 0x4],                             // don't write here
    fr: ReadOnly<u32, Flags::Register>,                 // flag
    _reserved1: [u32; 0x2],                             // don't write here
    ibrd: ReadWrite<u32, IntDivisor::Register>,         // integer baud-rate divisor
    fbrd: ReadWrite<u32, FracDivisor::Register>,        // fractional baud-rate divisor
    lcrh: ReadWrite<u32, LineControl::Register>,        // line control
    ctl: ReadWrite<u32, Control::Register>,             // control
    ifls: ReadWrite<u32, FifoInterrupts::Register>,     // interrupt fifo level select
    imsc: ReadWrite<u32, Interrupts::Register>,         // interrupt mask set/clear
    ris: ReadOnly<u32, Interrupts::Register>,           // raw interrupt status
    mis: ReadOnly<u32, Interrupts::Register>,           // masked interrupt status
    icr: WriteOnly<u32, Interrupts::Register>,          // interrupt clear
    dmactl: ReadWrite<u32>,                             // DMA control
}

register_bitfields![
    u32,
    Control [
        UART_ENABLE OFFSET(0) NUMBITS(1) [],
        TX_ENABLE OFFSET(8) NUMBITS(1) [],
        RX_ENABLE OFFSET(9) NUMBITS(1) []
    ],

    LineControl [
        FIFO_ENABLE OFFSET(4) NUMBITS(1) [],
        WORD_LENGTH OFFSET(5) NUMBITS(2) [
            Len5 = 0x0,
            Len6 = 0x1,
            Len7 = 0x2,
            Len8 = 0x3
        ]
    ],
    IntDivisor [
        DIVISOR OFFSET(0) NUMBITS(16) []
    ],
    FracDivisor [
        DIVISOR OFFSET(0) NUMBITS(6) []
    ],
    Flags [
        CTS OFFSET(0) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        RX_FIFO_EMPTY OFFSET(4) NUMBITS(1) [],
        TX_FIFO_FULL OFFSET(5) NUMBITS(1) [],
        RX_FIFO_FULL OFFSET(6) NUMBITS(1) [],
        TX_FIFO_EMPTY OFFSET(7) NUMBITS(1) []
    ],
    Interrupts [
        ALL_INTERRUPTS OFFSET(0) NUMBITS(12) [
            // sets all interrupts without writing 1's to reg with undefined behavior
            Set =  0b111111110010,
            // you are allowed to write 0 to everyone
            Clear = 0x000000 
        ],
        CTSIMM OFFSET(1) NUMBITS(1) [], // clear to send interrupt mask
        RX OFFSET(4) NUMBITS(1) [],   // receive interrupt mask
        TX OFFSET(5) NUMBITS(1) [],   // transmit interrupt mask
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],   // receive timeout interrupt mask
        FE OFFSET(7) NUMBITS(1) [],   // framing error interrupt mask
        PE OFFSET(8) NUMBITS(1) [],   // parity error interrupt mask
        BE OFFSET(9) NUMBITS(1) [],   // break error interrupt mask
        OE OFFSET(10) NUMBITS(1) [],  // overrun error interrupt mask
        END_OF_TRANSMISSION OFFSET(11) NUMBITS(1) [] // end of transmission interrupt mask
    ],
    FifoInterrupts [
        TXSEL OFFSET(0) NUMBITS(3) [
            OneEighthFull = 0x0,
            OneQuarterFull = 0x1,
            HalfFull = 0x2,
            ThreeQuartersFull = 0x3,
            SevenEightsFull = 0x7
        ],
        TX_FIFO_FULL OFFSET(5) NUMBITS(1) [
            OneEighthFull = 0x0,
            OneQuarterFull = 0x1,
            HalfFull = 0x2,
            ThreeQuartersFull = 0x3,
            SevenEightsFull = 0x7
        ]
    ]
];

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40001000 as *const UartRegisters) };

const UART1_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000B000 as *const UartRegisters) };

use cortexm4::nvic;
use events;
use events::EVENT_PRIORITY;

pub static UART0_NVIC: nvic::Nvic = unsafe { nvic::Nvic::new(events::NVIC_IRQ::UART0 as u32) };
pub static UART1_NVIC: nvic::Nvic = unsafe { nvic::Nvic::new(events::NVIC_IRQ::UART1 as u32) };

pub static mut UART0_RX_BUF: [u8; 4] = [0; 4];
pub static mut UART1_RX_BUF: [u8; 4] = [0; 4];

pub static mut UART1_ISR_RX_BUF: [u8; 4] = [0; 4];
pub static mut UART0_ISR_RX_BUF: [u8; 4] = [0; 4];

pub static mut UART0: UART = UART::new(&UART0_BASE, &UART0_NVIC, EVENT_PRIORITY::UART0);
pub static mut UART1: UART = UART::new(&UART1_BASE, &UART1_NVIC, EVENT_PRIORITY::UART1);

macro_rules! uart_nvic {
    ($fn_name:tt, $uart:ident) => {
        // handle RX interrupt
        pub extern "C" fn $fn_name() {
            unsafe {

                // get a copy of the masked interrupt status
                let isr_status = $uart.registers.mis.extract();

                // signal to kernel-space that there is an event
                events::set_event_flag($uart.event_priority);

                // handle RX
                if (isr_status.read(Interrupts::RX_TIMEOUT) != 0) || (isr_status.read(Interrupts::RX) != 0) {
                    while $uart.rx_fifo_not_empty() {
                        let byte = $uart.read_byte();
                        $uart.isr_buf.map( |buf| {
                            let index = $uart.isr_len.get();
                            buf[index] = byte;
                            $uart.isr_len.set(index+1)
                        });
                    }
                }

                if (isr_status.read(Interrupts::END_OF_TRANSMISSION) != 0) {
                    $uart.transaction.map( |mut transaction| {

                        while 
                        $uart.tx_fifo_not_full()
                         &&
                        transaction.index < transaction.length
                        {
                            $uart.send_byte(transaction.buffer[transaction.index]);
                            transaction.index += 1;
                        }
                    });
                }
                $uart.registers.icr.write(Interrupts::ALL_INTERRUPTS::Set);
                $uart.nvic.clear_pending();
            }
        }
    }
}

uart_nvic!(uart0_isr, UART0);
uart_nvic!(uart1_isr, UART1);


/// Stores an ongoing TX transaction
struct Transaction {
    /// The buffer containing the bytes to transmit as it should be returned to
    /// the client
    buffer: &'static mut [u8],
    /// The total amount to transmit
    length: usize,
    /// The index of the byte currently being sent
    index: usize,
}

use kernel::common::cells::VolatileCell;



pub struct UART {
    registers: &'static StaticRef<UartRegisters>,
    client: OptionalCell<&'static uart::Client>,
    transaction: MapCell<Transaction>,
    rx_buf: MapCell<&'static mut [u8]>,
    
    // ISR related stuff
    pub nvic: &'static nvic::Nvic,
    pub nvic_event: VolatileCell<bool>,
    event_priority: EVENT_PRIORITY,
    event_flags: ReadWrite<u32, Interrupts::Register>,
    isr_buf: MapCell<&'static mut [u8]>,
    isr_len: VolatileCell<usize>,
}

impl UART {
    const fn new(
        base_reg: &'static StaticRef<UartRegisters>, 
        nvic: &'static nvic::Nvic, 
        event_priority: EVENT_PRIORITY, 
        ) -> UART {
        UART {
            registers: base_reg,
            client: OptionalCell::empty(),
            transaction: MapCell::empty(),
            rx_buf: MapCell::empty(),
            nvic: nvic,
            nvic_event: VolatileCell::new(false),
            event_priority,
            event_flags: ReadWrite::new(0),
            isr_buf: MapCell::empty(),
            isr_len: VolatileCell::new(0),

        }
    }


    /// Initialize the UART hardware.
    ///
    /// This function needs to be run before the UART module is used.
    pub unsafe fn initialize(&self, buf: &'static mut [u8]) {
        self.isr_buf.put(buf);
        self.power_and_clock();
        self.enable_interrupts();
    }

    fn configure(&self, params: kernel::hil::uart::UARTParameters) -> ReturnCode {
        // These could probably be implemented, but are currently ignored, so
        // throw an error.
        if params.stop_bits != kernel::hil::uart::StopBits::One {
            return ReturnCode::ENOSUPPORT;
        }
        if params.parity != kernel::hil::uart::Parity::None {
            return ReturnCode::ENOSUPPORT;
        }
        if params.hw_flow_control != false {
            return ReturnCode::ENOSUPPORT;
        }

        // Disable the UART before configuring
        self.disable();
        self.set_baud_rate(params.baud_rate);

        // Set word length
        self.registers.lcrh.write(LineControl::WORD_LENGTH::Len8);

        self.fifo_enable();

        // Enable UART, RX and TX
        self.registers
            .ctl
            .write(Control::UART_ENABLE::SET + Control::RX_ENABLE::SET + Control::TX_ENABLE::SET);

        ReturnCode::SUCCESS
    }

    fn power_and_clock(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Serial);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}
        prcm::Clock::enable_uart();
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        // Fractional baud rate divider
        let div = (((MCU_CLOCK * 8) / baud_rate) + 1) / 2;
        // Set the baud rate
        self.registers.ibrd.write(IntDivisor::DIVISOR.val(div / 64));
        self.registers
            .fbrd
            .write(FracDivisor::DIVISOR.val(div % 64));
    }

    fn fifo_enable(&self) {
        self.registers.lcrh.modify(LineControl::FIFO_ENABLE::SET);
    }

    fn fifo_disable(&self) {
        self.registers.lcrh.modify(LineControl::FIFO_ENABLE::CLEAR);
    }

    fn disable(&self) {
        self.fifo_disable();
        self.registers.ctl.modify(
            Control::UART_ENABLE::CLEAR + Control::TX_ENABLE::CLEAR + Control::RX_ENABLE::CLEAR,
        );
    }


    pub fn enable_interrupts(&self) {
        // clear all
        self.registers.icr.write(Interrupts::ALL_INTERRUPTS::Clear);
        // set all interrupts
        self.registers.imsc.modify(Interrupts::RX.val(1));
        self.registers.imsc.modify(Interrupts::RX_TIMEOUT.val(1));
        self.registers.imsc.modify(Interrupts::END_OF_TRANSMISSION.val(1));
    }

    pub fn handle_interrupt(&self) {
        self.handle_event();
    } 

    pub fn handle_event(&self) {
        // get a copy of the masked interrupt status
        let isr_status = self.event_flags.extract();

        // do we have ISR data collected?
        self.nvic.disable();
        let len = self.isr_len.get();
        if len!= 0 {
            if let Some(mut buf) = self.rx_buf.take() {

                self.isr_buf.map( |isr_buf| {
                    for i in 0..len {
                        buf[i] = isr_buf[i];
                    }
                });
                self.isr_len.set(0);
                self.nvic.enable();

                // allow interrupts to fire as client is called
                self.client.map(move |client| {
                    client.receive_complete(
                        buf,
                        len,
                        kernel::hil::uart::Error::CommandComplete
                    );
                });
                
            }
            else
            {
                self.isr_len.set(0);
            }
        }
        self.nvic.enable();

        self.transaction.take().map(|mut transaction| {
            if transaction.index >= transaction.length {
                self.client.map(move |client| {
                    client.transmit_complete(
                        transaction.buffer,
                        kernel::hil::uart::Error::CommandComplete,
                    );
                });
            }
            else{
                self.transaction.put(transaction);
            }
        });
        events::clear_event_flag(self.event_priority);
        self.enable_interrupts();
    }

    // Pushes a byte into the TX FIFO.
    #[inline]
    pub fn send_byte(&self, c: u8) {
        // Put byte in data register
        self.registers.dr.set(c as u32);
    }

    // Pulls a byte into the RX FIFO.
    #[inline]
    pub fn read_byte(&self) -> u8 {
        self.registers.dr.get() as u8
    }

    /// Checks if there is space in the transmit fifo queue.
    #[inline]
    pub fn rx_fifo_not_empty(&self) -> bool {
        !self.registers.fr.is_set(Flags::RX_FIFO_EMPTY)
    }

    /// Checks if there is space in the transmit fifo queue.
    #[inline]
    pub fn tx_fifo_not_full(&self) -> bool {
        !self.registers.fr.is_set(Flags::TX_FIFO_FULL)
    }
}


impl kernel::hil::uart::UART for UART {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.client.set(client);
    }

    fn configure(&self, params: kernel::hil::uart::UARTParameters) -> ReturnCode {
        self.configure(params)
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        // if there is a weird input event, handle it
        if tx_len == 0 || tx_len > tx_data.len() {
            
            let error;
            
            if tx_len == 0 {
                error =  kernel::hil::uart::Error::CommandComplete
            } else {
                error = kernel::hil::uart::Error::TxLenLargerThanBuffer
            }
            
            self.client.map(move |client| {
                client.transmit_complete(
                    tx_data,
                    error
                );
            });

        // otherwise kick off the transfer
        } else {

            let mut index = 0;
            // we will send at least one byte, causing EOT interrupt
            while index < tx_len && self.tx_fifo_not_full() {
                self.send_byte(tx_data[index]);
                index += 1;
            }

            // EOT interrupt will cause this to be hanled in event_handler
            self.transaction.put(Transaction {
                buffer: tx_data,
                length: tx_len,
                index: index,
            });
        }
        
        
    }

    #[allow(unused)]
    fn receive(&self, rx_buffer: &'static mut [u8], _rx_len: usize) {
        self.rx_buf.put(rx_buffer);
    }

    fn abort_receive(&self) {
        unimplemented!()
    }
}
