use std::fmt::Debug;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct Slot {
    version: u64,
    count: u32,
}

/// A reusable counter keyed by `usize` with O(1) "clear" between sessions.
///
/// Storage is positional — `bucket[idx]` holds the count for source index `idx`.
/// Each session bumps a `version` stamp; slots are considered live only when their
/// stamp matches the current version, so a session "clear" is a single increment
/// instead of a vec-wide write.
///
/// Designed for hot paths that previously allocated a fresh `HashMap<usize, u32>`
/// per call. After the first `begin(capacity)` warms the buffer, subsequent
/// sessions allocate nothing.
#[derive(Clone)]
pub struct VersionedCounts {
    buckets: Vec<Slot>,
    current: u64,
}

impl VersionedCounts {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            current: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut s = Self::new();
        s.begin(capacity);
        s
    }

    #[inline]
    pub fn unique_count(&self) -> usize {
        self.iter_live().count()
    }

    /// Start a new session. Resizes if needed and bumps the version so all
    /// previously-touched slots read as stale on the first `bump`.
    #[inline]
    pub fn begin(&mut self, capacity: usize) {
        if self.buckets.len() < capacity {
            self.buckets.resize(capacity, Slot::default());
        }
        self.current = self.current.wrapping_add(1);
    }

    /// Increment the count at `idx`, returning the new count for this session.
    /// Stale slots are reset to `1` before counting.
    #[inline]
    pub fn bump(&mut self, idx: usize) -> u32 {
        let slot = &mut self.buckets[idx];
        if slot.version != self.current {
            slot.version = self.current;
            slot.count = 1;
        } else {
            slot.count += 1;
        }
        slot.count
    }

    /// Read the count at `idx` for the current session.
    /// Returns `0` if the slot is out of bounds or stale.
    #[inline]
    pub fn get(&self, idx: usize) -> u32 {
        match self.buckets.get(idx) {
            Some(s) if s.version == self.current => s.count,
            _ => 0,
        }
    }

    /// Live `(idx, count)` pairs in ascending idx order.
    #[inline]
    pub fn iter_live(&self) -> impl Iterator<Item = (usize, u32)> + '_ {
        let cur = self.current;
        self.buckets
            .iter()
            .enumerate()
            .filter_map(move |(i, s)| (s.version == cur).then_some((i, s.count)))
    }

    /// Live `(idx, count)` pairs in descending idx order — convenient for
    /// callers that need to mutate an indexed collection without invalidating
    /// not-yet-processed indices (e.g. `Vec::swap_remove`).
    #[inline]
    pub fn iter_live_rev(&self) -> impl Iterator<Item = (usize, u32)> + '_ {
        let cur = self.current;
        let len = self.buckets.len();
        (0..len).rev().filter_map(move |i| {
            let s = &self.buckets[i];
            (s.version == cur).then_some((i, s.count))
        })
    }

    /// Walk both buffers in parallel in descending idx order, yielding
    /// `(idx, count_self, count_other)` for any idx where at least one side
    /// is live this session. Slots stale or out-of-bounds on either side
    /// contribute `0` for that side. Stops at `max(self.len, other.len)`.
    #[inline]
    pub fn iter_pair_live_rev<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = (usize, u32, u32)> + 'a {
        let cur_self = self.current;
        let cur_other = other.current;
        let len = self.buckets.len().max(other.buckets.len());
        (0..len).rev().filter_map(move |i| {
            let a = self
                .buckets
                .get(i)
                .filter(|s| s.version == cur_self)
                .map(|s| s.count)
                .unwrap_or(0);
            let b = other
                .buckets
                .get(i)
                .filter(|s| s.version == cur_other)
                .map(|s| s.count)
                .unwrap_or(0);
            if a == 0 && b == 0 {
                None
            } else {
                Some((i, a, b))
            }
        })
    }
}

impl Default for VersionedCounts {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for VersionedCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VersionedCounts")
            .field("capacity", &self.buckets.len())
            .field("current", &self.current)
            .field("live", &self.iter_live().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_session_is_empty() {
        let mut c = VersionedCounts::new();
        c.begin(8);
        assert_eq!(c.iter_live().count(), 0);
    }

    #[test]
    fn bump_counts_per_session() {
        let mut c = VersionedCounts::new();
        c.begin(8);
        assert_eq!(c.bump(3), 1);
        assert_eq!(c.bump(3), 2);
        assert_eq!(c.bump(5), 1);
        assert_eq!(c.bump(3), 3);

        let mut live: Vec<_> = c.iter_live().collect();
        live.sort_by_key(|(i, _)| *i);
        assert_eq!(live, vec![(3, 3), (5, 1)]);
    }

    #[test]
    fn begin_clears_previous_session() {
        let mut c = VersionedCounts::new();
        c.begin(8);
        c.bump(3);
        c.bump(7);
        assert_eq!(c.iter_live().count(), 2);

        c.begin(8);
        assert_eq!(c.iter_live().count(), 0);

        c.bump(2);
        let live: Vec<_> = c.iter_live().collect();
        assert_eq!(live, vec![(2, 1)]);
    }

    #[test]
    fn iter_live_rev_yields_descending() {
        let mut c = VersionedCounts::new();
        c.begin(10);
        c.bump(7);
        c.bump(2);
        c.bump(5);
        c.bump(2);

        let rev: Vec<_> = c.iter_live_rev().collect();
        assert_eq!(rev, vec![(7, 1), (5, 1), (2, 2)]);
    }

    #[test]
    fn begin_grows_capacity_but_keeps_history() {
        let mut c = VersionedCounts::new();
        c.begin(4);
        c.bump(0);
        c.begin(16);
        assert_eq!(c.iter_live().count(), 0);
        c.bump(10);
        assert_eq!(c.iter_live().collect::<Vec<_>>(), vec![(10, 1)]);
    }

    #[test]
    fn iter_pair_live_rev_yields_union_descending() {
        let mut a = VersionedCounts::new();
        let mut b = VersionedCounts::new();
        a.begin(10);
        b.begin(10);

        // overlapping at idx 5; a-only at idx 7; b-only at idx 2.
        a.bump(5);
        a.bump(5);
        a.bump(7);
        b.bump(5);
        b.bump(2);
        b.bump(2);
        b.bump(2);

        let pairs: Vec<_> = a.iter_pair_live_rev(&b).collect();
        assert_eq!(pairs, vec![(7, 1, 0), (5, 2, 1), (2, 0, 3)]);
    }

    #[test]
    fn iter_pair_live_rev_handles_stale_other() {
        let mut a = VersionedCounts::new();
        let mut b = VersionedCounts::new();
        a.begin(8);
        b.begin(8);
        a.bump(4);
        b.bump(4);

        // Re-begin b without bumping anything — its slots are now stale.
        b.begin(8);

        let pairs: Vec<_> = a.iter_pair_live_rev(&b).collect();
        assert_eq!(pairs, vec![(4, 1, 0)]);
    }
}
