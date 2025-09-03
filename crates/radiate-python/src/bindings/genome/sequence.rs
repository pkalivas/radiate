use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct RwSequence<T> {
    values: Arc<RwLock<Vec<T>>>,
}

impl<T> RwSequence<T> {
    pub fn new(genes: Vec<T>) -> Self {
        RwSequence {
            values: Arc::new(RwLock::new(genes)),
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, Vec<T>> {
        self.values.read().unwrap()
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, Vec<T>> {
        self.values.write().unwrap()
    }

    pub fn len(&self) -> usize {
        self.values.read().unwrap().len()
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.values)
    }

    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.values)
    }

    pub fn take(&self) -> Vec<T> {
        std::mem::take(&mut *self.values.write().unwrap())
    }
}

impl<T: PartialEq> PartialEq for RwSequence<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.values.read().unwrap() == *other.values.read().unwrap()
    }
}
