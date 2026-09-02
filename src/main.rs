#![no_std]
#![no_main]

use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Entrypoint");

    let peripherals = embassy_stm32::init(Default::default());
    let daisy = new_daisy_board!(peripherals);
    let mut led = daisy.user_led;

    loop {
        info!("LED on");
        led.on();
        Timer::after_millis(500).await;

        info!("LED off");
        led.off();
        Timer::after_millis(500).await;
    }
}