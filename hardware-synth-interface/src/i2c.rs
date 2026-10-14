use core::cell::RefCell;

use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_stm32::i2c::I2c;
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use static_cell::StaticCell;

pub type I2cBus = I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>;

pub type SharedI2c = I2cDevice<'static, CriticalSectionRawMutex, I2cBus>;

pub static I2C_BUS: StaticCell<BlockingMutex<CriticalSectionRawMutex, RefCell<I2cBus>>> =
    StaticCell::new();
