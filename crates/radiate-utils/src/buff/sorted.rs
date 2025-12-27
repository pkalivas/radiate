#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "smallvec")]
use smallvec::SmallVec;
use std::{fmt::Debug, ops::Deref};

#[cfg(feature = "smallvec")]
pub type InnerBuff<T> = SmallVec<[T; 8]>;

#[cfg(not(feature = "smallvec"))]
pub type InnerBuff<T> = Vec<T>;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct SortedBuffer<T> {
    inner: InnerBuff<T>,
}

impl<T> SortedBuffer<T> {
    pub fn new() -> Self {
        SortedBuffer {
            inner: InnerBuff::new(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.inner
    }

    #[inline]
    pub fn contains(&self, value: &T) -> bool
    where
        T: Ord,
    {
        self.inner.binary_search(value).is_ok()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.inner.iter()
    }

    #[inline]
    pub fn insert_sorted_unique(v: &mut SortedBuffer<T>, value: T)
    where
        T: Ord,
    {
        match v.inner.binary_search(&value) {
            Ok(_) => {}
            Err(pos) => v.inner.insert(pos, value),
        }
    }

    #[inline]
    pub fn remove_sorted(v: &mut SortedBuffer<T>, value: &T)
    where
        T: Ord,
    {
        if let Ok(pos) = v.inner.binary_search(value) {
            v.inner.remove(pos);
        }
    }

    #[inline]
    pub fn set_sorted_unique(dst: &mut SortedBuffer<T>, src: impl IntoIterator<Item = T>)
    where
        T: Ord,
    {
        dst.inner.clear();
        dst.inner.extend(src);
        dst.inner.sort_unstable();
        dst.inner.dedup();
    }
}

impl<T> Deref for SortedBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> FromIterator<T> for SortedBuffer<T>
where
    T: Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut buffer = SortedBuffer::new();
        SortedBuffer::set_sorted_unique(&mut buffer, iter);
        buffer
    }
}

impl<T, I> From<I> for SortedBuffer<T>
where
    I: IntoIterator<Item = T>,
    T: Ord,
{
    fn from(iter: I) -> Self {
        let mut buffer = SortedBuffer::new();
        SortedBuffer::set_sorted_unique(&mut buffer, iter);
        buffer
    }
}

impl<T> Debug for SortedBuffer<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "smallvec")]
        {
            write!(f, "SV {:?}", self.inner.as_slice())?;
            return Ok(());
        }
        #[cfg(not(feature = "smallvec"))]
        {
            write!(f, "V {:?}", self.inner.as_slice())?;
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_buffer_insert_remove() {
        let mut buffer = SortedBuffer::new();
        SortedBuffer::insert_sorted_unique(&mut buffer, 5);
        SortedBuffer::insert_sorted_unique(&mut buffer, 3);
        SortedBuffer::insert_sorted_unique(&mut buffer, 8);
        SortedBuffer::insert_sorted_unique(&mut buffer, 5); // Duplicate, should not be added

        assert_eq!(&*buffer, &[3, 5, 8]);

        SortedBuffer::remove_sorted(&mut buffer, &5);
        assert_eq!(&*buffer, &[3, 8]);

        SortedBuffer::remove_sorted(&mut buffer, &10); // Not present, should do nothing
        assert_eq!(&*buffer, &[3, 8]);
    }

    #[test]
    fn test_sorted_buffer_from_iter() {
        let vec = vec![4, 2, 7, 2, 5];
        let buffer: SortedBuffer<i32> = vec.into_iter().collect();
        assert_eq!(&*buffer, &[2, 4, 5, 7]);
    }
}
