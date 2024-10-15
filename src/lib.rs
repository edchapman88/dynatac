#![no_std]

mod gpio;
mod mmio;
mod mutex;
/// pub so that the BLOCKING_WRITER can be used in the globally exported print macros
pub mod uart;

use uart::uart_io_update;

pub fn entry_point() {
    uart::init();
    println!("Initialised UART");

    loop {
        uart_io_update();
    }
}
