use crate::prelude::*;

#[task]
pub async fn blink(mut led: AnyPin<Output<PushPull>>) {
    loop {
        led.toggle();

        if led.is_set_high() {
            info!("ON!");
        } else {
            info!("OFF!");
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}
