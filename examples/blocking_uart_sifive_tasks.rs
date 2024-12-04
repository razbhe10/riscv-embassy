#![no_std]
#![no_main]

use e310x::Uart0;
use embassy_executor::Spawner;
use embassy_sifive::gpio::gpio0::Pin16;
use embassy_sifive::gpio::{NoInvert, IOF0};
use embassy_sifive::time::U32Ext;
use embassy_sifive::uart::Blocking;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use utils::pin;
use utils::sprintln;

extern crate panic_halt;

const PERIOD1: u32 = 1000; // 1s
const PERIOD2: u32 = 2000; // 2s
const PERIOD3: u32 = 3000; // 3s

async fn shared_sleep(period: u32) {
    Timer::after(Duration::from_millis(period.into())).await;
}

async fn common_task(task_id: u32, period: u32) {
    sprintln!("Starting embassy executor for task {}", task_id);

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
    mut tled: embassy_sifive::gpio::gpio0::Pin5<
        embassy_sifive::gpio::Output<embassy_sifive::gpio::Regular<embassy_sifive::gpio::NoInvert>>,
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
    let dr = embassy_sifive::init();
    let p = dr.peripherals;
    let pins = dr.pins;

    let blue_led_pin = pin!(pins, led_blue);
    let tx_pin = pin!(pins, uart0_tx);
    let rx_pin = pin!(pins, uart0_rx);

    let clocks = embassy_sifive::clock::configure(p.PRCI, p.AONCLK, 320_u32.mhz().into());
    let rx = utils::stdout::configure(
        p.UART0,
        tx_pin,
        rx_pin,
        embassy_sifive::time::Bps(115_200),
        clocks,
    );

    spawner.spawn(my_task(1, PERIOD1)).unwrap();
    spawner.spawn(my_task(2, PERIOD2)).unwrap();
    spawner.spawn(my_task(3, PERIOD3)).unwrap();
    spawner.spawn(uart_task(rx)).unwrap();
    spawner
        .spawn(blink_loop(blue_led_pin.into_output()))
        .unwrap();
}

#[embassy_executor::task()]
async fn uart_task(mut rx: embassy_sifive::uart::Rx<Uart0, Pin16<IOF0<NoInvert>>, Blocking>) {
    let mut buffer = [0u8; 1];

    // Connect tx to stdout

    sprintln!("UART initialized");

    loop {
        match rx.blocking_read() {
            Ok(byte) => {
                // Store the received byte in the buffer
                buffer[0] = byte;

                // Convert the received byte to a string and print it
                if let Ok(received) = core::str::from_utf8(&buffer[..1]) {
                    sprintln!("Received: {}", received);
                } else {
                    sprintln!("Received non-UTF8 data");
                }
            }
            Err(_) => {
                // No data received
            }
        }

        // Optional: Add a small delay to avoid busy-waiting
        Timer::after(Duration::from_millis(20)).await;
    }
}
