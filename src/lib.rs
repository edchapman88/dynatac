#![no_std]

mod gpio;
mod mmio;
mod uart;

use uart::uart_io_update;

pub fn entry_point() {
    uart::init();
    //uart::write_byte_blocking("a".as_bytes()[0]);
    uart::write_blocking("hello");
    uart::write("world");
    loop {
        uart_io_update();
    }
}
