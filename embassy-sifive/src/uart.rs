use core::future::poll_fn;
use core::task::Poll;
use core::{marker::PhantomData, ops::Deref};
use e310x::interrupt::{ExternalInterrupt, Priority};
use e310x::{uart0, Uart0, PLIC};
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal_nb::serial::ErrorKind;

use crate::gpio::{gpio0, IOF0};
use crate::{clock::Clocks, time::Bps};

pub struct Serial<UART, TX, RX, MODE> {
    uart: UART,
    tx: Tx<UART, TX, MODE>,
    rx: Rx<UART, RX, MODE>,
}

pub struct Tx<UART, TX, MODE> {
    uart: UART,
    pin: TX,
    _mode: PhantomData<MODE>,
}

pub struct Rx<UART, RX, MODE> {
    uart: UART,
    pin: RX,
    _mode: PhantomData<MODE>,
}

static UART_WAKER: AtomicWaker = AtomicWaker::new();

/// TX pin
pub trait TxPin<UART>: private::Sealed {}
impl<T> TxPin<Uart0> for gpio0::Pin17<IOF0<T>> {}
/// RX pin
pub trait RxPin<UART>: private::Sealed {}
impl<T> RxPin<Uart0> for gpio0::Pin16<IOF0<T>> {}

/// UartX trait extends the UART peripheral
pub trait UartX: Deref<Target = uart0::RegisterBlock> + private::Sealed {
    /// Steals the UART peripheral
    ///
    /// # Safety
    ///
    /// Using this function may break the guarantees of the singleton pattern.
    unsafe fn steal() -> Self;
}

impl UartX for Uart0 {
    unsafe fn steal() -> Self {
        Uart0::steal()
    }
}

pub struct Blocking;
pub struct Async;

impl<UART: UartX, TX: TxPin<UART>, RX: RxPin<UART>> Serial<UART, TX, RX, Blocking> {
    pub fn new_blocking(uart: UART, pins: (TX, RX), baud_rate: Bps, clocks: Clocks) -> Self {
        let div = clocks.tlclk().0 / baud_rate.0 - 1;
        unsafe {
            uart.ie().write(|w| w.txwm().bit(false).rxwm().bit(false));
            uart.div().write(|w| w.bits(div));
            uart.txctrl()
                .write(|w| w.counter().bits(1).enable().bit(true));
            uart.rxctrl().write(|w| w.enable().bit(true));
        }

        let tx = Tx {
            uart: unsafe { UART::steal() },
            pin: pins.0,
            _mode: PhantomData,
        };
        let rx = Rx {
            uart: unsafe { UART::steal() },
            pin: pins.1,
            _mode: PhantomData,
        };

        Serial { uart, tx, rx }
    }

    pub fn read(&mut self) -> nb::Result<u8, ErrorKind> {
        self.rx.blocking_read()
    }

    pub fn write(&mut self, byte: u8) -> nb::Result<(), ErrorKind> {
        self.tx.blocking_write(byte)
    }

    pub fn flush(&mut self) -> nb::Result<(), ErrorKind> {
        self.tx.blocking_flush()
    }
}

impl<UART: UartX, TX: TxPin<UART>, RX: RxPin<UART>> Serial<UART, TX, RX, Async> {
    pub fn new_async(uart: UART, pins: (TX, RX), baud_rate: Bps, clocks: Clocks) -> Self {
        let div = clocks.tlclk().0 / baud_rate.0 - 1;
        unsafe {
            uart.div().write(|w| w.bits(div));
            uart.txctrl()
                .write(|w| w.counter().bits(1).enable().bit(true));
            uart.rxctrl().write(|w| w.enable().bit(true));
        }

        let tx = Tx {
            uart: unsafe { UART::steal() },
            pin: pins.0,
            _mode: PhantomData,
        };
        let rx = Rx {
            uart: unsafe { UART::steal() },
            pin: pins.1,
            _mode: PhantomData,
        };

        Serial { uart, tx, rx }
    }

    pub fn enable_interrupts(&self) {
        // Clear the interrupt
        self.uart.ip().write(|w| w.rxwm().clear_bit());

        // Enable receive interrupt
        self.uart
            .ie()
            .write(|w| w.txwm().bit(false).rxwm().bit(true));

        // Set UART interrupt priority
        let priorities = PLIC::priorities();
        priorities.reset::<ExternalInterrupt>();
        unsafe { priorities.set_priority(ExternalInterrupt::UART0, Priority::P1) };

        // Enable UART interrupt
        let ctx = PLIC::ctx0();
        unsafe {
            ctx.threshold().set_threshold(Priority::P0);
            ctx.enables().enable(ExternalInterrupt::UART0);
            riscv::interrupt::enable();
            PLIC::enable();
        }
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<(), ErrorKind> {
        self.rx.async_read(buf).await
    }

    pub async fn write(&mut self, byte: u8) -> nb::Result<(), ErrorKind> {
        self.tx.async_write(byte).await
    }

    pub async fn flush(&mut self) -> nb::Result<(), ErrorKind> {
        self.tx.async_flush().await
    }
}

impl<UART, TX, RX, MODE> Serial<UART, TX, RX, MODE> {
    pub fn split(self) -> (Tx<UART, TX, MODE>, Rx<UART, RX, MODE>) {
        (self.tx, self.rx)
    }

    pub fn free(self) -> (UART, (TX, RX)) {
        (self.uart, (self.tx.pin, self.rx.pin))
    }
}

impl<UART: UartX, PIN: RxPin<UART>> Rx<UART, PIN, Blocking> {
    pub fn blocking_read(&mut self) -> nb::Result<u8, ErrorKind> {
        let rxdata = self.uart.rxdata().read();

        if rxdata.empty().bit_is_set() {
            Err(::nb::Error::WouldBlock)
        } else {
            Ok(rxdata.data().bits())
        }
    }
}

impl<UART: UartX, PIN: TxPin<UART>> Tx<UART, PIN, Blocking> {
    pub fn blocking_write(&mut self, byte: u8) -> nb::Result<(), ErrorKind> {
        let txdata = self.uart.txdata().read();

        if txdata.full().bit_is_set() {
            Err(::nb::Error::WouldBlock)
        } else {
            self.uart.txdata().write(|w| unsafe { w.data().bits(byte) });
            Ok(())
        }
    }

    pub fn blocking_flush(&mut self) -> nb::Result<(), ErrorKind> {
        if self.uart.ip().read().txwm().bit_is_set() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<UART: UartX, PIN: RxPin<UART>> Rx<UART, PIN, Async> {
    pub async fn async_read(&mut self, buf: &mut [u8]) -> Result<(), ErrorKind> {
        let result = poll_fn(|cx| {
            UART_WAKER.register(cx.waker());

            if self.uart.ip().read().rxwm().bit_is_set() {
                let data = self.uart.rxdata().read().data().bits();
                buf[0] = data;
                // Enable receive interrupt
                self.uart
                    .ie()
                    .write(|w| w.txwm().bit(false).rxwm().bit(true));
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        })
        .await;

        result
    }
}

impl<UART: UartX, PIN: TxPin<UART>> Tx<UART, PIN, Async> {
    pub async fn async_write(&mut self, byte: u8) -> nb::Result<(), ErrorKind> {
        let txdata = self.uart.txdata().read();

        if txdata.full().bit_is_set() {
            Err(::nb::Error::WouldBlock)
        } else {
            self.uart.txdata().write(|w| unsafe { w.data().bits(byte) });
            Ok(())
        }
    }

    pub async fn async_flush(&mut self) -> nb::Result<(), ErrorKind> {
        if self.uart.ip().read().txwm().bit_is_set() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

#[riscv_rt::external_interrupt(ExternalInterrupt::UART0)]
fn interrupt_handler() {
    let uart = unsafe { Uart0::steal() };
    // Clear the interrupt
    uart.ip().write(|w| w.rxwm().clear_bit());
    // Disable receive interrupt
    uart.ie().write(|w| w.rxwm().bit(false));

    UART_WAKER.wake();
}

// seal the "private" traits
mod private {
    use crate::gpio::{gpio0, IOF0};
    use e310x::Uart0;

    pub trait Sealed {}

    impl Sealed for Uart0 {}
    impl<T> Sealed for gpio0::Pin17<IOF0<T>> {}
    impl<T> Sealed for gpio0::Pin16<IOF0<T>> {}
}
