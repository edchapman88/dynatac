use crate::interupts::IRQHandlerDescriptor;
use crate::interupts::{gicc, gicd};
use crate::locks::{init_state_rw::InitStateLock, ReadWriteEx};
use crate::utils::BoundedUsize;

type HandlerTable = [Option<IRQHandlerDescriptor<IRQNumber>>; IRQNumber::MAX_INCLUSIVE + 1];

pub type IRQNumber = BoundedUsize<{ GICv2::MAX_IRQ_NUMBER }>;

/// Representation of the GIC.
pub struct GICv2 {
    /// The Distributor.
    gicd: gicd::GICD,
    /// The CPU Interface.
    gicc: gicc::GICC,
    /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
    handler_table: InitStateLock<HandlerTable>,
}

impl GICv2 {
    const MAX_IRQ_NUMBER: usize = 300; // Normally 1019, but keep it lower to save some space.

    pub const COMPATIBLE: &'static str = "GICv2 (ARM Generic Interrupt Controller v2)";

    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(gicd_mmio_start_addr: usize, gicc_mmio_start_addr: usize) -> Self {
        Self {
            gicd: gicd::GICD::new(gicd_mmio_start_addr),
            gicc: gicc::GICC::new(gicc_mmio_start_addr),
            handler_table: InitStateLock::new([None; IRQNumber::MAX_INCLUSIVE + 1]),
        }
    }
}
