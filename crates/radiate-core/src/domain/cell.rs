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

/// A thin, clone-on-write, thread-safe smart pointer for `[T]`
pub struct MutSlice<T> {
    data: *mut [T],
    ref_count: *const AtomicUsize,
}

impl<T> MutSlice<T> {
    /// Creates a new `MutSlice` from a boxed slice
    pub fn new(slice: Box<[T]>) -> Self {
        let data = Box::into_raw(slice);
        let ref_count = Box::into_raw(Box::new(AtomicUsize::new(1)));
        Self { data, ref_count }
    }

    /// Returns a shared reference to the slice
    pub fn get(&self) -> &[T] {
        unsafe { &*self.data }
    }

    /// Returns a mutable reference only if uniquely owned
    pub fn get_mut(&mut self) -> Option<&mut [T]> {
        if unsafe { (*self.ref_count).load(Ordering::Acquire) } == 1 {
            Some(unsafe { &mut *self.data })
        } else {
            None
        }
    }

    /// Returns the inner boxed slice if uniquely owned, else `None`
    pub fn into_inner(self) -> Option<Box<[T]>>
    where
        T: Clone,
    {
        if unsafe { (*self.ref_count).load(Ordering::Acquire) } == 1 {
            std::sync::atomic::fence(Ordering::SeqCst);
            unsafe {
                drop(Box::from_raw(self.ref_count as *mut AtomicUsize));
                Some(Box::from_raw(self.data))
            }
        } else {
            None
        }
    }

    /// Forces uniqueness by cloning if there are multiple owners
    pub fn make_unique(&mut self)
    where
        T: Clone,
    {
        if unsafe { (*self.ref_count).load(Ordering::Acquire) } != 1 {
            // Clone data
            let cloned = self.get().to_vec().into_boxed_slice();
            let new_data = Box::into_raw(cloned);
            let new_ref = Box::into_raw(Box::new(AtomicUsize::new(1)));

            // Drop the old reference if we're last
            if unsafe { (*self.ref_count).fetch_sub(1, Ordering::Release) == 1 } {
                std::sync::atomic::fence(Ordering::Acquire);
                unsafe {
                    drop(Box::from_raw(self.data));
                    drop(Box::from_raw(self.ref_count as *mut AtomicUsize));
                }
            }

            self.data = new_data;
            self.ref_count = new_ref;
        }
    }

    /// Forces a boxed clone
    pub fn clone_owned(&self) -> Box<[T]>
    where
        T: Clone,
    {
        self.get().to_vec().into_boxed_slice()
    }
}

impl<T> Clone for MutSlice<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.ref_count).fetch_add(1, Ordering::Relaxed);
        }
        Self {
            data: self.data,
            ref_count: self.ref_count,
        }
    }
}

impl<T> Drop for MutSlice<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.ref_count).fetch_sub(1, Ordering::Release) == 1 {
                std::sync::atomic::fence(Ordering::Acquire);
                drop(Box::from_raw(self.data));
                drop(Box::from_raw(self.ref_count as *mut AtomicUsize));
            }
        }
    }
}

impl<T> Deref for MutSlice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T: Clone> DerefMut for MutSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.make_unique();
        unsafe { &mut *self.data }
    }
}

impl<T> AsRef<[T]> for MutSlice<T> {
    fn as_ref(&self) -> &[T] {
        self.get()
    }
}

impl<T> PartialEq for MutSlice<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}
impl<T: PartialOrd> PartialOrd for MutSlice<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(other.get())
    }
}
impl<T> From<Box<[T]>> for MutSlice<T> {
    fn from(slice: Box<[T]>) -> Self {
        Self::new(slice)
    }
}
impl<T> From<&[T]> for MutSlice<T>
where
    T: Clone,
{
    fn from(slice: &[T]) -> Self {
        Self::new(slice.to_vec().into_boxed_slice())
    }
}
impl<T> From<&MutSlice<T>> for MutSlice<T>
where
    T: Clone,
{
    fn from(slice: &MutSlice<T>) -> Self {
        slice.clone()
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

    #[test]
    fn mutslice_basic_clone_and_mutation() {
        let slice = MutSlice::new(vec![1, 2, 3].into_boxed_slice());
        assert_eq!(&*slice, &[1, 2, 3]);

        let mut slice2 = slice.clone();
        assert_eq!(&*slice2, &[1, 2, 3]);

        // Mutate slice2, which should trigger clone-on-write
        slice2[0] = 10;
        assert_eq!(&*slice2, &[10, 2, 3]);
        // Original slice remains unchanged
        assert_eq!(&*slice, &[1, 2, 3]);
    }

    #[test]
    fn mutslice_get_mut_when_unique() {
        let mut slice = MutSlice::new(vec![1, 2, 3].into_boxed_slice());
        assert!(slice.get_mut().is_some());

        let slice2 = slice.clone();
        assert!(slice.get_mut().is_none());

        drop(slice2);
        assert!(slice.get_mut().is_some());
    }

    #[test]
    fn mutslice_into_inner_unique_and_non_unique() {
        let slice = MutSlice::new(vec![1, 2, 3].into_boxed_slice());
        // Unique ownership, should succeed
        let boxed = slice.into_inner();
        assert_eq!(&*boxed.unwrap(), &[1, 2, 3]);

        let slice = MutSlice::new(vec![1, 2, 3].into_boxed_slice());
        let slice2 = slice.clone();
        // Multiple owners, should return None
        assert!(slice.into_inner().is_none());

        drop(slice2);
    }

    #[test]
    fn mutslice_make_unique_and_clone_owned() {
        let mut slice = MutSlice::new(vec![1, 2, 3].into_boxed_slice());
        let slice2 = slice.clone();

        slice.make_unique();
        // After make_unique, it should be unique and mutable
        assert!(slice.get_mut().is_some());

        // clone_owned should produce a boxed slice equal to current data
        let cloned = slice.clone_owned();
        assert_eq!(&*cloned, &[1, 2, 3]);

        drop(slice2);
    }

    #[test]
    fn mutslice_deref_and_deref_mut() {
        let mut slice = MutSlice::new(vec![1, 2, 3].into_boxed_slice());
        assert_eq!(&*slice, &[1, 2, 3]);

        let slice2 = slice.clone();
        // DerefMut triggers clone-on-write
        let mut_ref = &mut *slice;
        mut_ref[0] = 42;

        // slice2 remains unchanged
        assert_eq!(&*slice2, &[1, 2, 3]);
        assert_eq!(&*slice, &[42, 2, 3]);
    }

    #[test]
    fn drops_do_not_leak() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        struct DropCounter(Arc<AtomicBool>);
        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let dropped = Arc::new(AtomicBool::new(false));
        {
            let dc = DropCounter(dropped.clone());
            let cell = MutCell::new(dc);
            let _cell2 = cell.clone();
            // When both dropped, DropCounter should be dropped once
        }
        assert!(dropped.load(Ordering::SeqCst));

        let dropped = Arc::new(AtomicBool::new(false));
        {
            let dc = DropCounter(dropped.clone());
            let slice = MutSlice::new(vec![dc].into_boxed_slice());
            let _slice2 = slice.clone();
            // When both dropped, DropCounter should be dropped once
        }
        assert!(dropped.load(Ordering::SeqCst));
    }
}
