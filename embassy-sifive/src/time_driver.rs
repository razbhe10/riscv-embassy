//! This module provides a timer driver implementation for the E310x platform.
//! It includes functionality for managing alarms and handling timer interrupts.

use core::{cell::Cell, ptr, sync::atomic::Ordering};
use critical_section::CriticalSection;
use e310x::CLINT;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;

use embassy_time_driver::{AlarmHandle, Driver};
use portable_atomic::AtomicU8;
use riscv::interrupt::Interrupt;

const FREQUENCY_HZ: u64 = 32768;
const ALARM_COUNT: usize = 1;

/// Represents the state of an alarm.
struct AlarmState {
    /// The timestamp at which the alarm is set to fire.
    timestamp: Cell<u64>,
    /// The callback function to be called when the alarm fires.
    callback: Cell<*const ()>,
    /// The context to be passed to the callback function.
    ctx: Cell<*mut ()>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    /// Creates a new `AlarmState` with default values.
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(ptr::null()),
            ctx: Cell::new(ptr::null_mut()),
        }
    }
}

/// Represents the timer driver.
struct TimerDriver {
    /// The count of alarms allocated.
    alarm_count: AtomicU8,
    /// The array of alarm states.
    /// Each alarm state includes a timestamp, callback, and context.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver {
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [AlarmState::new(); ALARM_COUNT]),
});

impl TimerDriver {
    /// Retrieves the alarm state for the given alarm handle.
    ///
    /// # Safety
    ///
    /// This function assumes the `AlarmState` is created by this driver and
    /// never creates one that's out of bounds.
    fn get_alarm<'a>(&'a self, cs: CriticalSection<'a>, alarm: AlarmHandle) -> &'a AlarmState {
        unsafe { self.alarms.borrow(cs).get_unchecked(alarm.id() as usize) }
    }
}

impl Driver for TimerDriver {
    /// Returns the current time in ticks.
    fn now(&self) -> u64 {
        CLINT::mtime().read()
    }

    /// Allocates a new alarm handle if available.
    ///
    /// # Safety
    ///
    /// This function must be called within a critical section to ensure
    /// atomicity of the alarm count increment.
    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        critical_section::with(|_| {
            let id = self.alarm_count.load(Ordering::Relaxed);
            if id < ALARM_COUNT as u8 {
                self.alarm_count.store(id + 1, Ordering::Relaxed);
                Some(AlarmHandle::new(id))
            } else {
                None
            }
        })
    }

    /// Sets the callback function and context for the given alarm handle.
    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);

            alarm.callback.set(callback as *const ());
            alarm.ctx.set(ctx);
        })
    }

    /// Schedules the wake-up event for the given alarm handle at the specified timestamp.
    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);

            let t = self.now();
            if timestamp <= t {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                CLINT::mtimer_disable();

                alarm.timestamp.set(u64::MAX);

                return false;
            }

            // Set the compare value for the alarm.
            CLINT::mtimecmp0().write(timestamp);
            unsafe { CLINT::mtimer_enable() };
            true
        })
    }
}

/// Custom interrupt handler for the machine timer interrupt.
#[riscv_rt::core_interrupt(Interrupt::MachineTimer)]
fn custom_machine_timer_handler() {
    // Clear the timer interrupt.
    CLINT::mtimecmp0().modify(|f| *f += FREQUENCY_HZ);
}
