mod aggregate;
mod builder;
mod chromosome;
mod codec;
mod crossover;
mod diversity;
mod eval;
mod graph;
mod iter;
mod mutation;
mod node;
mod replacement;
mod transaction;

pub use aggregate::GraphAggregate;
pub use chromosome::GraphChromosome;
pub use codec::GraphCodec;
pub use crossover::GraphCrossover;
pub use diversity::NeatDistance;
pub use eval::{GraphEvalCache, GraphEvaluator};
pub use graph::Graph;
pub use iter::GraphIterator;
pub use mutation::GraphMutator;
pub use node::{Direction, GraphNode, GraphNodeId};
pub use replacement::GraphReplacement;
pub use transaction::GraphTransaction;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::SmallVec;

/// A tiered, stack-backed buffer that upgrades through multiple inline sizes
/// before falling back to a heap Vec.
#[derive(Clone, PartialEq)]
pub enum Buffer<T> {
    S8(SmallVec<[T; 8]>),
    S16(SmallVec<[T; 16]>),
    S32(SmallVec<[T; 32]>),
    Heap(Vec<T>),
}

impl<T> Default for Buffer<T> {
    fn default() -> Self {
        Self::S8(SmallVec::new())
    }
}

impl<T> Buffer<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    pub fn clear(&mut self) {
        match self {
            Buffer::S8(b) => b.clear(),
            Buffer::S16(b) => b.clear(),
            Buffer::S32(b) => b.clear(),
            Buffer::Heap(b) => b.clear(),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        match self {
            Buffer::S8(b) => b.as_slice(),
            Buffer::S16(b) => b.as_slice(),
            Buffer::S32(b) => b.as_slice(),
            Buffer::Heap(b) => b.as_slice(),
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Buffer::S8(b) => b.as_mut_slice(),
            Buffer::S16(b) => b.as_mut_slice(),
            Buffer::S32(b) => b.as_mut_slice(),
            Buffer::Heap(b) => b.as_mut_slice(),
        }
    }

    /// Reserve one more slot; upgrades tier as needed.
    #[inline]
    fn reserve_one(&mut self) {
        let len = self.len();
        match self {
            Buffer::S8(b) if len == b.capacity() => {
                let mut n: SmallVec<[T; 16]> = SmallVec::with_capacity(len.max(16));
                n.extend(b.drain(..));
                *self = Buffer::S16(n);
            }
            Buffer::S16(b) if len == b.capacity() => {
                let mut n: SmallVec<[T; 32]> = SmallVec::with_capacity(len.max(32));
                n.extend(b.drain(..));
                *self = Buffer::S32(n);
            }
            Buffer::S32(b) if len == b.capacity() => {
                let mut n: Vec<T> = Vec::with_capacity(len.checked_mul(2).unwrap_or(len + 1));
                n.extend(b.drain(..));
                *self = Buffer::Heap(n);
            }
            Buffer::Heap(b) if len == b.capacity() => {
                b.reserve(if len == 0 { 4 } else { len });
            }
            _ => {}
        }
    }

    #[inline]
    pub fn insert_sorted_unique(&mut self, value: T)
    where
        T: Ord + Clone,
    {
        let slice = self.as_slice();

        match slice.binary_search(&value) {
            Ok(_) => return,
            Err(pos) => {
                self.reserve_one();
                match self {
                    Buffer::S8(b) => b.insert(pos, value),
                    Buffer::S16(b) => b.insert(pos, value),
                    Buffer::S32(b) => b.insert(pos, value),
                    Buffer::Heap(b) => b.insert(pos, value),
                }
            }
        }
    }

    #[inline]
    pub fn remove_sorted(&mut self, value: &T) -> bool
    where
        T: Ord,
    {
        let slice = self.as_slice();
        if let Ok(pos) = slice.binary_search(value) {
            match self {
                Buffer::S8(b) => {
                    b.remove(pos);
                }
                Buffer::S16(b) => {
                    b.remove(pos);
                }
                Buffer::S32(b) => {
                    b.remove(pos);
                }
                Buffer::Heap(b) => {
                    b.remove(pos);
                }
            }
            true
        } else {
            false
        }
    }

    pub fn set_sorted_unique(&mut self, src: impl IntoIterator<Item = T>)
    where
        T: Ord,
    {
        Self::set_sorted_unique_2(self, src);
    }

    #[inline]
    fn set_sorted_unique_2<I: IntoIterator<Item = T>>(dst: &mut Self, src: I)
    where
        T: Ord,
    {
        let mut v: Vec<T> = src.into_iter().collect();
        v.sort_unstable();
        v.dedup();

        // Rebuild self with the smallest fitting tier.
        let n = v.len();
        *dst = if n <= 8 {
            let mut s: SmallVec<[T; 8]> = SmallVec::with_capacity(n);
            s.extend(v.into_iter());
            Buffer::S8(s)
        } else if n <= 16 {
            let mut s: SmallVec<[T; 16]> = SmallVec::with_capacity(n);
            s.extend(v.into_iter());
            Buffer::S16(s)
        } else if n <= 32 {
            let mut s: SmallVec<[T; 32]> = SmallVec::with_capacity(n);
            s.extend(v.into_iter());
            Buffer::S32(s)
        } else {
            Buffer::Heap(v)
        };
    }

    /// Push without maintaining sorted order (rarely needed; useful for bulk load before a final sort/dedup).
    #[inline]
    pub fn push_unchecked(&mut self, value: T) {
        self.reserve_one();
        match self {
            Buffer::S8(b) => b.push(value),
            Buffer::S16(b) => b.push(value),
            Buffer::S32(b) => b.push(value),
            Buffer::Heap(b) => b.push(value),
        }
    }

    /// Consume into a Vec<T> (cheap for Heap).
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Buffer::S8(b) => b.into_vec(),
            Buffer::S16(b) => b.into_vec(),
            Buffer::S32(b) => b.into_vec(),
            Buffer::Heap(b) => b,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Buffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for Buffer<T> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(ser)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de> + Ord> Deserialize<'de> for Buffer<T> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let v = Vec::<T>::deserialize(de)?;
        let mut buf = Buffer::new();
        buf.set_sorted_unique(v);
        Ok(buf)
    }
}
