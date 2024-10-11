/// There are 58 General-Purpose Input/Output (GPIO) lines split into three banks. Bank 0 contains GPIOs 0 to 27, bank 1 contains GPIOs 28 to 45, and bank 2 contains GPIOs 46 to 57.
use crate::{mmio, mmio::PERIPHERAL_BASE};

const GPFSEL0: u32 = PERIPHERAL_BASE + 0x200000;
#[allow(dead_code)]
const GPSET0: u32 = PERIPHERAL_BASE + 0x20001C;
#[allow(dead_code)]
const GPCLR0: u32 = PERIPHERAL_BASE + 0x200028;
const GPPUPPDN0: u32 = PERIPHERAL_BASE + 0x2000E4;

/// Implemented by values are written to mmio addresses.
pub trait BinField {
    fn val(&self) -> u32;
    /// The size of a binary field.
    fn field_size(&self) -> u32;
}

/// Set the pull state of a GPIO pin to either `Pull::Float`, `Pull:High`, or `Pull:Low`.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Pull {
    Float = 0b00,
    High = 0b01,
    Low = 0b10,
}
impl BinField for Pull {
    fn val(&self) -> u32 {
        *self as u32
    }
    fn field_size(&self) -> u32 {
        2
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Select {
    Input = 0,
    Output = 1,
    AltFn0 = 0b100,
    AltFn1 = 0b101,
    AltFn2 = 0b110,
    AltFn3 = 0b111,
    AltFn4 = 0b011,
    AltFn5 = 0b010,
}
impl BinField for Select {
    fn val(&self) -> u32 {
        *self as u32
    }
    fn field_size(&self) -> u32 {
        3
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pin(pub u32);

impl Pin {
    pub fn select(self, func: Select) -> Self {
        call(self, func, GPFSEL0);
        self
    }
    pub fn pull(self, pull: Pull) -> Self {
        call(self, pull, GPPUPPDN0);
        self
    }
}

fn call<T: BinField>(pin: Pin, value: T, base_addr: u32) {
    let pin_number = pin.0;
    let field_size = value.field_size();
    let field_mask = (1 << field_size) - 1;

    let num_fields = 32 / field_size;
    let reg = base_addr + ((pin_number / num_fields) * 4);
    let shift = (pin_number % num_fields) * field_size;

    let mut curval = mmio::read(reg);
    curval &= !(field_mask << shift);
    curval |= value.val() << shift;
    mmio::write(reg, curval);
}
