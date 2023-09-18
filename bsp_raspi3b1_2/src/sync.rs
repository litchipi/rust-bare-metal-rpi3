use core::cell::UnsafeCell;

pub struct RwLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for RwLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for RwLock<T> where T: ?Sized + Send {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn write<'a, R>(&'a self, f: impl FnOnce(&'a mut T) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        let data = unsafe { &mut *self.data.get() };

        crate::irq::exec_with_irq_masked(|| f(data))
    }

    pub fn read<'a, R>(&'a self, f: impl FnOnce(&'a T) -> R) -> R {
        let data = unsafe { &*self.data.get() };

        crate::irq::exec_with_irq_masked(|| f(data))
    }
}

pub struct InitLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for InitLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for InitLock<T> where T: ?Sized + Send {}

impl<T> InitLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn write<'a, R>(&'a self, f: impl FnOnce(&'a mut T) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        assert!(unsafe { !crate::INIT_DONE });
        let data = unsafe { &mut *self.data.get() };

        f(data)
    }

    pub fn read<'a, R>(&'a self, f: impl FnOnce(&'a T) -> R) -> R {
        let data = unsafe { &*self.data.get() };

        crate::irq::exec_with_irq_masked(|| f(data))
    }
}
