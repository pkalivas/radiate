use crate::sentry_id;

sentry_id!(ArenaSlotId);

pub struct ArenaBuffer<T> {
    buffer: Vec<T>,
}

impl<T> ArenaBuffer<T> {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn add(&mut self, item: T) -> ArenaSlotId {
        let id = ArenaSlotId::new();
        self.buffer.push(item);
        id
    }

    pub fn get(&self, id: ArenaSlotId) -> Option<&T> {
        self.buffer.get(id.0 as usize)
    }
}
