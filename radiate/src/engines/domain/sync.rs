use crate::{Chromosome, Phenotype};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Debug)]
pub struct SyncCell<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> SyncCell<T> {
    pub fn new(value: T) -> Self {
        SyncCell {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    pub fn clone(other: &SyncCell<T>) -> Self {
        SyncCell {
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

    pub fn read(&self) -> SyncCellGuard<T> {
        let read_lock = self.inner.read().unwrap();
        SyncCellGuard { inner: read_lock }
    }

    pub fn write(&self) -> SyncCellGuardMut<T> {
        let write_lock = self.inner.write().unwrap();
        SyncCellGuardMut { inner: write_lock }
    }

    pub fn set(&self, value: T) {
        let mut write_lock = self.inner.write().unwrap();
        *write_lock = value;
    }
}

impl<T: PartialEq> PartialEq for SyncCell<T> {
    fn eq(&self, other: &Self) -> bool {
        let self_value = self.read();
        let other_value = other.read();
        (*self_value) == (*other_value)
    }
}

impl<T: Clone> Clone for SyncCell<T> {
    fn clone(&self) -> Self {
        let inner = self.inner.read().unwrap().clone();
        SyncCell {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
}

impl<C: Chromosome> From<Phenotype<C>> for SyncCell<Phenotype<C>> {
    fn from(individual: Phenotype<C>) -> Self {
        SyncCell::new(individual)
    }
}

#[derive(Debug)]
pub struct SyncCellGuard<'a, T> {
    inner: RwLockReadGuard<'a, T>,
}

impl<T> SyncCellGuard<'_, T> {
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Deref for SyncCellGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> AsRef<T> for SyncCellGuard<'_, T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

pub struct SyncCellGuardMut<'a, T> {
    inner: RwLockWriteGuard<'a, T>,
}

impl<T> Deref for SyncCellGuardMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for SyncCellGuardMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
