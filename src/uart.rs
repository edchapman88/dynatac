use lazy_static::lazy_static;
use spin::Mutex;

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
const UART_MAX_QUEUE: u32 = 100;

#[derive(Debug)]
pub struct Writer {
    write_cur: usize,
    read_cur: usize,
    buffer: [u8; UART_MAX_QUEUE as usize],
}
impl Writer {
    fn enqueue(&mut self, bt: u8) {
        (*self).buffer[self.write_cur] = bt;
        (*self).write_cur += 1;
    }

    fn send(&mut self) {
        if !self.queue_empty() && tx_empty() {
            write_byte_blocking(self.buffer[self.read_cur]);
            (*self).read_cur += 1;
        }
    }

    fn queue_empty(&mut self) -> bool {
        if self.write_cur == self.read_cur {
            // Reset cursors and start writing over the previous buffer values.
            (*self).write_cur = 0;
            (*self).read_cur = 0;
            return true;
        }
        false
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        write_cur: 0,
        read_cur: 0,
        buffer: [0; UART_MAX_QUEUE as usize],
    });
}

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

fn rx_empty() -> bool {
    (mmio::read(AUX_MU_LSR_REG) & 0x01) == 0
}

fn tx_empty() -> bool {
    (mmio::read(AUX_MU_LSR_REG) & 0x20) != 0
}

fn read_byte_blocking() -> u8 {
    while rx_empty() {}
    mmio::read(AUX_MU_IO_REG) as u8
}

pub fn write_byte_blocking(bt: u8) {
    while !tx_empty() {}
    mmio::write(AUX_MU_IO_REG, bt.into());
}

fn write_byte(bt: u8) {
    WRITER.lock().enqueue(bt);
}

pub fn write_blocking(msg: &str) {
    msg.as_bytes().iter().for_each(|bt| {
        write_byte_blocking(*bt);
    })
}

pub fn write(msg: &str) {
    msg.as_bytes()
        .iter()
        .enumerate()
        .for_each(|(i, bt)| match bt {
            // replace newline with carriage return
            //'\n' => write_byte('\r'),
            bt => {
                write_byte_blocking(i as u8);
                write_byte(*bt);
            }
        });
}

pub fn uart_io_update() {
    WRITER.lock().send();

    if !rx_empty() {
        match read_byte_blocking() {
            //'\r' => write_byte('\n'),
            ch => write_byte(ch),
        }
    }
}
