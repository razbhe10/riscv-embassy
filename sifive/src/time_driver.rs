// time_driver.rs

use embassy_time_driver::{AlarmHandle, Driver};
use hifive1::hal::e310x::CLINT;

struct E310xTimeDriver {}

// Register the driver using the time_driver_impl macro
embassy_time_driver::time_driver_impl!(static DRIVER: E310xTimeDriver = E310xTimeDriver{});

impl Driver for E310xTimeDriver {
    fn now(&self) -> u64 {
        // Return the current time in ticks
        CLINT::mtime().read()
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        Some(AlarmHandle::new(0))
    }

    fn set_alarm_callback(&self, _alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        // Set the alarm callback
        unsafe {
            ALARM_CALLBACK = Some(callback);
            ALARM_CTX = ctx;
        }
    }

    fn set_alarm(&self, _alarm: AlarmHandle, timestamp: u64) -> bool {
        // Schedule the wake-up event
        // Set the mtimecmp register to the specified timestamp
        CLINT::mtimecmp0().write(timestamp);

        // Enable timer interrupt
        unsafe { CLINT::mtimer_enable() };
        true
    }
}

static mut ALARM_CALLBACK: Option<fn(*mut ())> = None;
static mut ALARM_CTX: *mut () = core::ptr::null_mut();
