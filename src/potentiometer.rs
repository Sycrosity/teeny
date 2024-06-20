use core::ops::Div;

use esp_hal::{
    analog::adc::{AdcChannel, AdcPin},
    gpio::AnalogPin,
    peripherals::ADC1,
};

use crate::prelude::*;

#[cfg(target_arch = "xtensa")]
pub type AdcCal = ();
#[cfg(target_arch = "riscv32")]
pub type AdcCal = esp_hal::analog::adc::AdcCalCurve<esp_hal::peripherals::ADC1>;

#[cfg(target_arch = "xtensa")]
pub struct Potentiometer<PIN> {
    pin: AdcPin<PIN, ADC1>,
    adc: &'static AdcMutex,
    min: u16,
    max: u16,
}

#[cfg(target_arch = "riscv32")]
pub struct Potentiometer<PIN, CS> {
    pin: AdcPin<PIN, ADC1, CS>,
    adc: &'static AdcMutex,
    min: u16,
    max: u16,
}

#[cfg(target_arch = "xtensa")]
impl<PIN> Potentiometer<PIN>
where
    PIN: AnalogPin + AdcChannel,
{
    pub fn new(pin: AdcPin<PIN, ADC1>, adc: &'static AdcMutex, min: u16, max: u16) -> Self {
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

#[cfg(target_arch = "riscv32")]
impl<PIN, CS> Potentiometer<PIN, CS>
where
    PIN: AnalogPin + AdcChannel,
    CS: esp_hal::analog::adc::AdcCalScheme<ADC1>,
{
    pub fn new(pin: AdcPin<PIN, ADC1, CS>, adc: &'static AdcMutex, min: u16, max: u16) -> Self {
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
