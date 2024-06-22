use crate::{
    display::{BoundingBox, DisplayError},
    prelude::*,
};

#[rustfmt::skip]
const PLAY_BUTTON_ICON: &[u8] = &[
    0b10000000,
    0b11000000,
    0b11100000,
    0b11110000,
    0b11100000,
    0b11000000,
    0b10000000,
    0b00000000
];

#[rustfmt::skip]
const SKIP_BUTTON_ICON: &[u8] = &[
    0b10001000,
    0b11001000,
    0b11101000,
    0b11111000,
    0b11101000,
    0b11001000,
    0b10001000,
    0b00000000
];

#[rustfmt::skip]
const SKIP_BACK_BUTTON_ICON: &[u8] = &[
    0b10001000,
    0b10011000,
    0b10111000,
    0b11111000,
    0b10111000,
    0b10011000,
    0b10001000,
    0b00000000
];

#[rustfmt::skip]
const PAUSE_BUTTON_ICON: &[u8] = &[
    0b10010000,
    0b10010000,
    0b10010000,
    0b10010000,
    0b10010000,
    0b10010000,
    0b10010000,
    0b00000000
];

pub static PLAY_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[task]
pub async fn publish_play_pause(mut pin: AnyInput<'static>) {
    let signal = &PLAY_SIGNAL;

    loop {
        pin.wait_for_rising_edge().await;
        debug!("PLAY!");

        signal.signal(true);
    }
}

#[task]
pub async fn display_play_pause(mut i2c: SharedI2C) {
    async fn display_play_pause_internal(i2c: &mut SharedI2C) -> Result<(), DisplayError> {
        const BOUNDING_BOX: BoundingBox = BoundingBox::new(Point::new(111, 0), Point::new(118, 7));

        const PLAY_BUTTON: ImageRaw<'static, BinaryColor, BigEndian> =
            ImageRaw::<BinaryColor>::new(PLAY_BUTTON_ICON, 4);

        const PAUSE_BUTTON: ImageRaw<'static, BinaryColor, BigEndian> =
            ImageRaw::<BinaryColor>::new(PAUSE_BUTTON_ICON, 4);

        use embedded_graphics::{
            image::*,
            pixelcolor::{raw::BigEndian, BinaryColor},
            prelude::*,
            primitives::Rectangle,
        };

        let mut display = Ssd1306::new(
            ssd1306::I2CDisplayInterface::new(i2c),
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        );
        display.init().await?;
        let mut display = display.into_buffered_graphics_mode();

        let signal = &PLAY_SIGNAL;

        loop {
            display.fill_solid(
                &Rectangle::with_corners(BOUNDING_BOX.start, BOUNDING_BOX.end),
                BinaryColor::Off,
            )?;

            let play = signal.wait().await;

            let raw_image = if play { PLAY_BUTTON } else { PAUSE_BUTTON };

            Image::new(&raw_image, BOUNDING_BOX.start).draw(&mut display)?;

            display.flush().await?;

            Timer::after_millis(250).await;

            display.fill_solid(
                &Rectangle::with_corners(BOUNDING_BOX.start, BOUNDING_BOX.end),
                BinaryColor::Off,
            )?;

            display.flush().await?;
        }
    }
    loop {
        if let Err(e) = display_play_pause_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        } else {
            unreachable!()
        }

        Timer::after_secs(1).await;
    }
}

pub static RAW_SKIP_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[task]
pub async fn publish_raw_skip(mut pin: AnyInput<'static>) {
    let signal = &RAW_SKIP_SIGNAL;

    loop {
        pin.wait_for_rising_edge().await;

        signal.signal(());
    }
}

#[derive(Debug)]
pub enum SkipType {
    Skip,
    SkipBack,
}

pub static SKIP_SIGNAL: Signal<CriticalSectionRawMutex, crate::buttons::SkipType> = Signal::new();

#[task]
pub async fn publish_skip() {
    let raw_signal = &RAW_SKIP_SIGNAL;
    let signal = &SKIP_SIGNAL;

    loop {
        raw_signal.wait().await;
        // raw_signal.reset();

        // Timer::after_millis(250).await;

        // let skip = match raw_signal.signaled() {
        //     true => Skip::SkipBack,
        //     false => Skip::Skip,
        // };

        // Timer::after_millis(100).await;

        // raw_signal.reset();

        signal.signal(SkipType::Skip);
    }
}

#[task]
pub async fn display_skip(mut i2c: SharedI2C) {
    async fn display_skip_internal(i2c: &mut SharedI2C) -> Result<(), DisplayError> {
        const BOUNDING_BOX: BoundingBox = BoundingBox::new(Point::new(120, 0), Point::new(127, 7));

        const SKIP_BUTTON: ImageRaw<'static, BinaryColor, BigEndian> =
            ImageRaw::<BinaryColor>::new(SKIP_BUTTON_ICON, 5);

        const SKIPBACK_BUTTON: ImageRaw<'static, BinaryColor, BigEndian> =
            ImageRaw::<BinaryColor>::new(SKIP_BACK_BUTTON_ICON, 5);

        use embedded_graphics::{
            image::*,
            pixelcolor::{raw::BigEndian, BinaryColor},
            prelude::*,
            primitives::Rectangle,
        };

        let mut display = Ssd1306::new(
            ssd1306::I2CDisplayInterface::new(i2c),
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        );
        display.init().await?;
        let mut display = display.into_buffered_graphics_mode();

        let signal = &SKIP_SIGNAL;

        loop {
            display.fill_solid(
                &Rectangle::with_corners(BOUNDING_BOX.start, BOUNDING_BOX.end),
                BinaryColor::Off,
            )?;

            let skip = signal.wait().await;
            error!("SKIP!");

            let raw_image = match skip {
                SkipType::Skip => SKIP_BUTTON,
                SkipType::SkipBack => SKIPBACK_BUTTON,
            };

            Image::new(&raw_image, BOUNDING_BOX.start).draw(&mut display)?;
            display.flush().await?;

            Timer::after_millis(250).await;

            display.fill_solid(
                &Rectangle::with_corners(BOUNDING_BOX.start, BOUNDING_BOX.end),
                BinaryColor::Off,
            )?;

            display.flush().await?;
        }
    }

    loop {
        if let Err(e) = display_skip_internal(&mut i2c).await {
            warn!("Display error: {e:?}");
        }

        Timer::after_secs(1).await;
    }
}
