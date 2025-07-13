use crate::{interupts, state};
use core::cell::UnsafeCell;

/// In contrast to a real Mutex implementation, does not protect against concurrent access from other cores to the contained data. The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is executing on a single core.
pub struct IRQSafeMutex<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for IRQSafeMutex<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for IRQSafeMutex<T> where T: ?Sized + Send {}

impl<T> IRQSafeMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
    pub fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut T) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this mutable reference will ever only be given out once at a time. For now assert that the system is only running with a single core.
        assert!(
            state::state_manager().is_single_core(),
            "IQRSafeMutex::lock is only safe during single core operation and the kernel state was not single core"
        );

        let data = unsafe { &mut *self.data.get() };

        f(data)
    }
}
impl<T> super::Mutex for IRQSafeMutex<T> {
    type Data = T;

    fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        let data = unsafe { &mut *self.data.get() };

        // Execute the closure while IRQs are masked.
        interupts::exec_with_irq_masked(|| f(data))
    }
}
