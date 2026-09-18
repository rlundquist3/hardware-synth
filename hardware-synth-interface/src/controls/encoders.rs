use alloc::format;
use daisy_embassy::hal::{exti::ExtiInput, mode::Async};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use logger::serial_log;

static ENCODER0: Signal<CriticalSectionRawMutex, i32> = Signal::new();

#[derive(Debug)]
pub enum EncoderDirection {
    Clockwise,
    Counterclockwise,
}

const DELAY: Duration = Duration::from_millis(250);

#[embassy_executor::task]
pub async fn encoder_0_handler(mut clk: ExtiInput<'static, Async>, dt: ExtiInput<'static, Async>) {
    let mut start = Instant::now();
    let mut last_clk_state = clk.get_level();
    let mut change = 0;

    loop {
        clk.wait_for_any_edge().await;
        let clk_state = clk.get_level();

        if last_clk_state != clk_state {
            let dt_state = dt.get_level();

            let direction = match clk_state == dt_state {
                true => {
                    change -= 1;
                    EncoderDirection::Counterclockwise
                }
                false => {
                    change += 1;
                    EncoderDirection::Clockwise
                }
            };

            serial_log(&format!("Encoder 0 {:?} {:?}", direction, change));

            if start.elapsed() >= DELAY {
                ENCODER0.signal(change);
                change = 0;
                start = Instant::now();
            }

            // TODO: figure out how to signal this better (likely different struct than Signal) and maybe debounce
        }

        last_clk_state = clk_state;
    }
}

#[embassy_executor::task]
pub async fn encoder_0_receiver_test() {
    loop {
        let value = ENCODER0.wait().await;
        serial_log(&format!("Received {:?}", value));
    }
}

#[embassy_executor::task]
pub async fn encoder_0_click_handler(mut sw: ExtiInput<'static, Async>) {
    loop {
        sw.wait_for_low().await;

        serial_log(&format!("Encoder 0 clicked",));
        Timer::after_millis(250).await;
    }
}
