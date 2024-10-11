#![no_std]

use core::{
    char,
    ptr::{read_volatile, write_volatile},
};

const GPIO_MAX_PIN: u32 = 53;
const GPIO_FUNCTION_ALT5: u32 = 2;

const PERIPHERAL_BASE: u32 = 0xFE000000;
const GPFSEL0: u32 = PERIPHERAL_BASE + 0x200000;
const GPSET0: u32 = PERIPHERAL_BASE + 0x20001C;
const GPCLR0: u32 = PERIPHERAL_BASE + 0x200028;
const GPPUPPDN0: u32 = PERIPHERAL_BASE + 0x2000E4;

const AUX_BASE: u32 = PERIPHERAL_BASE + 0x215000;
const AUX_ENABLES: u32 = AUX_BASE + 4;
const AUX_MU_IO_REG: u32 = AUX_BASE + 64;
const AUX_MU_IER_REG: u32 = AUX_BASE + 68;
const AUX_MU_IIR_REG: u32 = AUX_BASE + 72;
const AUX_MU_LCR_REG: u32 = AUX_BASE + 76;
const AUX_MU_MCR_REG: u32 = AUX_BASE + 80;
const AUX_MU_LSR_REG: u32 = AUX_BASE + 84;
const AUX_MU_CNTL_REG: u32 = AUX_BASE + 96;
const AUX_MU_BAUD_REG: u32 = AUX_BASE + 104;
const AUX_UART_CLOCK: u32 = 500000000;
const UART_MAX_QUEUE: u32 = 16 * 1024;

fn aux_mu_baud(baud: u32) -> u32 {
    (AUX_UART_CLOCK / (baud * 8)) - 1
}

pub enum Pull {
    Float = 0,
    Low,
    High,
}

fn gpio_call(pin_number: u32, value: u32, base: u32, field_size: u32, field_max: u32) -> u32 {
    let field_mask = (1 << field_size) - 1;

    if pin_number > field_max {
        return 0;
    };
    if value > field_mask {
        return 0;
    };

    let num_fields = 32 / field_size;
    let reg = base + ((pin_number / num_fields) * 4);
    let shift = (pin_number % num_fields) * field_size;

    let mut curval = mmio_read(reg);
    curval &= !(field_mask << shift);
    curval |= value << shift;
    mmio_write(reg, curval);

    return 1;
}

fn gpio_pull(pin_number: u32, pull: Pull) {
    gpio_call(pin_number, pull as u32, GPPUPPDN0, 2, GPIO_MAX_PIN);
}

fn gpio_function(pin_number: u32, value: u32) {
    gpio_call(pin_number, value, GPFSEL0, 3, GPIO_MAX_PIN);
}

fn gpio_use_as_alt5(pin_number: u32) {
    gpio_pull(pin_number, Pull::Float);
    gpio_function(pin_number, GPIO_FUNCTION_ALT5);
}

pub fn entry_point() {
    uart_init();
    uart_writeByteBlockingActual('c');
    uart_writeByteBlockingActual('c');
    uart_writeByteBlockingActual('c');
    uart_writeByteBlockingActual('c');
    uart_writeByteBlockingActual('c');
    loop {}
}
fn mmio_read(addr: u32) -> u32 {
    unsafe { read_volatile(addr as *mut u32) }
}

fn mmio_write(addr: u32, val: u32) {
    unsafe {
        write_volatile(addr as *mut u32, val);
    }
}

fn uart_init() {
    mmio_write(AUX_ENABLES, 1); //enable UART1
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_CNTL_REG, 0);
    mmio_write(AUX_MU_LCR_REG, 3); //8 bits
    mmio_write(AUX_MU_MCR_REG, 0);
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_IIR_REG, 0xC6); //disable interrupts
    mmio_write(AUX_MU_BAUD_REG, aux_mu_baud(115200));
    gpio_use_as_alt5(14);
    gpio_use_as_alt5(15);
    mmio_write(AUX_MU_CNTL_REG, 3); //enable RX/TX
}
fn uart_isWriteByteReady() -> bool {
    (mmio_read(AUX_MU_LSR_REG) & 0x20) != 0
}

fn uart_writeByteBlockingActual(ch: char) {
    while !uart_isWriteByteReady() {}
    mmio_write(AUX_MU_IO_REG, ch.into());
}
