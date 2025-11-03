use std::{
    cell::UnsafeCell,
    fmt::{Debug, Formatter},
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
        Self {
            inner: Box::into_raw(Box::new(ArcInner {
                value: UnsafeCell::new(value),
                ref_count: AtomicUsize::new(1),
            })),
            consumed: false,
        }
    }

    pub fn is_unique(&self) -> bool {
        // SAFETY: We're only reading the ref_count
        unsafe { (*self.inner).ref_count.load(Ordering::Acquire) == 1 }
    }

    pub fn is_shared(&self) -> bool {
        !self.is_unique()
    }

    pub fn strong_count(&self) -> usize {
        // SAFETY: We're only reading the ref_count
        unsafe { (*self.inner).ref_count.load(Ordering::Acquire) }
    }

    pub fn get(&self) -> &T {
        // SAFETY: This is inherently unsafe because we don't know if there exists a mutable
        // reference to the inner value elsewhere.
        //
        // We assume that the caller has ensured that there are no mutable references
        // to the inner value when calling this method. So straight up - make sure that you don't have
        // any mutable references to the inner value when calling this method.
        assert!(!self.consumed, "Cannot access consumed MutCell");
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
        // SAFETY: If there is more than one reference to the
        // inner value, we will clone it and decrement the ref count.
        // If there is only one reference, we will consume the inner value and
        // drop the inner box.
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

impl<T: Debug> Debug for MutCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get())
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

    #[test]
    fn mutcell_is_unique_and_shared() {
        let cell = MutCell::new(42);
        assert!(cell.is_unique());

        let cell2 = cell.clone();

        assert!(cell.is_shared());
        assert!(cell2.is_shared());
        assert!(!cell.is_unique());
        assert!(!cell2.is_unique());
        assert_eq!(*cell, 42);
        assert_eq!(*cell2, 42);
        assert!(cell.get() == cell2.get());
    }

    #[test]
    fn mut_cell_drop() {
        let cell = MutCell::new(42);
        {
            let _cell2 = cell.clone();
            assert!(cell.is_shared());
        } // _cell2 goes out of scope, ref count should decrease

        assert!(cell.is_unique());
        drop(cell); // Should not panic
    }

    #[test]
    fn mut_cell_deref() {
        let mut cell = MutCell::new(42);
        assert_eq!(*cell, 42);
        let mut_ref = cell.get_mut();
        *mut_ref = 100;
        assert_eq!(*cell, 100);
    }
}
