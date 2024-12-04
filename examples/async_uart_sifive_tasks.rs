#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use e310x::Uart0;
use embassy_executor::Spawner;
use embassy_sifive::gpio::gpio0::Pin16;
use embassy_sifive::gpio::{NoInvert, IOF0};
use embassy_sifive::time::U32Ext;
use embassy_sifive::uart::Async;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use utils::pin;

extern crate panic_halt;

const PERIOD1: u32 = 1000; // 1s

async fn shared_sleep(period: u32) {
    Timer::after(Duration::from_millis(period.into())).await;
}

#[embassy_executor::task()]
async fn blink_loop(
    mut tled: embassy_sifive::gpio::gpio0::Pin5<
        embassy_sifive::gpio::Output<embassy_sifive::gpio::Regular<embassy_sifive::gpio::NoInvert>>,
    >,
) {
    info!("Starting blink loop embassy");

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let dr = embassy_sifive::init();
    let p = dr.peripherals;
    let pins = dr.pins;

    let blue_led_pin = pin!(pins, led_blue);
    let tx_pin = pin!(pins, uart0_tx);
    let rx_pin = pin!(pins, uart0_rx);

    let clocks = embassy_sifive::clock::configure(p.PRCI, p.AONCLK, 320_u32.mhz().into());
    let serial = embassy_sifive::uart::Serial::new_async(
        p.UART0,
        (tx_pin.into_iof0(), rx_pin.into_iof0()),
        embassy_sifive::time::Bps(115_200),
        clocks,
    );

    spawner.spawn(uart_task(serial)).unwrap();
    spawner
        .spawn(blink_loop(blue_led_pin.into_output()))
        .unwrap();
}

#[embassy_executor::task()]
async fn uart_task(
    mut serial: embassy_sifive::uart::Serial<
        Uart0,
        embassy_sifive::gpio::gpio0::Pin17<IOF0<NoInvert>>,
        Pin16<IOF0<NoInvert>>,
        Async,
    >,
) {
    let mut buffer = [0u8; 1];
    serial.enable_interrupts();

    loop {
        match serial.read(&mut buffer).await {
            Ok(_) => {
                // Store the received byte in the buffer
                // Convert the received byte to a string and print it
                if let Ok(received) = core::str::from_utf8(&buffer[..1]) {
                    info!("Received: {}", received);
                    serial.write(buffer[0]).await.unwrap();
                } else {
                    info!("Received non-UTF8 character");
                }
            }
            Err(_) => {
                // No data received
            }
        }
    }
}
