#![no_std]

use core::ptr::write_volatile;

pub fn entry_point() {
    unsafe {
        // Set the 'direction' of the required GPIO pins to be 'output'.
        // write_volatile(DIRSET_P0, 1 << ROW3);
    }
    loop {}
}
