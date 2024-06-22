use crate::prelude::*;

use core::fmt::{Debug, Write};

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[derive(Debug, Clone)]
#[allow(unused)]
enum DisplayError {
    Display(display_interface::DisplayError),
    Format(core::fmt::Error),
    TerminalModeError(ssd1306::mode::TerminalModeError),
}

impl From<display_interface::DisplayError> for DisplayError {
    fn from(value: display_interface::DisplayError) -> Self {
        Self::Display(value)
    }
}
impl From<core::fmt::Error> for DisplayError {
    fn from(value: core::fmt::Error) -> Self {
        Self::Format(value)
    }
}
impl From<ssd1306::mode::TerminalModeError> for DisplayError {
    fn from(value: ssd1306::mode::TerminalModeError) -> Self {
        Self::TerminalModeError(value)
    }
}

async fn screen_counter_internal(i2c: &mut SharedI2C) -> Result<(), DisplayError> {
    error!("init screen");

    let i2c = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0);
    display.init()?;

    display.clear()?;

    let mut display = display.into_terminal_mode();

    let mut counter: u16 = 0;

    loop {
        Timer::after_millis(1).await;

        display.reset_pos()?;

        display.write_fmt(format_args!("{}", counter))?;

        counter = match counter.checked_add(1) {
            Some(next) => next,
            None => {
                display.clear()?;
                1
            }
        };
    }
}

#[task]
pub async fn screen_counter(mut i2c: SharedI2C) {
    loop {
        match screen_counter_internal(&mut i2c).await {
            Ok(_) => unreachable!(),
            Err(e) => warn!("Display error: {e:?}"),
        }

        Timer::after_secs(1).await;
    }
}
