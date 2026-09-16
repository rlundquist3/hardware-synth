use alloc::{collections::VecDeque, string::String};
use embassy_stm32::i2c::I2c;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use ssd1306::{Ssd1306, prelude::*};

pub struct DisplayContent {
    pub text: String,
}

pub static DISPLAY: Channel<CriticalSectionRawMutex, DisplayContent, 2> = Channel::new();

const TEXT_STYLE: MonoTextStyle<'_, BinaryColor> = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

#[embassy_executor::task]
pub async fn display_handler(
    mut display: Ssd1306<
        I2CInterface<I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>>,
        DisplaySize128x64,
        ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
    >,
) {
    let mut displayed_messages: VecDeque<String> = VecDeque::new();

    loop {
        let content = DISPLAY.receive().await;

        if displayed_messages.len() > 6 {
            displayed_messages.pop_front();
        }
        displayed_messages.push_back(content.text);

        display.clear_buffer();

        displayed_messages
            .iter()
            .enumerate()
            .for_each(|(i, message)| {
                let _ = Text::with_alignment(
                    message,
                    Point {
                        x: 0,
                        y: (10 * i as i32),
                    },
                    TEXT_STYLE,
                    Alignment::Left,
                )
                .draw(&mut display);
            });

        display.flush().unwrap();
    }
}
