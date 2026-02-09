use core::mem::{ManuallyDrop, MaybeUninit};
use core::ptr;
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ---------------- InlineBuffer ----------------
pub struct InlineBuffer<T, const N: usize> {
    len: usize,
    buf: [MaybeUninit<T>; N],
}

impl<T, const N: usize> InlineBuffer<T, N> {
    #[inline]
    pub fn new() -> Self {
        Self {
            len: 0,
            buf: [const { MaybeUninit::uninit() }; N],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr() as *const T, self.len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut T, self.len) }
    }

    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }
        unsafe {
            self.buf.get_unchecked_mut(self.len).write(value);
        }
        self.len += 1;
        Ok(())
    }

    /// Move element i out (no drop); caller must ensure i < len.
    #[inline]
    unsafe fn read_unchecked(&mut self, i: usize) -> T {
        unsafe { self.buf.get_unchecked(i).assume_init_read() }
    }

    /// Drops all initialized elements and sets len=0.
    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.len {
            unsafe {
                ptr::drop_in_place(self.buf.get_unchecked_mut(i).as_mut_ptr());
            }
        }
        self.len = 0;
    }

    #[inline]
    pub fn drain(self) -> Drain<T, N> {
        Drain::new(self)
    }
}

impl<T, const N: usize> Drop for InlineBuffer<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: Clone, const N: usize> Clone for InlineBuffer<T, N> {
    fn clone(&self) -> Self {
        let mut out = Self::new();
        for x in self.as_slice() {
            out.push(x.clone()).ok().unwrap();
        }
        out
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize, const N: usize> Serialize for InlineBuffer<T, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for InlineBuffer<T, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        if vec.len() > N {
            return Err(serde::de::Error::custom(format!(
                "Too many elements for InlineBuffer<{}, {}>: {}",
                core::any::type_name::<T>(),
                N,
                vec.len()
            )));
        }
        let mut out = Self::new();
        for x in vec {
            out.push(x).ok().unwrap();
        }
        Ok(out)
    }
}

/// Allocation-free draining iterator for InlineBuffer.
pub struct Drain<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    idx: usize,
    len: usize,
}
impl<T, const N: usize> Drain<T, N> {
    fn new(v: InlineBuffer<T, N>) -> Self {
        let v = ManuallyDrop::new(v);
        let buf = unsafe { ptr::read(&v.buf) };
        Self {
            buf,
            idx: 0,
            len: v.len,
        }
    }
}
impl<T, const N: usize> Iterator for Drain<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.idx == self.len {
            return None;
        }
        let i = self.idx;
        self.idx += 1;
        Some(unsafe { self.buf.get_unchecked(i).assume_init_read() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.idx;
        (remaining, Some(remaining))
    }
}

impl<T, const N: usize> ExactSizeIterator for Drain<T, N> {
    fn len(&self) -> usize {
        self.len - self.idx
    }
}

impl<T, const N: usize> Drop for Drain<T, N> {
    fn drop(&mut self) {
        for i in self.idx..self.len {
            unsafe {
                ptr::drop_in_place(self.buf.get_unchecked_mut(i).as_mut_ptr());
            }
        }
    }
}

// ---------------- Buffer tiers ----------------

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Buffer<T> {
    // building
    Building(Vec<T>),

    // frozen tiers
    I1(InlineBuffer<T, 1>),
    I2(InlineBuffer<T, 2>),
    I4(InlineBuffer<T, 4>),
    I8(InlineBuffer<T, 8>),

    // big frozen
    Heap(Vec<T>),
}

impl<T> Default for Buffer<T> {
    fn default() -> Self {
        Buffer::Building(Vec::new())
    }
}

impl<T> Buffer<T> {
    pub fn new(ins: impl Into<Vec<T>>) -> Self {
        let mut b = Buffer::Building(ins.into());
        b.freeze();
        b
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        match self {
            Buffer::Building(v) => v.push(value),
            _ => panic!("Buffer is frozen; cannot push"),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Buffer::Building(v) => v.len(),
            Buffer::I1(b) => b.len(),
            Buffer::I2(b) => b.len(),
            Buffer::I4(b) => b.len(),
            Buffer::I8(b) => b.len(),
            Buffer::Heap(v) => v.len(),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        match self {
            Buffer::Building(v) => v.as_slice(),
            Buffer::I1(b) => b.as_slice(),
            Buffer::I2(b) => b.as_slice(),
            Buffer::I4(b) => b.as_slice(),
            Buffer::I8(b) => b.as_slice(),
            Buffer::Heap(v) => v.as_slice(),
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Buffer::Building(v) => v.as_mut_slice(),
            Buffer::I1(b) => b.as_mut_slice(),
            Buffer::I2(b) => b.as_mut_slice(),
            Buffer::I4(b) => b.as_mut_slice(),
            Buffer::I8(b) => b.as_mut_slice(),
            Buffer::Heap(v) => v.as_mut_slice(),
        }
    }

    /// Freeze once: picks smallest tier and moves elements. After this, no growth.
    pub fn freeze(&mut self) {
        let v = match core::mem::replace(self, Buffer::Building(Vec::new())) {
            Buffer::Building(v) => v,
            already => {
                *self = already;
                return;
            }
        };

        *self = Self::freeze_from_vec(v);
    }

    fn freeze_from_vec(v: Vec<T>) -> Self {
        match v.len() {
            0 => Buffer::Heap(v),
            1 => {
                let mut it = v.into_iter();
                let mut b = InlineBuffer::<T, 1>::new();
                b.push(it.next().unwrap()).ok().unwrap();
                Buffer::I1(b)
            }
            2 => {
                let mut it = v.into_iter();
                let mut b = InlineBuffer::<T, 2>::new();
                b.push(it.next().unwrap()).ok().unwrap();
                b.push(it.next().unwrap()).ok().unwrap();
                Buffer::I2(b)
            }
            3 | 4 => {
                let mut b = InlineBuffer::<T, 4>::new();
                for x in v {
                    b.push(x).ok().unwrap();
                }
                Buffer::I4(b)
            }
            5..=8 => {
                let mut b = InlineBuffer::<T, 8>::new();
                for x in v {
                    b.push(x).ok().unwrap();
                }
                Buffer::I8(b)
            }
            _ => Buffer::Heap(v),
        }
    }

    /// Compacts *frozen* inline tiers down to the smallest tier that fits len.
    /// This is your “reserved 8 but only using 2 → shrink to 2”.
    pub fn compact(&mut self) {
        let len = self.len();

        // only affects inline tiers; building/heap are left as-is (you could also shrink heap).
        match self {
            Buffer::I8(_) | Buffer::I4(_) | Buffer::I2(_) | Buffer::I1(_) => {}
            _ => return,
        }

        // Move out of current inline tier into the smallest tier.
        // We do this by temporarily taking self and rebuilding.
        let tmp = core::mem::replace(self, Buffer::Heap(Vec::new()));
        *self = match tmp {
            Buffer::I8(mut b) => Self::compact_from_inline(&mut b, len),
            Buffer::I4(mut b) => Self::compact_from_inline(&mut b, len),
            Buffer::I2(b) => Buffer::I2(b),
            Buffer::I1(b) => Buffer::I1(b),
            other => other,
        };
    }

    fn compact_from_inline<const N: usize>(b: &mut InlineBuffer<T, N>, len: usize) -> Buffer<T> {
        match len {
            0 => Buffer::Heap(Vec::new()),
            1 => {
                let mut out = InlineBuffer::<T, 1>::new();
                // SAFETY: len==1
                let v0 = unsafe { b.read_unchecked(0) };
                out.push(v0).ok().unwrap();
                // prevent dropping moved element
                b.len = 0;
                Buffer::I1(out)
            }
            2 => {
                let mut out = InlineBuffer::<T, 2>::new();
                let v0 = unsafe { b.read_unchecked(0) };
                let v1 = unsafe { b.read_unchecked(1) };
                out.push(v0).ok().unwrap();
                out.push(v1).ok().unwrap();
                b.len = 0;
                Buffer::I2(out)
            }
            3 | 4 => {
                let mut out = InlineBuffer::<T, 4>::new();
                for i in 0..len {
                    let vi = unsafe { b.read_unchecked(i) };
                    out.push(vi).ok().unwrap();
                }
                b.len = 0;
                Buffer::I4(out)
            }
            _ => {
                // len 5..=8 stay I8; if N was 4 and len was 3/4 this already handled above.
                // For N=8 and len 5..=8, keep.
                // For safety, if len doesn't fit tiers, fallback heap.
                if len <= 8 {
                    let mut out = InlineBuffer::<T, 8>::new();
                    for i in 0..len {
                        let vi = unsafe { b.read_unchecked(i) };
                        out.push(vi).ok().unwrap();
                    }
                    b.len = 0;
                    Buffer::I8(out)
                } else {
                    // shouldn’t happen for inline inputs; but safe fallback
                    let mut out = Vec::with_capacity(len);
                    for i in 0..len {
                        let vi = unsafe { b.read_unchecked(i) };
                        out.push(vi);
                    }
                    b.len = 0;
                    Buffer::Heap(out)
                }
            }
        }
    }
}

impl<T: Clone> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        match self {
            Buffer::Building(v) => Buffer::Building(v.clone()),
            Buffer::I1(b) => Buffer::I1(b.clone()),
            Buffer::I2(b) => Buffer::I2(b.clone()),
            Buffer::I4(b) => Buffer::I4(b.clone()),
            Buffer::I8(b) => Buffer::I8(b.clone()),
            Buffer::Heap(v) => Buffer::Heap(v.clone()),
        }
    }
}

impl<T: PartialEq> PartialEq for Buffer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> FromIterator<T> for Buffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut b = Buffer::Building(iter.into_iter().collect());
        b.freeze();
        b
    }
}

impl<T: Debug> Debug for Buffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Buffer::Building(v) => write!(f, "B {:?}", v),
            Buffer::I1(b) => write!(f, "I1 {:?}", b.as_slice()),
            Buffer::I2(b) => write!(f, "I2 {:?}", b.as_slice()),
            Buffer::I4(b) => write!(f, "I4 {:?}", b.as_slice()),
            Buffer::I8(b) => write!(f, "I8 {:?}", b.as_slice()),
            Buffer::Heap(v) => write!(f, "H {:?}", v),
        }
    }
}

// IntoIterator: allocation-free for inline; heap uses Vec’s IntoIter.
// If you truly “don’t want iterators”, delete this block.
impl<T> IntoIterator for Buffer<T> {
    type Item = T;
    type IntoIter = BufferIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Buffer::I1(b) => BufferIntoIter::I1(b.drain()),
            Buffer::I2(b) => BufferIntoIter::I2(b.drain()),
            Buffer::I4(b) => BufferIntoIter::I4(b.drain()),
            Buffer::I8(b) => BufferIntoIter::I8(b.drain()),
            Buffer::Heap(v) => BufferIntoIter::Heap(v.into_iter()),
            Buffer::Building(v) => BufferIntoIter::Heap(v.into_iter()),
        }
    }
}

pub enum BufferIntoIter<T> {
    I1(Drain<T, 1>),
    I2(Drain<T, 2>),
    I4(Drain<T, 4>),
    I8(Drain<T, 8>),
    Heap(std::vec::IntoIter<T>),
}

impl<T> Iterator for BufferIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self {
            BufferIntoIter::I1(it) => it.next(),
            BufferIntoIter::I2(it) => it.next(),
            BufferIntoIter::I4(it) => it.next(),
            BufferIntoIter::I8(it) => it.next(),
            BufferIntoIter::Heap(it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BufferIntoIter::I1(it) => it.size_hint(),
            BufferIntoIter::I2(it) => it.size_hint(),
            BufferIntoIter::I4(it) => it.size_hint(),
            BufferIntoIter::I8(it) => it.size_hint(),
            BufferIntoIter::Heap(it) => it.size_hint(),
        }
    }
}

impl<T> ExactSizeIterator for BufferIntoIter<T> {}
