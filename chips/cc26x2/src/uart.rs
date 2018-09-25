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
        // CTSIMM OFFSET(1) NUMBITS(1) [], // clear to send interrupt mask
        RX OFFSET(4) NUMBITS(1) [],   // receive interrupt mask
        TX OFFSET(5) NUMBITS(1) [],   // transmit interrupt mask
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],   // receive timeout interrupt mask
        // FE OFFSET(7) NUMBITS(1) [],   // framing error interrupt mask
        // PE OFFSET(8) NUMBITS(1) [],   // parity error interrupt mask
        // BE OFFSET(9) NUMBITS(1) [],   // break error interrupt mask
        // OE OFFSET(10) NUMBITS(1) [],  // overrun error interrupt mask
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
use peripheral_interrupts;
pub static UART0_NVIC: nvic::Nvic = unsafe { nvic::Nvic::new(peripheral_interrupts::UART0) };
pub static UART1_NVIC: nvic::Nvic = unsafe { nvic::Nvic::new(peripheral_interrupts::UART1) };

pub static mut UART0_RX_BUF: [u8; 4] = [0; 4];
pub static mut UART1_RX_BUF: [u8; 4] = [0; 4];

pub static mut UART0: UART = UART::new(&UART0_BASE, &UART0_NVIC);
pub static mut UART1: UART = UART::new(&UART1_BASE, &UART1_NVIC);

pub static mut UART1_ISR_RX_BUF: [u8; 4] = [0; 4];
pub static mut UART1_ISR_RX_LEN: usize = 0;

use cortexm4::simple_isr;



// handle RX interrupt
pub extern "C" fn uart1_isr() {
    unsafe { 
        simple_isr();

        // get a copy of the masked interrupt status
        let isr_status = UART1.registers.mis.extract();
        if (isr_status.read(Interrupts::RX) != 0) ||  (isr_status.read(Interrupts::RX_TIMEOUT) != 0){
                loop {
                    let read_byte = UART1.registers.dr.get();
                    let cur_byte = read_byte as u8;
               
                        UART1_ISR_RX_BUF[UART1_ISR_RX_LEN] = cur_byte;
                        UART1_ISR_RX_LEN += 1;
                   
                    if UART1.registers.fr.read(Flags::RX_FIFO_EMPTY) != 0 {
                        break;
                    }
                }
        }

        UART1.nvic.clear_pending();
        UART1.registers.icr.write(Interrupts::RX.val(1));
        UART1.nvic_event.set(true) 
   };
}

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
    pub nvic: &'static nvic::Nvic,
    pub nvic_event: VolatileCell<bool>
}

impl UART {
    const fn new(base_reg: &'static StaticRef<UartRegisters>, nvic: &'static nvic::Nvic) -> UART {
        UART {
            registers: base_reg,
            client: OptionalCell::empty(),
            transaction: MapCell::empty(),
            rx_buf: MapCell::empty(),
            nvic: nvic,
            nvic_event: VolatileCell::new(false)
        }
    }

    /// Initialize the UART hardware.
    ///
    /// This function needs to be run before the UART module is used.
    pub fn initialize(&self) {
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

    // pub fn set_custom_nvic(& self) {
    //     self.nvic.disable();
    //     unsafe { crt1::BASE_VECTORS[self.nvic.get_index()] = UART1_ISR };
    //     self.nvic.clear_pending();
    //     self.nvic.disable();
    // }

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

    pub fn rx_only_int(&self) {
        // clear all
        self.registers.icr.write(Interrupts::ALL_INTERRUPTS::Clear);
        // disable all interrupts
        self.registers.imsc.modify(Interrupts::ALL_INTERRUPTS::Set);

        // self.registers.imsc.write(Interrupts::ALL_INTERRUPTS::Clear);
        // self.registers.imsc.write(Interrupts::RX.val(1));

    }


    pub fn enable_interrupts(&self) {
        // clear all
        self.registers.icr.write(Interrupts::ALL_INTERRUPTS::Clear);
        // set all interrupts
        self.registers.imsc.modify(Interrupts::ALL_INTERRUPTS::Set);
    }

    // clears all interrupts related to UART.
    pub fn handle_interrupt(&self) {
        // get a copy of the masked interrupt status
        let isr_status = self.registers.mis.extract();

        // handle RX interrupt
        if (isr_status.read(Interrupts::RX) != 0) ||  (isr_status.read(Interrupts::RX_TIMEOUT) != 0){

                let mut rx_buf = self.rx_buf.take();
                let mut rx_len = 0;
                loop {
                    let read_byte = self.registers.dr.get();
                    // top 4 bits are status
                    if ((read_byte >> 8) & 0xF) != 0 {
                        //panic!("We have a UART Overrun, Break, Parity, or Framing Error")
                    }
                    let cur_byte = read_byte as u8;

                    if let Some(ref mut buf) = rx_buf {
                        buf[rx_len] = cur_byte;
                        rx_len += 1;
                    }
                    
                    if self.registers.fr.read(Flags::RX_FIFO_EMPTY) != 0 {
                        break;
                    }
                }

                if let Some(mut buf) = rx_buf {
                    self.client.map(move |client| {
                        client.receive_complete(
                            buf,
                            rx_len,
                            kernel::hil::uart::Error::CommandComplete
                        );
                    });
                }
        }
        // else assumed to be write
        else {
            self.transaction.take().map(|mut transaction| {
                transaction.index += 1;
                if transaction.index < transaction.length {
                    self.send_byte(transaction.buffer[transaction.index]);
                    self.transaction.put(transaction);
                } else {
                    self.client.map(move |client| {
                        client.transmit_complete(
                            transaction.buffer,
                            kernel::hil::uart::Error::CommandComplete,
                        );
                    });
                }
            });
            
        }

        // Clear interrupts
        self.registers.icr.write(Interrupts::ALL_INTERRUPTS::Set);

    }

    pub fn handle_event(&self) {

        if let Some(mut buf) = self.rx_buf.take() {
            let len;
            unsafe { 
                len = UART1_ISR_RX_LEN;
                for i in 0..len {
                    buf[i] = UART1_ISR_RX_BUF[i];
                }
                UART1_ISR_RX_LEN = 0;
            }
            self.nvic.enable();

            self.client.map(move |client| {
                client.receive_complete(
                    buf,
                    len,
                    kernel::hil::uart::Error::CommandComplete
                );
            });

            
        }
    }

    /// Transmits a single byte if the hardware is ready.
    pub fn send_byte(&self, c: u8) {
        // Put byte in data register
        self.registers.dr.set(c as u32);
    }

    /// Checks if there is space in the transmit fifo queue.
    pub fn tx_ready(&self) -> bool {
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
        if tx_len > 0 && tx_data.len() > 0 {
            self.send_byte(tx_data[0]);
        }

        self.transaction.put(Transaction {
            buffer: tx_data,
            length: tx_len,
            index: 0,
        });
    }

    #[allow(unused)]
    fn receive(&self, rx_buffer: &'static mut [u8], _rx_len: usize) {
        self.rx_buf.put(rx_buffer);
    }

    fn abort_receive(&self) {
        unimplemented!()
    }
}
