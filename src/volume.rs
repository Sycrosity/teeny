use esp_hal::{
    analog::adc::{AdcCalCurve, AdcPin},
    gpio::{Analog, GpioPin},
};

use crate::prelude::*;

#[task]
pub async fn publish_volume(
    adc: &'static ADCMutex,
    mut pot: AdcPin<
        GpioPin<Analog, 3>,
        esp_hal::peripherals::ADC1,
        AdcCalCurve<esp_hal::peripherals::ADC1>,
    >,
) {
    let mut ticker = Ticker::every(Duration::from_millis(25));

    let publisher = VOLUME_CHANNEL.publisher().unwrap();

    let mut prev_value = 0.;

    loop {
        let value = nb::block!(adc.lock().await.read_oneshot(&mut pot)).unwrap() as f32;

        let value = value / 2754.;

        if value < prev_value - 0.01 || value > prev_value + 0.01 {
            trace!("ADC reading = {}", value);
            publisher.publish_immediate(value);
            prev_value = value;
        }

        ticker.next().await;
    }
}
