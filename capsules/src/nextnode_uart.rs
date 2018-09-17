use kernel::hil::uart;
use kernel::hil::uart::{UART, Client};

pub static mut WRITE_BUF: [u8; 64] = [0; 64];
pub static mut READ_BUF: [u8; 64] = [0; 64];

pub struct NextnodeUart<'a, U: UART>{
    uart: &'a U,
    baud_rate: u32,
    tx_buffer: &'static mut [u8],
    rx_buffer: &'static mut [u8],
}

impl<U: UART> NextnodeUart<'a, U> {

    // pub fn new(uart: &'a U, baud_rate: u32) -> NextnodeUart<'a, U>{
    //      debug_verbose!("Here I am.");
    //      NextnodeUart{ uart: &uart }
    // }
}

pub fn test(){
    debug_verbose!("hi");
}

impl<U: UART> Client for NextnodeUart<'a, U> {

    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        
    }

    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, error: uart::Error) {
         debug_verbose!("\r\n{:?}", buffer);


    }
    //     self.rx_in_progress
    //         .take()
    //         .map(|appid| {
    //             self.apps
    //                 .enter(appid, |app, _| {
    //                     app.read_callback.map(|mut cb| {
    //                         // An iterator over the returned buffer yielding only the first `rx_len`
    //                         // bytes
    //                         let rx_buffer = buffer.iter().take(rx_len);
    //                         match error {
    //                             uart::Error::CommandComplete | uart::Error::Aborted => {
    //                                 // Receive some bytes, signal error type and return bytes to process buffer
    //                                 if let Some(mut app_buffer) = app.read_buffer.take() {
    //                                     for (a, b) in app_buffer.iter_mut().zip(rx_buffer) {
    //                                         *a = *b;
    //                                     }
    //                                     let rettype = if error == uart::Error::CommandComplete {
    //                                         ReturnCode::SUCCESS
    //                                     } else {
    //                                         ReturnCode::ECANCEL
    //                                     };
    //                                     cb.schedule(From::from(rettype), rx_len, 0);
    //                                 } else {
    //                                     // Oops, no app buffer
    //                                     cb.schedule(From::from(ReturnCode::EINVAL), 0, 0);
    //                                 }
    //                             }
    //                             _ => {
    //                                 // Some UART error occurred
    //                                 cb.schedule(From::from(ReturnCode::FAIL), 0, 0);
    //                             }
    //                         }
    //                     });
    //                 }).unwrap_or_default();
    //         }).unwrap_or_default();

    //     // Whatever happens, we want to make sure to replace the rx_buffer for future transactions
    //     self.rx_buffer.replace(buffer);
    // }
}

