#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hifive1::hal::device::DevicePeripherals;
use hifive1::hal::{prelude::*, DeviceResources};
use hifive1::{pin, pins, sprintln};

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

#[embassy_executor::task()]
async fn blink_loop(
    mut tled: hifive1::hal::gpio::gpio0::Pin5<
        hifive1::hal::gpio::Output<hifive1::hal::gpio::Regular<hifive1::hal::gpio::NoInvert>>,
    >,
) {
    sprintln!("Starting blink loop embassy");

    let mut status = false;
    loop {
        // toggle led
        if status {
            tled.set_low().unwrap();
        } else {
            tled.set_high().unwrap();
        }

        status = !status;
        shared_sleep(PERIOD1).await;
    }
}

#[embassy_executor::task(pool_size = 3)]
async fn my_task(num: u32, period: u32) {
    common_task(num, period).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;
    let pins = dr.pins;

    let blue_led_pin = pins!(pins, (led_blue));
    let tx_pin = pin!(pins, uart0_tx);
    let rx_pin = pin!(pins, uart0_rx);

    enable_uart(p, tx_pin.into_output(), rx_pin.into_output());

    spawner.spawn(my_task(1, PERIOD1)).unwrap();
    spawner.spawn(my_task(2, PERIOD2)).unwrap();
    spawner.spawn(my_task(3, PERIOD3)).unwrap();
    spawner
        .spawn(blink_loop(blue_led_pin.into_output()))
        .unwrap();
}

fn enable_uart(
    p: DevicePeripherals,
    tx: hifive1::hal::gpio::gpio0::Pin17<
        hifive1::hal::gpio::Output<hifive1::hal::gpio::Regular<hifive1::hal::gpio::NoInvert>>,
    >,
    rx: hifive1::hal::gpio::gpio0::Pin16<
        hifive1::hal::gpio::Output<hifive1::hal::gpio::Regular<hifive1::hal::gpio::NoInvert>>,
    >,
) {
    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());

    // Configure UART for stdout
    hifive1::stdout::configure(p.UART0, tx, rx, 115_200.bps(), clocks);
    sprintln!("UART initialization done");
}
