use core::ptr::{read_volatile, write_volatile};

pub const PERIPHERAL_BASE: u32 = 0xFE000000;

pub fn read(addr: u32) -> u32 {
    unsafe { read_volatile(addr as *mut u32) }
}

pub fn write(addr: u32, val: u32) {
    unsafe {
        write_volatile(addr as *mut u32, val);
    }
}
