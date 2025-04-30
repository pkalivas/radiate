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

    pub fn is_unique(&self) -> bool {
        unsafe { (*self.ref_count).load(Ordering::Acquire) == 1 }
    }

    pub fn is_shared(&self) -> bool {
        !self.is_unique()
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
                // Still need to decrement the ref count!
                let clone = (*self.value).clone();
                (*self.ref_count).fetch_sub(1, Ordering::Release);
                clone
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

impl<T: Clone> DerefMut for MutCell<T> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutcell_basic_clone_and_mutation() {
        let mut cell = MutCell::new(5);
        assert_eq!(*cell, 5);

        // Clone the cell
        let mut cell2 = cell.clone();
        assert_eq!(*cell2, 5);

        // Mutate original cell
        *cell.get_mut() = 10;
        assert_eq!(*cell, 10);
        // The clone still sees the original value because this is not copy-on-write
        assert_eq!(*cell2, 10);

        // Mutate via clone
        *cell2.get_mut() = 20;
        assert_eq!(*cell2, 20);
        assert_eq!(*cell, 20);
    }

    #[test]
    fn mutcell_into_inner_unique() {
        let cell = MutCell::new(String::from("hello"));
        let inner = cell.into_inner();
        assert_eq!(inner, "hello");
    }

    #[test]
    fn mutcell_into_inner_clone_when_multiple() {
        let cell = MutCell::new(String::from("hello"));
        let cell2 = cell.clone();

        let inner = cell.into_inner();
        assert_eq!(inner, "hello");

        // Drop cell2 to avoid leak
        drop(cell2);
    }

    #[test]
    fn mutcell_partial_eq_and_ord() {
        let cell1 = MutCell::new(10);
        let cell2 = MutCell::new(20);
        let cell3 = MutCell::new(10);

        assert!(cell1 == cell3);
        assert!(cell1 != cell2);
        assert!(cell1 < cell2);
        assert!(cell2 > cell3);
    }
}
