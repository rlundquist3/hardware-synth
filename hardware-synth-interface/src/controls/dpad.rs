use alloc::format;
use daisy_embassy::hal::{exti::ExtiInput, mode::Async};
use embassy_time::Timer;
use embedded_hal::i2c::I2c as _;

use crate::{
    controls::{
        CONTROL_BUFFER, ControlEvent,
        ControlEvent::{
            NavigationDown, NavigationEnter, NavigationLeft, NavigationRight, NavigationUp,
        },
    },
    i2c::SharedI2c,
};

const DPAD_ADDRESS: u8 = 0x20;
const INPUT_PORT: u8 = 0x00;
const DEBOUNCE_MS: u64 = 5;

const BUTTONS: [(u8, ControlEvent); 5] = [
    (0b0000_0001, NavigationUp),
    (0b0000_0010, NavigationDown),
    (0b0000_0100, NavigationRight),
    (0b0000_1000, NavigationLeft),
    (0b0001_0000, NavigationEnter),
];

#[embassy_executor::task]
pub async fn dpad_handler(mut interrupt: ExtiInput<'static, Async>, mut i2c: SharedI2c) {
    let mut previous = 0xFF;

    loop {
        interrupt.wait_for_low().await;
        Timer::after_millis(DEBOUNCE_MS).await;

        let mut input = [0u8; 1];
        if i2c
            .write_read(DPAD_ADDRESS, &[INPUT_PORT], &mut input)
            .is_err()
        {
            Timer::after_millis(50).await;
            continue;
        }

        let current = input[0];

        for (mask, event) in BUTTONS {
            let newly_pressed = previous & mask != 0 && current & mask == 0;

            if newly_pressed {
                CONTROL_BUFFER.send(event).await;
            }
        }

        previous = current;
    }
}
