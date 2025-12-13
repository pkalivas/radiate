use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct ArenaKey(usize);

impl AsRef<usize> for ArenaKey {
    fn as_ref(&self) -> &usize {
        &self.0
    }
}

pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, item: T) -> ArenaKey {
        self.items.push(item);
        ArenaKey(self.items.len() - 1)
    }

    pub fn get(&self, index: &ArenaKey) -> Option<&T> {
        self.items.get(*index.as_ref())
    }

    pub fn get_mut(&mut self, index: &ArenaKey) -> Option<&mut T> {
        self.items.get_mut(*index.as_ref())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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
    use super::Arena;

    #[test]
    fn test_arena_insert_and_get() {
        let mut arena = Arena::new();
        let key1 = arena.insert(10);
        let key2 = arena.insert(20);

        assert_eq!(arena.get(&key1), Some(&10));
        assert_eq!(arena.get(&key2), Some(&20));
        assert_eq!(arena.len(), 2);
    }
}
