#![feature(const_fn)]
#![feature(infer_outlives_requirements, in_band_lifetimes)]
#![feature(tool_attributes)]
#![forbid(unsafe_code)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;
extern crate fixedvec;
extern crate cc26x0;
#[macro_use]
extern crate bitfield;
pub mod test;

#[macro_use]
pub mod net;

pub mod adc;
pub mod aes_ccm;
pub mod alarm;
pub mod ambient_light;
pub mod analog_comparator;
pub mod app_flash_driver;
pub mod ble_advertising_driver;
pub mod button;
pub mod console;
pub mod crc;
pub mod dac;
pub mod fm25cl;
pub mod fxos8700cq;
pub mod gpio;
pub mod gpio_async;
pub mod humidity;
pub mod i2c_master_slave_driver;
pub mod ieee802154;
pub mod isl29035;
pub mod led;
pub mod lps25hb;
pub mod ltc294x;
pub mod max17205;
pub mod mcp230xx;
pub mod mx25r6435f;
pub mod ninedof;
pub mod nonvolatile_storage_driver;
pub mod nonvolatile_to_pages;
pub mod nrf51822_serialization;
pub mod pca9544a;
pub mod rf233;
pub mod rf233_const;
pub mod rng;
pub mod sdcard;
pub mod segger_rtt;
pub mod si7021;
pub mod spi;
pub mod temperature;
pub mod tmp006;
pub mod tsl2561;
pub mod usb;
pub mod usb_user;
pub mod usbc_client;
pub mod virtual_alarm;
pub mod virtual_flash;
pub mod virtual_i2c;
pub mod virtual_spi;
pub mod virtual_uart;
