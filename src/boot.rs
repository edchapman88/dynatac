// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

use core::arch::global_asm;

// Assembly files.
global_asm!(include_str!("../asm/boot.s"));
global_asm!(include_str!("../asm/exception.s"));

/// Used by `arch` code to find the early boot core.
#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

/// The function is called from the assembly `_start` function.
#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_init()
}
