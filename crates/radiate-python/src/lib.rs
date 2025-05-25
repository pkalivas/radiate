mod builder;
mod codex;
mod conversion;
mod engines;
mod epoch;
mod evaluator;
mod metric;
mod object;
mod params;
mod problem;
mod random;

pub use builder::*;
pub use codex::{PyBitCodex, PyCharCodex, PyFloatCodex, PyIntCodex};
pub use engines::*;
pub use epoch::*;
pub use evaluator::PyEvaluator;
pub use metric::*;
pub use object::*;
pub use params::*;
pub use problem::{PyProblem, ThreadSafePythonFn};
use pyo3::{PyResult, Python};
pub use random::PyRandomProvider;
use std::cell::UnsafeCell;

// Adapted from PYO3 with the only change that
// we allow mutable access with when the GIL is held

pub struct GILOnceCell<T>(UnsafeCell<Option<T>>);

// T: Send is needed for Sync because the thread which drops the GILOnceCell can be different
// to the thread which fills it.
unsafe impl<T: Send + Sync> Sync for GILOnceCell<T> {}
unsafe impl<T: Send> Send for GILOnceCell<T> {}

impl<T> GILOnceCell<T> {
    /// Create a `GILOnceCell` which does not yet contain a value.
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    pub fn is_initialized(&self) -> bool {
        // Safe because GIL is held, so no other thread can be writing to this cell concurrently.
        unsafe { &*self.0.get() }.is_some()
    }

    /// as long as we have the GIL we can mutate
    /// this creates a context that checks that.
    pub fn with_gil<F, O>(&self, _py: Python<'_>, mut op: F) -> PyResult<O>
    where
        F: FnMut(&mut T) -> PyResult<O>,
    {
        // Safe because GIL is held, so no other thread can be writing to this cell concurrently.
        let inner = unsafe { &mut *self.0.get() }
            .as_mut()
            .expect("not yet initialized");

        op(inner)
    }

    /// Set the value in the cell.
    ///
    /// If the cell has already been written, `Err(value)` will be returned containing the new
    /// value which was not written.
    pub fn set(&self, _py: Python<'_>, value: T) -> Result<(), T> {
        // Safe because GIL is held, so no other thread can be writing to this cell concurrently.
        let inner = unsafe { &mut *self.0.get() };
        if inner.is_some() {
            return Err(value);
        }

        *inner = Some(value);
        Ok(())
    }
}
