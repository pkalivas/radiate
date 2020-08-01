
use std::fmt;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NeuronId(u16);

impl NeuronId {
    pub fn new(id: usize) -> Self {
        Self(id as u16)
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
pub struct EdgeId(u16);

impl EdgeId {
    pub fn new(id: usize) -> Self {
        Self(id as u16)
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
