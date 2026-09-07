use embassy_stm32::i2c::I2c;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use ssd1306::{Ssd1306, prelude::*};

pub struct DisplayContent<'a> {
    pub text: &'a str,
}

pub static DISPLAY: Signal<CriticalSectionRawMutex, DisplayContent> = Signal::new();

const TEXT_STYLE: MonoTextStyle<'_, BinaryColor> = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

#[embassy_executor::task]
pub async fn display_handler(
    mut display: Ssd1306<
        I2CInterface<I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>>,
        DisplaySize128x64,
        ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
    >,
) {
    loop {
        let content = DISPLAY.wait().await;

        display.clear_buffer();
        let _ = Text::with_alignment(
            content.text,
            display.bounding_box().center(),
            TEXT_STYLE,
            Alignment::Center,
        )
        .draw(&mut display);

        display.flush().unwrap();
    }
}
