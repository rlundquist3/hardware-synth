use alloc::string::String;
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
pub struct LogMessage {
    pub level: LogLevel,
    pub message: heapless::String<128>,
}

pub fn serial_log(message: &str) {
    let mut s = heapless::String::<128>::new();
    let _ = s.push_str(message);

    let _ = LOGGER.try_send(LogMessage {
        level: LogLevel::Info,
        message: s,
    });
}

pub fn serial_error(message: &str) {
    let mut s = heapless::String::<128>::new();
    let _ = s.push_str(message);

    let _ = LOGGER.try_send(LogMessage {
        level: LogLevel::Error,
        message: s,
    });
}

impl fmt::Display for LogMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.level {
            LogLevel::Info => f.write_str(&self.message),
            LogLevel::Error => {
                f.write_str("ERROR: ")?;
                f.write_str(&self.message)
            }
        }
    }
}

pub static LOGGER: Channel<CriticalSectionRawMutex, LogMessage, 20> = Channel::new();

#[embassy_executor::task]
pub async fn log_handler(mut logger: UartTx<'static, embassy_stm32::mode::Blocking>) {
    let mut log_writer = String::new();

    loop {
        let message = LOGGER.receive().await;
        write!(&mut log_writer, "{}\r\n", message).ok();

        logger.blocking_write(log_writer.as_bytes()).ok();
        logger.blocking_flush().ok();
        log_writer.clear();
    }
}
