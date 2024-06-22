use core::ops::Div;

use esp_hal::{
    analog::adc::{AdcCalScheme, AdcChannel, AdcPin},
    peripherals::ADC1,
};

use crate::prelude::*;

pub struct Potentiometer<PIN, CS> {
    pin: AdcPin<PIN, ADC1, CS>,
    adc: &'static ADCMutex,
    min: u16,
    max: u16,
}

impl<PIN, CS> Potentiometer<PIN, CS>
where
    PIN: AdcChannel,
    CS: AdcCalScheme<ADC1>,
{
    pub fn new(pin: AdcPin<PIN, ADC1, CS>, adc: &'static ADCMutex, min: u16, max: u16) -> Self {
        Self { pin, min, max, adc }
    }

    pub async fn read(&mut self) -> u16 {
        (nb::block!(self.adc.lock().await.read_oneshot(&mut self.pin)).unwrap())
            .saturating_sub(self.min)
    }

    pub async fn normalised(&mut self) -> f32 {
        (self.read().await.saturating_sub(self.min) as f32).div(self.max as f32)
    }

    pub const fn max(&self) -> u16 {
        self.max
    }

    pub const fn min(&self) -> u16 {
        self.min
    }
}
