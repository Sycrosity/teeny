use esp_hal::gpio::AnyOutput;

use crate::prelude::*;

#[task]
pub async fn blink(mut led: AnyOutput<'static>) {
    loop {
        led.toggle();

        if led.is_set_high() {
            trace!("ON!");
        } else {
            trace!("OFF!");
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}
