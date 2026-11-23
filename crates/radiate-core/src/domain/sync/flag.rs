//! A one-bit signal split into a write half ([`Flag`]) and a read half
//! ([`Signal`]).
//!
//! The owner keeps the [`Flag`] and lends out as many [`Signal`]s as it likes.
//! A `Signal` can only observe, so whoever holds one can loop on it
//! (`while !signal.is_raised() { .. }`) but can never raise or lower it,
//! and so can never bypass whatever the owner does alongside the write
//! (notifying a condvar, removing a queue entry, publishing an event).

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Write half. Raising it is visible to every [`Signal`] lent from it.
///
/// `Clone` hands out more *write* capability; owners that need to pair the
/// write with a side effect should keep the `Flag` private and expose their
/// own method instead (see `ThreadSync::stop`).
#[derive(Clone, Debug, Default)]
pub struct Flag(Arc<AtomicBool>);

impl Flag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Raise the flag. Returns `true` if this call raised it (it was lowered).
    #[inline]
    pub fn raise(&self) -> bool {
        !self.0.swap(true, Ordering::Release)
    }

    /// Lower the flag. Returns `true` if this call lowered it.
    #[inline]
    pub fn lower(&self) -> bool {
        self.0.swap(false, Ordering::Release)
    }

    #[inline]
    pub fn store(&self, raised: bool) {
        self.0.store(raised, Ordering::Release);
    }

    #[inline]
    pub fn is_raised(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// Lend out a read-only view.
    pub fn reader(&self) -> FlagReader {
        FlagReader(Arc::clone(&self.0))
    }
}

/// Read half. Cheap to clone; cannot change the flag.
///
/// `Acquire` loads pair with the `Release` writes in [`Flag`], so anything the
/// writer did before raising is visible to a reader that sees it raised.
#[derive(Clone, Debug)]
pub struct FlagReader(Arc<AtomicBool>);

impl FlagReader {
    pub fn never() -> Self {
        FlagReader(Arc::new(AtomicBool::new(false)))
    }

    #[inline]
    pub fn is_raised(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// True if both signals read the same flag.
    pub fn same_as(&self, other: &FlagReader) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Default for FlagReader {
    fn default() -> Self {
        Self::never()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals_see_raise_and_lower() {
        let flag = Flag::new();
        let (a, b) = (flag.reader(), flag.reader());
        assert!(!a.is_raised());
        assert!(flag.raise());
        assert!(!flag.raise(), "second raise is a no-op");
        assert!(a.is_raised() && b.is_raised());
        assert!(flag.lower());
        assert!(!b.is_raised());
        assert!(a.same_as(&b));
    }

    #[test]
    fn never_is_never_raised() {
        assert!(!FlagReader::never().is_raised());
    }
}
