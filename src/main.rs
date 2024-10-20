#![no_std]
// 'binary' crates in Rust must include a function called 'main' to define the executable entry point unless the `no_main` attribute may be applied at the crate level.
#![no_main]

use aarch64_cpu::asm;
use dynatac::entry_point;
use dynatac::exception;
use dynatac::println;

mod boot;

/// Pause execution on the core.
#[inline(always)]
pub fn halt_cpu() -> ! {
    loop {
        asm::wfe()
    }
}

fn panic_prevent_reenter() {
    use core::sync::atomic::{AtomicBool, Ordering};
    static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
    if !PANIC_IN_PROGRESS.load(Ordering::Relaxed) {
        PANIC_IN_PROGRESS.store(true, Ordering::Relaxed);
        return;
    }
    halt_cpu();
}

// #[panic_handler] is used to define the behavior of the Rust `panic!` macro (a panic is a fatal exception) in #![no_std] applications.
// https://doc.rust-lang.org/nomicon/panic-handler.html
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    panic_prevent_reenter();
    println!("{}", info);
    halt_cpu();
}

unsafe fn kernel_init() -> ! {
    exception::handling_init();
    entry_point();
    halt_cpu();
}
