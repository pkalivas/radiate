
use std::fmt;

#[cfg(feature = "tiny-ids")]
mod index_type {
    pub type NeuronIndex = u8;
    pub type EdgeIndex = u16;
}

#[cfg(feature = "small-ids")]
mod index_type {
    pub type NeuronIndex = u16;
    pub type EdgeIndex = u32;
}

#[cfg(not(any(feature = "tiny-ids", feature = "small-ids")))]
mod index_type {
    pub type NeuronIndex = u32;
    pub type EdgeIndex = u32;
}

use index_type::*;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NeuronId(NeuronIndex);

impl NeuronId {
    pub const MIN: usize = 0;
    pub const MAX: usize = NeuronIndex::MAX as usize;

    pub fn new(index: usize) -> Self {
        if index > Self::MAX as usize {
            panic!("NeuronId too small, layer has more then {} neurons", Self::MAX);
        }
        Self(index as NeuronIndex)
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for NeuronId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EdgeId(EdgeIndex);

impl EdgeId {
    pub const MIN: usize = 0;
    pub const MAX: usize = EdgeIndex::MAX as usize;

    pub fn new(index: usize) -> Self {
        if index > Self::MAX as usize {
            panic!("EdgeId too small, layer has more then {} edges", Self::MAX);
        }
        Self(index as EdgeIndex)
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
