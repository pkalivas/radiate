use std::any::TypeId;

use foldhash::HashMap;

pub struct Registry {
    entries: HashMap<TypeId, Box<dyn std::any::Any>>,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            entries: HashMap::default(),
        }
    }

    pub fn insert<T: 'static>(&mut self, value: T) {
        self.entries.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.entries
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.entries
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
}
