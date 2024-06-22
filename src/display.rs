use core::fmt::{Debug, Write};

use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives::PrimitiveStyleBuilder};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use crate::prelude::*;

const MAX_CHARS: usize =
    (DisplaySize128x64::WIDTH / 8) as usize * (DisplaySize128x64::HEIGHT / 8) as usize;

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum DisplayError {
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
    // info!("Initialising screen...");

    let i2c = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0);
    display.init().await?;
    display.clear().await?;
    // info!("Screen Initialised!");

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

        Timer::after_millis(20).await;
    }
}

#[task]
pub async fn screen_counter(mut i2c: SharedI2C) {
    loop {
        if let Err(e) = screen_counter_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        }

        Timer::after_secs(1).await;
    }
}

async fn display_shapes_internal(i2c: &mut SharedI2C) -> Result<(), DisplayError> {
    // info!("Initialising screen...");

    let i2c = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0);
    display.init().await?;
    display.clear().await?;

    // info!("Screen Initialised!");

    let mut display = display.into_buffered_graphics_mode();

    let mut sub = VOLUME_CHANNEL.subscriber().unwrap();

    loop {
        let yoffset = 20;

        let style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(BinaryColor::On)
            .build();

        // screen outline
        // default display size is 128x64 if you don't pass a _DisplaySize_
        // enum to the _Builder_ struct
        embedded_graphics::primitives::Rectangle::new(Point::new(0, 10), Size::new(127, 36))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        // triangle
        embedded_graphics::primitives::Triangle::new(
            Point::new((sub.next_message_pure().await * 128.) as i32, 16 + yoffset),
            Point::new(16 + 16, 16 + yoffset),
            Point::new(16 + 8, yoffset),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        // square
        embedded_graphics::primitives::Rectangle::new(Point::new(52, yoffset), Size::new_equal(16))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        // circle
        embedded_graphics::primitives::Circle::new(Point::new(88, yoffset), 16)
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.flush().await?;

        Timer::after_millis(20).await;
    }
}

#[task]
pub async fn display_shapes(mut i2c: SharedI2C) {
    loop {
        if let Err(e) = display_shapes_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        }

        Timer::after_secs(1).await;
    }
}

pub async fn init_display(i2c: SharedI2C) -> Result<(), DisplayError> {
    info!("Initialising screen...");

    let i2c = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0);
    display.init().await?;
    display.clear().await?;

    info!("Screen Initialised!");

    Ok(())
}

async fn display_volume_internal(i2c: &mut SharedI2C) -> Result<(), DisplayError> {
    // info!("Initialising screen...");

    let i2c = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0);
    display.init().await?;
    display.clear().await?;
    // info!("Screen Initialised!");

    let mut display = display.into_buffered_graphics_mode();

    let mut sub = VOLUME_CHANNEL.subscriber().unwrap();

    loop {
        let on_style = PrimitiveStyleBuilder::new()
            .fill_color(BinaryColor::On)
            .build();

        let off_style = PrimitiveStyleBuilder::new()
            .fill_color(BinaryColor::Off)
            .build();

        embedded_graphics::primitives::Rectangle::new(Point::new(0, 64 - 8), Size::new(128, 8))
            .into_styled(off_style)
            .draw(&mut display)
            .unwrap();

        embedded_graphics::primitives::Rectangle::new(
            Point::new(0, 64 - 8),
            Size::new((sub.next_message_pure().await * 128.) as u32, 8),
        )
        .into_styled(on_style)
        .draw(&mut display)
        .unwrap();

        display.flush().await?;

        Timer::after_millis(20).await;
    }
}

#[task]
pub async fn display_volume(mut i2c: SharedI2C) {
    loop {
        if let Err(e) = display_volume_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        } else {
            unreachable!()
        }

        Timer::after_secs(1).await;
    }
}
