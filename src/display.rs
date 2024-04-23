use crate::prelude::*;

use core::fmt::{Debug, Write};

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[task]
pub async fn screen_counter(i2c: SharedI2C) {
    let i2c = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0);
    while let Err(e) = display.init() {
        warn!("{e:?}");
    }

    display.clear().unwrap();

    let mut display = display.into_terminal_mode();

    let mut counter: u16 = 0;

    loop {
        Timer::after_millis(1).await;

        handle_errors(display.reset_pos());

        handle_errors(display.write_fmt(format_args!("{}", counter)));

        counter = match counter.checked_add(1) {
            Some(next) => next,
            None => {
                handle_errors(display.clear());
                1
            }
        };
    }
}

fn handle_errors<E: Debug>(result: Result<(), E>) {
    if let Err(e) = result {
        warn!("{e:?}");
    }
}
