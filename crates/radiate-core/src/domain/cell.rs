use std::{
    cell::UnsafeCell,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct MutCell<T> {
    inner: *const ArcInner<T>,
    consumed: bool,
}

struct ArcInner<T> {
    value: UnsafeCell<T>,
    ref_count: AtomicUsize,
}

// Ensure MutCell<T> is safe to send/sync if T is
unsafe impl<T: Send> Send for MutCell<T> {}
unsafe impl<T: Sync> Sync for MutCell<T> {}

impl<T> MutCell<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::into_raw(Box::new(ArcInner {
            value: UnsafeCell::new(value),
            ref_count: AtomicUsize::new(1),
        }));
        Self {
            inner,
            consumed: false,
        }
    }

    pub fn is_unique(&self) -> bool {
        unsafe { (*self.inner).ref_count.load(Ordering::Acquire) == 1 }
    }

    pub fn is_shared(&self) -> bool {
        !self.is_unique()
    }

    pub fn get(&self) -> &T {
        unsafe { &*(*self.inner).value.get() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        assert!(self.is_unique(), "Cannot mutably borrow shared MutCell");
        unsafe { &mut *(*self.inner).value.get() }
    }

    pub fn into_inner(mut self) -> T
    where
        T: Clone,
    {
        unsafe {
            if (*self.inner).ref_count.load(Ordering::Acquire) == 1 {
                self.consumed = true;
                std::sync::atomic::fence(Ordering::SeqCst);
                let boxed = Box::from_raw(self.inner as *mut ArcInner<T>);
                boxed.value.into_inner()
            } else {
                let clone = (*(*self.inner).value.get()).clone();
                (*self.inner).ref_count.fetch_sub(1, Ordering::Release);
                clone
            }
        }
    }
}

impl<T> Clone for MutCell<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner).ref_count.fetch_add(1, Ordering::Relaxed);
        }
        Self {
            inner: self.inner,
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
            if (*self.inner).ref_count.fetch_sub(1, Ordering::Release) == 1 {
                std::sync::atomic::fence(Ordering::Acquire);
                drop(Box::from_raw(self.inner as *mut ArcInner<T>));
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

impl<T: PartialEq> PartialEq for MutCell<T> {
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
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutcell_basic_clone_and_mutation_updated() {
        let mut cell = MutCell::new(5);
        assert_eq!(*cell, 5);

        // Mutate the cell (unique, so this is allowed)
        *cell.get_mut() = 10;
        assert_eq!(*cell, 10);

        // Cloning happens only after mutation:
        let cell2 = cell.clone();
        // Now, getting a mutable reference from either cell will panic because it's no longer unique.
        // Instead, test that both views see the update:
        assert_eq!(*cell2, 10);
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
