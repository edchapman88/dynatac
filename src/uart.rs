use core::fmt;
use core::fmt::Write;

use crate::{
    gpio::{self, Pull, Select},
    mmio,
    mutex::NullLock,
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
const UART_MAX_QUEUE: u32 = 100;

pub static BLOCKING_WRITER: NullLock<BlockingWriter> = NullLock::new(BlockingWriter);

pub static FIFO_WRITER: NullLock<FIFOWriter> = NullLock::new(FIFOWriter {
    write_cur: 0,
    read_cur: 0,
    buffer: [0; UART_MAX_QUEUE as usize],
});

#[derive(Debug)]
pub struct BlockingWriter;
impl BlockingWriter {
    fn write_byte(bt: u8) {
        let bt = match bt {
            b'\n' => {
                Self::write_byte(b'\r');
                b'\n'
            }
            bt => bt,
        };
        while !tx_empty() {}
        mmio::write(AUX_MU_IO_REG, bt.into());
    }

    fn write(msg: &str) {
        msg.as_bytes().iter().for_each(|bt| {
            Self::write_byte(*bt);
        })
    }
}
impl fmt::Write for BlockingWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::write(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    BlockingWriter.write_fmt(args).unwrap();
}

#[derive(Debug)]
pub struct FIFOWriter {
    write_cur: usize,
    read_cur: usize,
    buffer: [u8; UART_MAX_QUEUE as usize],
}
impl FIFOWriter {
    fn enqueue(&mut self, bt: u8) {
        (*self).buffer[self.write_cur] = bt;
        (*self).write_cur += 1;
    }

    fn send(&mut self) {
        if !self.queue_empty() && tx_empty() {
            BlockingWriter::write_byte(self.buffer[self.read_cur]);
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

pub fn uart_io_update() {
    FIFO_WRITER.lock(|writer| writer.send());

    if !rx_empty() {
        let bt = match read_byte_blocking() {
            b'\r' => b'\n',
            bt => bt,
        };
        FIFO_WRITER.lock(|writer| writer.enqueue(bt));
    }
}
