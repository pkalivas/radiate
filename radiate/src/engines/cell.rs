use super::{Chromosome, EngineContext};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Clone)]
pub struct EngineCell<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    inner: Arc<RwLock<EngineContext<C, T>>>,
}

impl<C, T> EngineCell<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    pub fn new(inner: EngineContext<C, T>) -> Self {
        EngineCell {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn read(&self) -> EngineCellGuard<C, T> {
        let guard = self.inner.read().unwrap();
        EngineCellGuard { inner: guard }
    }

    pub fn write(&self) -> EngineCellGuardMut<C, T> {
        let guard = self.inner.write().unwrap();
        EngineCellGuardMut { inner: guard }
    }
}

impl<C, T> Debug for EngineCell<C, T>
where
    C: Chromosome + Debug + 'static,
    T: Clone + Send + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.read();
        write!(f, "EngineCell {{ inner: {:?} }}", guard)
    }
}

pub struct EngineCellGuard<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    inner: RwLockReadGuard<'a, EngineContext<C, T>>,
}

impl<'a, C, T> Deref for EngineCellGuard<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    type Target = EngineContext<C, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, C, T> Debug for EngineCellGuard<'a, C, T>
where
    C: Chromosome + Debug + 'static,
    T: Clone + Send + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineCellGuard {{ inner: {:?} }}", self.inner)
    }
}

pub struct EngineCellGuardMut<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    inner: RwLockWriteGuard<'a, EngineContext<C, T>>,
}

impl<'a, C, T> Deref for EngineCellGuardMut<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    type Target = EngineContext<C, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, C, T> DerefMut for EngineCellGuardMut<'a, C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, C, T> Debug for EngineCellGuardMut<'a, C, T>
where
    C: Chromosome + Debug + 'static,
    T: Clone + Send + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineCellGuardMut {{ inner: {:?} }}", self.inner)
    }
}
