use alloc::{format, string::String};
use core::{
    fmt::{self, Write},
    write,
};
use embassy_stm32::usart::UartTx;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

#[derive(Debug)]
pub enum LogLevel {
    Info,
    Error,
}
pub struct LogMessage<'a> {
    pub level: LogLevel,
    pub message: &'a str,
}

impl fmt::Display for LogMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.level {
            LogLevel::Info => f.write_str(self.message),
            LogLevel::Error => f.write_str(&format!("ERROR: {}", self.message)),
        }
    }
}

pub static LOGGER: Channel<CriticalSectionRawMutex, LogMessage, 5> = Channel::new();

#[embassy_executor::task]
pub async fn log_handler(mut logger: UartTx<'static, embassy_stm32::mode::Blocking>) {
    let mut s = String::new();

    loop {
        let message = LOGGER.receive().await;
        write!(&mut s, "{}\r\n", message).ok();

        logger.blocking_write(s.as_bytes()).ok();
        logger.blocking_flush().ok();
        s.clear();
    }
}
