//! Prelude

pub use crate::clock::AonExt as _e310x_hal_clock_AonExt;
pub use crate::clock::PrciExt as _e310x_hal_clock_PrciExt;
pub use crate::gpio::GpioExt as _e310x_hal_gpio_GpioExt;
pub use crate::time::U32Ext as _e310x_hal_time_U32Ext;
pub use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin, StatefulOutputPin},
    i2c::I2c as _embedded_hal_i2c_I2c,
    pwm::SetDutyCycle,
    spi::{SpiBus, SpiDevice},
};

pub use embedded_hal_nb::{
    serial::{Read, Write},
    spi::FullDuplex,
};
