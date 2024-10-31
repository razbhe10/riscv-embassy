// time_driver.rs

use e310x::Clint as CLINT;
use embassy_time_driver::{AlarmHandle, Driver};
use hifive1::hal::{e310x, DeviceResources};
use riscv::register::mie;

struct E310xTimeDriver {}

// Register the driver using the time_driver_impl macro
embassy_time_driver::time_driver_impl!(static DRIVER: E310xTimeDriver = E310xTimeDriver{});

impl Driver for E310xTimeDriver {
    fn now(&self) -> u64 {
        // Return the current time in ticks
        let clint = unsafe { CLINT::steal() };
        clint.mtime().read().bits() as u64
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
        let dr = unsafe { DeviceResources::steal() };
        let mut clint = dr.core_peripherals.clint;
        clint.mtimecmp.set_mtimecmp(timestamp);
        // let clint = unsafe { CLINT::steal() };
        // clint
        //     .mtimecmp()
        //     .write(|w| unsafe { w.bits(timestamp as u32) });
        // clint
        //     .mtimecmph()
        //     .write(|w| unsafe { w.bits((timestamp >> 32) as u32) });

        // Enable timer interrupt
        unsafe {
            mie::set_mtimer();
        };
        true
    }
}

static mut ALARM_CALLBACK: Option<fn(*mut ())> = None;
static mut ALARM_CTX: *mut () = core::ptr::null_mut();
