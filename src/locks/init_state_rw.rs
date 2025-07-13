use crate::{interupts, state};
use core::cell::UnsafeCell;

/// A pseudo-lock that is RW during the single-core kernel init phase and RO afterwards. Intended to encapsulate data that is populated during kernel init when no concurrency exists.
pub struct InitStateLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for InitStateLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for InitStateLock<T> where T: ?Sized + Send {}

impl<T> InitStateLock<T> {
    /// Create an instance.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}
impl<T> super::ReadWriteEx for InitStateLock<T> {
    type Data = T;

    fn write<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        assert!(
            state::state_manager().is_init(),
            "InitStateLock::write called after kernel init phase"
        );
        assert!(
            !interupts::enable::is_local_irq_masked(),
            "InitStateLock::write called with IRQs unmasked"
        );

        let data = unsafe { &mut *self.data.get() };

        f(data)
    }

    fn read<'a, R>(&'a self, f: impl FnOnce(&'a Self::Data) -> R) -> R {
        let data = unsafe { &*self.data.get() };

        f(data)
    }
}
