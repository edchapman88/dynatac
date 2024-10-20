#![no_std]

/// pub so that exception::handling_init can be run at kernelinit.
pub mod exception;
mod gpio;
mod mmio;
mod mutex;
/// pub so that the BLOCKING_WRITER can be used in the globally exported print macros
pub mod uart;

use uart::uart_io_update;

pub fn entry_point() {
    uart::init();
    println!("Initialised UART");

    let (_, privilege_level) = exception::current_privilege_level();
    println!("Current privilege level: {}", privilege_level);

    // Cause an exception by accessing a virtual address for which no translation was set up. This
    // code accesses the address 8 GiB, which is outside the mapped address space.
    //
    // For demo purposes, the exception handler will catch the faulting 8 GiB address and allow
    // execution to continue.
    println!("Trying to read from address 8 GiB...");
    let mut big_addr: u64 = 8 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    println!("Whoa! We recovered from a synchronous exception!");

    // Now use address 9 GiB. The exception handler won't forgive us this time.
    println!("Trying to read from address 9 GiB...");
    big_addr = 9 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    loop {
        uart_io_update();
    }
}
