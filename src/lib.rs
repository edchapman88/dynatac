#![no_std]

mod gpio;
mod mmio;
mod uart;

use uart::write_blocking;

pub fn entry_point() {
    uart::init();
    write_blocking("Hello World!");
}
