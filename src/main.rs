#![no_std]
// 'binary' crates in Rust must include a function called 'main' to define the executable entry point unless the `no_main` attribute may be applied at the crate level.
#![no_main]

use dynatac::entry_point;

// #[panic_handler] is used to define the behavior of the Rust `panic!` macro (a panic is a fatal exception) in #![no_std] applications.
// https://doc.rust-lang.org/nomicon/panic-handler.html
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Define the reset handler. Exporting the function with the symbol "Reset_Handler" is not
// essential.
#[export_name = "Reset_Handler"]
pub fn reset() {
    entry_point();
}

// Define a pointer to the reset handler function which can be picked up by the linker and included
// at the correct position in the vector table.
#[link_section = ".vectors.reset_handler"]
#[no_mangle]
pub static __RESET_HANDLER: fn() = reset;
