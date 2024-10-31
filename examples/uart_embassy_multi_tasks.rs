#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hifive1::hal::{prelude::*, DeviceResources};
use hifive1::{pin, sprintln};

use sifive as _;

extern crate panic_halt;
use embassy_time::{Duration, Timer};

const PERIOD1: u32 = 1000; // 1s
const PERIOD2: u32 = 2000; // 2s
const PERIOD3: u32 = 3000; // 2s

async fn shared_sleep(period: u32) {
    // sleep.delay_ms(period);
    Timer::after(Duration::from_millis(period.into())).await;
}

async fn common_task(task_id: u32, period: u32) {
    sprintln!("Starting embassy executor for task {}", task_id);

    // let clint = dr.core_peripherals.clint;
    // let mut sleep = Sleep::new(clint.mtimecmp, clocks);

    let mut status = false;
    loop {
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            sprintln!("Counter{}: {}", task_id, COUNTER);
        }

        status = !status;
        shared_sleep(period).await;
    }
}

#[embassy_executor::task(pool_size = 3)]
async fn my_task(num: u32, period: u32) {
    common_task(num, period).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    enable_uart();

    spawner.spawn(my_task(1, PERIOD1)).unwrap();
    spawner.spawn(my_task(2, PERIOD2)).unwrap();
    spawner.spawn(my_task(3, PERIOD3)).unwrap();
}

fn enable_uart() {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );
    sprintln!("UART initialization done");
}
