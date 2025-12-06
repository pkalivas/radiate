use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct ArenaKey(usize);

impl AsRef<ArenaKey> for ArenaKey {
    fn as_ref(&self) -> &ArenaKey {
        self
    }
}

pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn insert(&mut self, item: T) -> ArenaKey {
        self.items.push(item);
        ArenaKey(self.items.len() - 1)
    }

    pub fn get(&self, index: impl AsRef<ArenaKey>) -> Option<&T> {
        self.items.get(index.as_ref().0)
    }

    pub fn get_mut(&mut self, index: impl AsRef<ArenaKey>) -> Option<&mut T> {
        self.items.get_mut(index.as_ref().0)
    }
}

impl<T> std::ops::Index<usize> for Arena<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

#[cfg(test)]
mod tests {
    use super::{Arena, ArenaKey};

    #[test]
    fn test_arena_insert_and_get() {
        let mut arena = Arena::new();
        let key1 = arena.insert(10);
        let key2 = arena.insert(20);

        assert_eq!(arena.get(key1), Some(&10));
        assert_eq!(arena.get(key2), Some(&20));
        assert_eq!(arena.get(&ArenaKey(2)), None);
    }
}
