#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VarSpec {
    pub id: VarId,
    pub card: usize,
}

impl VarSpec {
    pub fn new(id: u32, card: usize) -> Self {
        Self {
            id: VarId(id),
            card,
        }
    }
}

impl From<usize> for VarId {
    fn from(v: usize) -> Self {
        VarId(v as u32)
    }
}

impl From<u32> for VarId {
    fn from(v: u32) -> Self {
        VarId(v)
    }
}
