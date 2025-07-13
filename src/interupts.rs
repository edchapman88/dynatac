// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
pub mod enable;
pub mod gic;
mod gicc;
mod gicd;
mod null_irq_manager;

use crate::locks::{init_state_rw::InitStateLock, ReadWriteEx};
use core::marker::PhantomData;
use gic::IRQNumber;

static CUR_IRQ_MANAGER: InitStateLock<&'static (dyn IRQManager<IRQNumberType = IRQNumber> + Sync)> =
    InitStateLock::new(&null_irq_manager::NULL_IRQ_MANAGER);

/// Implemented by types that handle IRQs.
pub trait IRQHandler {
    /// Called when the corresponding interrupt is asserted.
    fn handle(&self) -> Result<(), &'static str>;
}

pub trait IRQManager {
    /// The IRQ number type depends on the implementation.
    type IRQNumberType: Copy;

    /// Register a handler.
    fn register_handler(
        &self,
        irq_handler_descriptor: IRQHandlerDescriptor<Self::IRQNumberType>,
    ) -> Result<(), &'static str>;

    /// Enable an interrupt in the controller.
    fn enable(&self, irq_number: &Self::IRQNumberType);

    /// This function is called directly from the CPU's IRQ exception vector. On AArch64, this means that the respective CPU core has disabled exception handling. This function can therefore not be preempted and runs start to finish. Takes an IRQContext token to ensure it can only be called from IRQ context.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn handle_pending_irqs<'irq_context>(&'irq_context self, ic: &IRQContext<'irq_context>);

    /// Print list of registered handlers.
    fn print_handler(&self) {}
}

/// Interrupt descriptor.
#[derive(Copy, Clone)]
pub struct IRQHandlerDescriptor<T>
where
    T: Copy,
{
    number: T,
    name: &'static str,
    handler: &'static (dyn IRQHandler + Sync),
}

impl<T> IRQHandlerDescriptor<T>
where
    T: Copy,
{
    pub const fn new(
        number: T,
        name: &'static str,
        handler: &'static (dyn IRQHandler + Sync),
    ) -> Self {
        Self {
            number,
            name,
            handler,
        }
    }
    pub const fn number(&self) -> T {
        self.number
    }
    pub const fn name(&self) -> &'static str {
        self.name
    }
    pub const fn handler(&self) -> &'static (dyn IRQHandler + Sync) {
        self.handler
    }
}

/// An instance of this type indicates that the local core is currently executing in IRQ context, aka executing an interrupt vector or subcalls of it.
/// Concept and implementation derived from the `CriticalSection` introduced in
/// <https://github.com/rust-embedded/bare-metal>
#[derive(Clone, Copy)]
pub struct IRQContext<'irq_context> {
    _0: PhantomData<&'irq_context ()>,
}

impl<'irq_context> IRQContext<'irq_context> {
    /// Creates an IRQContext token.
    ///
    /// # Safety
    ///
    /// - This must only be called when the current core is in an interrupt context and will not
    ///   live beyond the end of it. That is, creation is allowed in interrupt vector functions. For
    ///   example, in the ARMv8-A case, in `extern "C" fn current_elx_irq()`.
    /// - Note that the lifetime `'irq_context` of the returned instance is unconstrained. User code
    ///   must not be able to influence the lifetime picked for this type, since that might cause it
    ///   to be inferred to `'static`.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        IRQContext { _0: PhantomData }
    }
}

/// Executes the provided closure while IRQs are masked on the executing core.
///
/// While the function temporarily changes the HW state of the executing core, it restores it to the
/// previous state before returning, so this is deemed safe.
#[inline(always)]
pub fn exec_with_irq_masked<T>(f: impl FnOnce() -> T) -> T {
    let saved = enable::local_irq_mask_save();
    let ret = f();
    enable::local_irq_restore(saved);

    ret
}

/// Register a new IRQ manager.
pub fn register_irq_manager(
    new_manager: &'static (dyn IRQManager<IRQNumberType = IRQNumber> + Sync),
) {
    CUR_IRQ_MANAGER.write(|manager| *manager = new_manager);
}

/// Return a reference to the currently registered IRQ manager.
///
/// This is the IRQ manager used by the architectural interrupt handling code.
pub fn irq_manager() -> &'static dyn IRQManager<IRQNumberType = IRQNumber> {
    CUR_IRQ_MANAGER.read(|manager| *manager)
}
