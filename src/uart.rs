use crate::{
    gpio,
    gpio::{Pull, Select},
    mmio,
};

const AUX_BASE: u32 = mmio::PERIPHERAL_BASE + 0x215000;
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
#[allow(dead_code)]
const UART_MAX_QUEUE: u32 = 16 * 1024;

fn aux_mu_baud(baud: u32) -> u32 {
    (AUX_UART_CLOCK / (baud * 8)) - 1
}

pub fn init() {
    mmio::write(AUX_ENABLES, 1); //enable UART1
    mmio::write(AUX_MU_IER_REG, 0);
    mmio::write(AUX_MU_CNTL_REG, 0);
    mmio::write(AUX_MU_LCR_REG, 3); //8 bits
    mmio::write(AUX_MU_MCR_REG, 0);
    mmio::write(AUX_MU_IER_REG, 0);
    mmio::write(AUX_MU_IIR_REG, 0xC6); //disable interrupts
    mmio::write(AUX_MU_BAUD_REG, aux_mu_baud(115200));
    gpio::Pin(14).pull(Pull::Float).select(Select::AltFn5);
    gpio::Pin(15).pull(Pull::Float).select(Select::AltFn5);
    mmio::write(AUX_MU_CNTL_REG, 3); //enable RX/TX
}
fn write_byte_ready() -> bool {
    (mmio::read(AUX_MU_LSR_REG) & 0x20) != 0
}

fn write_byte_blocking(ch: char) {
    while !write_byte_ready() {}
    mmio::write(AUX_MU_IO_REG, ch.into());
}

pub fn write_blocking(msg: &str) {
    msg.chars().for_each(|ch| match ch {
        // replace newline with carriage return
        '\n' => write_byte_blocking('\r'),
        ch => write_byte_blocking(ch),
    });
}
