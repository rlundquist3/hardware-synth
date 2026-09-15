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
const LOG_CAPACITY: usize = 256;

pub struct LogMessage {
    pub level: LogLevel,
    pub message: heapless::String<LOG_CAPACITY>,
}

impl LogMessage {
    /// Push as much of the intended slice onto the allotted 256 bytes as fits, truncating the remainder
    pub fn new(level: LogLevel, message: &str) -> Self {
        let mut s = heapless::String::<LOG_CAPACITY>::new();
        for c in message.chars() {
            match s.push(c) {
                Ok(()) => continue,
                Err(_) => break,
            }
        }

        LogMessage { level, message: s }
    }
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

pub fn serial_log(message: &str) {
    let _ = LOGGER.try_send(LogMessage::new(LogLevel::Info, message));
}

pub fn serial_error(message: &str) {
    let _ = LOGGER.try_send(LogMessage::new(LogLevel::Error, message));
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
