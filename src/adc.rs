use esp_hal::{
    analog::adc::{
        AdcCalScheme, AdcConfig, AdcPin, Attenuation, CalibrationAccess, RegisterAccess, ADC,
    },
    gpio::{Analog, GpioPin, Pin},
};
use hal::analog::adc::AdcChannel;

use crate::prelude::*;

// pub fn init_adc_pin<const GPIONUM: u8, ADC1>(pin: GpioPin<Analog, GPIONUM>, attenuation: Attenuation ) {

//     // let analog_pin = io.pins.gpio3.into_analog();

//     let mut adc1_config = AdcConfig::new();

//     // type AdcCal = ();
//     // type AdcCal = esp_hal::analog::adc::AdcCalBasic<ADC1>;
//     // type AdcCal = esp_hal::analog::adc::AdcCalLine<ADC1>;
//     // type AdcCal = esp_hal::analog::adc::AdcCalCurve<ADC1>;

//     let mut adc1_pin = adc1_config.enable_pin_with_cal::<_, AdcCal>(analog_pin, attenuation);

//     let mut adc1 = ADC::<ADC1>::new(peripherals.ADC1, adc1_config);

// }

// impl<ADCI> AdcCalScheme<ADCI>
// where
//     ADCI: AdcCalEfuse + AdcHasLineCal + AdcHasCurveCal + CalibrationAccess,

// pub struct AdcController<'d, ADCI>
// where
//     ADCI: RegisterAccess + 'd,
// {
//     // _adc_instance: hal::peripheral::PeripheralRef<'d, ADCI>,
//     adc: Option<ADC<'d, ADCI>>,
//     config: AdcConfig<ADCI>,
// }

// impl<'d, ADCI> AdcController<'d, ADCI>
// where
//     ADCI: RegisterAccess + 'd,
// {
//     pub fn new() -> Self {
//         Self {
//             adc: None,
//             config: AdcConfig::new(),
//         }
//     }

//     pub fn init_pin<PIN>(&mut self, pin: PIN, attenuation: Attenuation) -> AdcPin<PIN, ADCI, ()>
//     where
//         PIN: AdcChannel,
//     {
//         self.config.enable_pin(pin, attenuation)
//     }

//     pub fn init_pin_with_cal<PIN, CS>(
//         &mut self,
//         pin: PIN,
//         attenuation: Attenuation,
//     ) -> AdcPin<PIN, ADCI, CS>
//     where
//         ADCI: CalibrationAccess,
//         PIN: AdcChannel,
//         CS: AdcCalScheme<ADCI>,
//     {
//         self.config.enable_pin_with_cal(pin, attenuation)
//     }

//     pub fn read_pin<PIN, CS>(&mut self, pin: &mut AdcPin<PIN, ADCI, CS>) -> nb::Result<u16, ()>
//     where
//         PIN: AdcChannel,
//         CS: AdcCalScheme<ADCI>,
//     {
//         self.adc.as_mut()
//             .expect("ADC must be initialised before use!")
//             .read_oneshot(pin)
//     }

//     pub fn init_adc<Instance>(&mut self, adc_instance: Instance)
//     where
//         Instance: hal::peripheral::Peripheral<P = ADCI> + 'd,
//     -> Self
//     {
//         // self.adc = Some(ADC::<ADCI>::new(adc_instance, self.config));

//         Self {
//             adc: Some(ADC::<ADCI>::new(adc_instance, self.config)),
//             config: AdcConfig::new(),
//         }
//     }
// }
