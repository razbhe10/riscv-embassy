#![no_std]

// lib.rs or main.rs

use device::DeviceResources;
// Import time driver
mod time_driver;

pub mod clock;
pub mod core;
pub mod device;
pub mod gpio;
pub mod prelude;
pub mod time;
pub mod uart;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn init() -> DeviceResources {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = DeviceResources::take().expect("Failed to take DeviceResources");

    // unsafe {
    //     if let Err(e) = clock::init(config.clocks) {
    //         error!("unable to initialize Clocks for reason: {:?}", e);
    //         // Panic here?
    //     }
    //     #[cfg(feature = "time-driver")]
    //     time_driver::init(config.time_interrupt_priority);
    // }

    peripherals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
