use std::{any::TypeId, collections::HashMap};

/// A simple type-based registry for storing and retrieving values of different types.
/// I'm not sure if we'll use this yet...but it could prove useful (Python possibly?)
#[derive(Default)]
pub struct Regristry {
    records: HashMap<TypeId, Box<dyn std::any::Any>>,
}

impl Regristry {
    pub fn insert<T: 'static>(&mut self, value: T) {
        self.records.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.records
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.records
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.records
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
}

#[cfg(test)]
mod tests {
    use super::Regristry;

    #[test]
    fn test_registry_insert_and_get() {
        let mut registry = Regristry::default();

        registry.insert(42u32);
        registry.insert("hello".to_string());

        let value_u32: &u32 = registry.get::<u32>().unwrap();
        let value_string: &String = registry.get::<String>().unwrap();

        assert_eq!(*value_u32, 42);
        assert_eq!(value_string, "hello");
    }

    #[test]
    fn test_registry_get_mut() {
        let mut registry = Regristry::default();

        registry.insert(10_i32);

        {
            let value_i32: &mut i32 = registry.get_mut::<i32>().unwrap();
            *value_i32 += 5;
        }

        let value_i32: &i32 = registry.get::<i32>().unwrap();
        assert_eq!(*value_i32, 15);
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = Regristry::default();
        registry.insert(3.14_f64);
        let removed_value: f64 = registry.remove::<f64>().unwrap();
        assert_eq!(removed_value, 3.14);
        assert!(registry.get::<f64>().is_none());
    }
}
