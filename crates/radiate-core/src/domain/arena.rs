use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// A unique id to identify an arena member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeId(u64, usize);

impl NodeId {
    pub fn new(index: usize) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        NodeId(COUNTER.fetch_add(1, Ordering::Relaxed), index)
    }
}

pub struct Arena<T> {
    members: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            members: Vec::new(),
        }
    }

    pub fn add(&mut self, member: T) -> NodeId {
        let index = self.members.len();
        self.members.push(member);
        NodeId::new(index)
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.members.get(id.1)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.members.get_mut(id.1)
    }
}
