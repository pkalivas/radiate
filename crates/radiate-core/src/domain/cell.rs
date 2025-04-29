use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct MutCell<T> {
    value: *mut T,
    ref_count: *const AtomicUsize,
    consumed: bool,
}

impl<T> MutCell<T> {
    pub fn new(value: T) -> Self {
        let value = Box::into_raw(Box::new(value));
        let ref_count = Box::into_raw(Box::new(AtomicUsize::new(1)));
        MutCell {
            value,
            ref_count,
            consumed: false,
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.value }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value }
    }

    pub fn into_inner(mut self) -> T
    where
        T: Clone,
    {
        unsafe {
            if (*self.ref_count).load(Ordering::Acquire) == 1 {
                std::sync::atomic::fence(Ordering::SeqCst);
                let value = Box::from_raw(self.value);
                drop(Box::from_raw(self.ref_count as *mut AtomicUsize));
                self.consumed = true;
                *value
            } else {
                // Multiple owners exist, clone
                (*self.value).clone()
            }
        }
    }
}

impl<T> Deref for MutCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for MutCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T> PartialEq for MutCell<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: PartialOrd> PartialOrd for MutCell<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(other.get())
    }
}

impl<T> From<T> for MutCell<T> {
    fn from(value: T) -> Self {
        MutCell::new(value)
    }
}

impl<T> Clone for MutCell<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.ref_count).fetch_add(1, Ordering::Relaxed);
        }
        MutCell {
            value: self.value,
            ref_count: self.ref_count,
            consumed: false,
        }
    }
}

impl<T> Drop for MutCell<T> {
    fn drop(&mut self) {
        if self.consumed {
            return;
        }
        unsafe {
            if (*self.ref_count).fetch_sub(1, Ordering::Release) == 1 {
                std::sync::atomic::fence(Ordering::Acquire);
                drop(Box::from_raw(self.value));
                drop(Box::from_raw(self.ref_count as *mut AtomicUsize));
            }
        }
    }
}
