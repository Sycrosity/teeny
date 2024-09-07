use crate::prelude::*;

#[task]
pub async fn wait_for_improv() {
    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
