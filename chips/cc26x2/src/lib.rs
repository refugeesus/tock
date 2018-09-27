#![feature(const_fn, untagged_unions, used, asm, core_intrinsics, naked_functions)]
#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]

#[macro_use]
extern crate cortexm4;
extern crate cortexm;
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;
#[macro_use] extern crate enum_primitive;
extern crate num_traits;

pub mod aon;
pub mod chip;
pub mod crt1;
pub mod gpio;
pub mod i2c;
pub mod events;
pub mod prcm;
pub mod rtc;
pub mod trng;
pub mod uart;


pub use crt1::init;
