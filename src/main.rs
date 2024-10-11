#![no_std]
// 'binary' crates in Rust must include a function called 'main' to define the executable entry point unless the `no_main` attribute may be applied at the crate level.
#![no_main]

use core::arch::global_asm;
use dynatac::entry_point;

// #[panic_handler] is used to define the behavior of the Rust `panic!` macro (a panic is a fatal exception) in #![no_std] applications.
// https://doc.rust-lang.org/nomicon/panic-handler.html
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

global_asm!(
    "
    .section \".text.boot\"

    // Check processor ID is zero (executing on main core), else hang
    mrs x1, mpidr_el1
    and x1, x1, #3
    cbz x1, 2f

    // We're not on the main core, so hang in an infinite wait loop
    1:  wfe
        b 1b
    2:  // We're on the main core!

        // Set the stack pointer to __stack_top symbol defined in the linker script.
        ldr x1, = __stack_top
        mov sp, x1

        bl main
        // In case it does return, halt the master core too
        b 1b
"
);

#[no_mangle]
extern "C" fn main() -> ! {
    entry_point();
    loop {}
}
