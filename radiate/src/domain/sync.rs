use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Debug)]
pub struct RwCell<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> RwCell<T> {
    pub fn new(value: T) -> Self {
        RwCell {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    pub fn clone(other: &RwCell<T>) -> Self {
        RwCell {
            inner: Arc::clone(&other.inner),
        }
    }

    pub fn into_inner(self) -> T {
        Arc::try_unwrap(self.inner)
            .ok()
            .expect("Multiple references to SyncCell exist")
            .into_inner()
            .expect("RwLock poisoned")
    }

    pub fn inner(&self) -> &Arc<RwLock<T>> {
        &self.inner
    }

    pub fn read(&self) -> RwCellGuard<T> {
        let read_lock = self.inner.read().unwrap();
        RwCellGuard { inner: read_lock }
    }

    pub fn write(&self) -> RwCellGuardMut<T> {
        let write_lock = self.inner.write().unwrap();
        RwCellGuardMut { inner: write_lock }
    }

    pub fn set(&self, value: T) {
        let mut write_lock = self.inner.write().unwrap();
        *write_lock = value;
    }
}

impl<T: PartialEq> PartialEq for RwCell<T> {
    fn eq(&self, other: &Self) -> bool {
        let self_value = self.read();
        let other_value = other.read();
        (*self_value) == (*other_value)
    }
}

impl<T: Clone> Clone for RwCell<T> {
    fn clone(&self) -> Self {
        let inner = self.inner.read().unwrap().clone();
        RwCell {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
}

#[derive(Debug)]
pub struct RwCellGuard<'a, T> {
    inner: RwLockReadGuard<'a, T>,
}

impl<T> RwCellGuard<'_, T> {
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Deref for RwCellGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> AsRef<T> for RwCellGuard<'_, T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T: PartialEq> PartialEq for RwCellGuard<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        (*self.inner) == (*other.inner)
    }
}

impl<T: Eq> Eq for RwCellGuard<'_, T> {}

impl<T: Hash> Hash for RwCellGuard<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

pub struct RwCellGuardMut<'a, T> {
    inner: RwLockWriteGuard<'a, T>,
}

impl<T> Deref for RwCellGuardMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for RwCellGuardMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: PartialEq> PartialEq for RwCellGuardMut<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        (*self.inner) == (*other.inner)
    }
}

impl<T: Eq> Eq for RwCellGuardMut<'_, T> {}

impl<T: Hash> Hash for RwCellGuardMut<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}
