
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NeuronId(Uuid);

impl NeuronId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EdgeId(Uuid);

impl EdgeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
