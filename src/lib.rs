#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]
#![allow(clippy::unused_unit, clippy::const_is_empty)]

// #[cfg(feature = "adc")]
// pub mod adc;
#[cfg(feature = "alloc")]
pub mod alloc;
pub mod blink;
pub mod display;
pub mod errors;

pub mod logger;
#[cfg(feature = "net")]
pub mod net;

pub mod prelude {

    pub const SSID: &str = env!("SSID");
    pub const PASSWORD: &str = env!("PASSWORD");

    pub const TICKS_PER_SECOND: u64 = 16_000_000;

    pub use core::f64::consts::PI;

    pub use crate::errors::*;

    pub use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
    pub use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex, signal::Signal};

    pub type SharedI2C =
        I2cDevice<'static, NoopRawMutex, I2C<'static, esp_hal::peripherals::I2C0, Async>>;

    #[allow(unused)]
    pub use esp_backtrace as _;

    pub use esp_println::{print, println};

    pub use embassy_executor::task;

    pub use esp_hal::{
        embassy,
        gpio::{AnyPin, Output, PushPull},
        i2c::I2C,
        prelude::*,
        Async,
    };

    pub use heapless::String;

    pub use nb::block;

    pub use embassy_time::{Delay, Duration, Instant, Ticker, Timer};

    pub use log::{debug, error, info, log, trace, warn};

    pub use static_cell::make_static;

    pub use ssd1306::prelude::*;
}
