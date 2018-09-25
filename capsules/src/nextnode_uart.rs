use kernel::hil::uart;
use kernel::hil::uart::{UART, Client};
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

const DEFAULT_BAUD: u32 = 115200;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    //subscribed: bool,
}

pub struct NextnodeUart<'a, U: UART>{
    uart: &'a U,
    baud: u32,
    apps: Grant<App>,
}

impl<U: UART> NextnodeUart<'a, U> {

    pub fn new(
        uart: &'a U,
        grant: Grant<App>,
    ) -> NextnodeUart<'a, U>{
        uart.configure(uart::UARTParameters {
            baud_rate: DEFAULT_BAUD,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
        NextnodeUart{ uart: &uart , baud: DEFAULT_BAUD, apps: grant}
    }

    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        debug!("Subscribing cb");
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            }).unwrap_or_else(|err| err.into())
    }

}

impl<U: UART> Client for NextnodeUart<'a, U> {

    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        
    }

    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, error: uart::Error) {
        for n in 0..rx_len {
            debug_str!("{}", buffer[n] as char);
            for cntr in self.apps.iter() {
                
                cntr.enter(|app, _| {
                    //if app.subscribed {
                        //app.subscribed = false;
                        app.callback.map(|mut cb| cb.schedule(buffer[n] as usize, 0, 0));
                    //}
                });
            }
        }

         self.uart.receive(buffer, 4);
    }


}



/// Syscall Number
pub const DRIVER_NUM: usize = 0xA3_0000;
const RECEIVE_BYTE: usize = 1;

// System Call implementation
impl<U: UART> Driver for NextnodeUart<'a, U> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        debug!("Received subscribe request");

        match subscribe_num {
            // subscribe to temperature reading with callback
            RECEIVE_BYTE => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // check whether the driver exists!!
            0 => ReturnCode::SUCCESS,
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}