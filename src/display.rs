use core::fmt::{Debug, Write};

use embedded_graphics::{
    mono_font::{iso_8859_14::FONT_4X6, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use crate::prelude::*;

const MAX_CHARS: usize = (DisplaySize128x64::WIDTH / 4) as usize;

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum TeenyDisplayError {
    Display(display_interface::DisplayError),
    Format(core::fmt::Error),
    TerminalModeError(ssd1306::mode::TerminalModeError),
}

impl From<display_interface::DisplayError> for TeenyDisplayError {
    fn from(value: display_interface::DisplayError) -> Self {
        Self::Display(value)
    }
}

impl From<core::fmt::Error> for TeenyDisplayError {
    fn from(value: core::fmt::Error) -> Self {
        Self::Format(value)
    }
}

impl From<ssd1306::mode::TerminalModeError> for TeenyDisplayError {
    fn from(value: ssd1306::mode::TerminalModeError) -> Self {
        Self::TerminalModeError(value)
    }
}

// pub async fn clear(
//     display: &mut Ssd1306<
//         I2CInterface<
//             &mut I2cDevice<'static, NoopRawMutex, I2C<'static,
// esp_hal::peripherals::I2C0, Async>>,         >,
//         DisplaySize128x64,
//         ssd1306::mode::BasicMode,
//     >,
// ) -> Result<(), DisplayError> {
//     use ssd1306::command::AddrMode;

//     let old_addr_mode = self.addr_mode;
//     if old_addr_mode != AddrMode::Horizontal {
//         self.set_addr_mode(AddrMode::Horizontal).await?;
//     }

//     let dim = self.dimensions();
//     self.set_draw_area((0, 0), dim).await?;

//     let num_pixels = dim.0 as u16 * dim.1 as u16;

//     const BITS_PER_BYTE: u16 = 8;
//     const BYTES_PER_BATCH: u16 = 64;
//     const PIXELS_PER_BATCH: u16 = BITS_PER_BYTE * BYTES_PER_BATCH;

//     // Not all screens have number of pixels divisible by 512, so add 1 to
// cover tail     let num_batches = num_pixels / PIXELS_PER_BATCH + 1;

//     for _ in 0..num_batches {
//         self.draw(&[0; BYTES_PER_BATCH as usize]).await?;
//     }

//     if old_addr_mode != AddrMode::Horizontal {
//         self.set_addr_mode(old_addr_mode).await?;
//     }

//     Ok(())
// }

pub struct BoundingBox {
    pub start: Point,
    pub end: Point,
}

impl BoundingBox {
    pub const fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    pub const fn start(&self) -> (u8, u8) {
        (self.start.x as u8, self.end.y as u8)
    }

    pub const fn end(&self) -> (u8, u8) {
        (self.end.x as u8, self.end.y as u8)
    }
}

#[task]
pub async fn screen_counter(mut i2c: SharedI2C) {
    async fn screen_counter_internal(i2c: &mut SharedI2C) -> Result<(), TeenyDisplayError> {
        const BOUNDING_BOX: BoundingBox = BoundingBox::new(Point::new(0, 0), Point::new(64, 6));

        let mut display = Ssd1306::new(
            I2CDisplayInterface::new(i2c),
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        );
        display.init().await?;
        display.clear().await?;

        let mut display = display.into_buffered_graphics_mode();

        let text_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);

        let mut counter: u16 = 0;

        let mut ticker = Ticker::every(Duration::from_millis(20));

        loop {
            let mut string: String<MAX_CHARS> = String::new();

            string.write_fmt(format_args!("{counter}"))?;

            //overwrite previous text
            display.fill_solid(
                &Rectangle::with_corners(BOUNDING_BOX.start, BOUNDING_BOX.end),
                BinaryColor::Off,
            )?;

            Text::new(&string, Point::new(0, 6), text_style).draw(&mut display)?;

            display.flush().await?;

            counter = if let Some(next) = counter.checked_add(1) {
                next
            } else {
                display.clear(BinaryColor::Off)?;
                1
            };

            ticker.next().await;
        }
    }

    loop {
        if let Err(e) = screen_counter_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        }

        Timer::after_secs(1).await;
    }
}

#[task]
pub async fn display_shapes(mut i2c: SharedI2C) {
    async fn display_shapes_internal(i2c: &mut SharedI2C) -> Result<(), TeenyDisplayError> {
        let mut display = Ssd1306::new(
            I2CDisplayInterface::new(i2c),
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        );
        display.init().await?;

        let mut display = display.into_buffered_graphics_mode();

        let y_offset = 20;

        let style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(BinaryColor::On)
            .build();

        let mut sub = VOLUME_CHANNEL.subscriber().unwrap();

        loop {
            let volume = sub.next_message_pure().await;

            // screen outline
            embedded_graphics::primitives::Rectangle::new(Point::new(0, 10), Size::new(127, 36))
                .into_styled(style)
                .draw(&mut display)?;

            // triangle
            embedded_graphics::primitives::Triangle::new(
                Point::new((volume * 128.) as i32, 16 + y_offset),
                Point::new(16 + 16, 16 + y_offset),
                Point::new(16 + 8, y_offset),
            )
            .into_styled(style)
            .draw(&mut display)?;

            // square
            embedded_graphics::primitives::Rectangle::new(
                Point::new(52, y_offset),
                Size::new_equal(16),
            )
            .into_styled(style)
            .draw(&mut display)?;

            // circle
            embedded_graphics::primitives::Circle::new(Point::new(88, y_offset), 16)
                .into_styled(style)
                .draw(&mut display)?;

            display.flush().await?;
        }
    }

    loop {
        if let Err(e) = display_shapes_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        }

        Timer::after_secs(1).await;
    }
}
