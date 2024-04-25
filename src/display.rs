use crate::prelude::*;

use core::fmt::{Debug, Write};

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

const MAX_CHARS: usize =
    (DisplaySize128x64::WIDTH / 8) as usize * (DisplaySize128x64::HEIGHT / 8) as usize;

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
    info!("Initialising screen...");

    let i2c = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0);
    display.init().await?;

    display.clear().await?;

    info!("Screen Initialised!");

    let mut display = display.into_terminal_mode();

    let mut counter: u16 = 0;

    loop {
        display.reset_pos().await?;

        let mut string: String<MAX_CHARS> = String::new();

        string.write_fmt(format_args!("{counter}"))?;

        display.write_str(&string).await?;

        counter = if let Some(next) = counter.checked_add(1) {
            next
        } else {
            display.clear().await?;
            1
        };
    }
}

#[task]
pub async fn screen_counter(mut i2c: SharedI2C) {
    loop {
        if let Err(e) = screen_counter_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        } else {
            unreachable!()
        }

        Timer::after_secs(1).await;
    }
}
