#![no_std]

/// pub so that exception::handling_init can be run at kernelinit.
pub mod exception;
mod gpio;
pub mod interupts;
mod locks;
mod mmio;
pub mod state;
/// pub so that the BLOCKING_WRITER can be used in the globally exported print macros
pub mod uart;
mod utils;

use uart::uart_io_update;

pub fn entry_point() {
    uart::init();
    println!("Initialised UART");

    let (_, privilege_level) = exception::current_privilege_level();
    println!("Current privilege level: {}", privilege_level);

    loop {
        uart_io_update();
    }
}
