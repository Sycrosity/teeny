use esp_hal::{
    analog::adc::{AdcCalCurve, AdcPin},
    gpio::{Analog, GpioPin},
};

use crate::{display::DisplayError, potentiometer::Potentiometer, prelude::*};

#[task]
pub async fn publish_volume(
    adc: &'static ADCMutex,
    pot: AdcPin<
        GpioPin<Analog, 3>,
        esp_hal::peripherals::ADC1,
        AdcCalCurve<esp_hal::peripherals::ADC1>,
    >,
) {
    let mut pot = Potentiometer::new(pot, adc, 0, 2754);

    let mut ticker = Ticker::every(Duration::from_millis(25));

    let publisher = VOLUME_CHANNEL.publisher().unwrap();

    let mut prev_value = 0.;

    loop {
        let avg_value = pot.read().await as f32 / pot.max() as f32;

        ticker.next().await;

        if avg_value < prev_value - 0.02 || avg_value > prev_value + 0.02 {
            trace!("ADC reading = {}", avg_value);
            publisher.publish_immediate(avg_value);
            prev_value = avg_value;
        }
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

async fn display_volume_internal(i2c: &mut SharedI2C) -> Result<(), DisplayError> {
    use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives};

    let mut display = Ssd1306::new(
        ssd1306::I2CDisplayInterface::new(i2c),
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    );
    display.init().await?;
    let mut display = display.into_buffered_graphics_mode();

    let on_style = primitives::PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::On)
        .build();

    let off_style = primitives::PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::Off)
        .build();

    let mut sub = VOLUME_CHANNEL.subscriber().unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(5));

    loop {
        let volume = sub.next_message_pure().await;

        primitives::Rectangle::new(Point::new(0, 64 - 8), Size::new(128, 8))
            .into_styled(off_style)
            .draw(&mut display)
            .unwrap();

        primitives::Rectangle::new(Point::new(0, 64 - 8), Size::new((volume * 128.) as u32, 8))
            .into_styled(on_style)
            .draw(&mut display)
            .unwrap();

        display.flush().await?;

        ticker.next().await;
    }
}
