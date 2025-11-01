#[macro_export]
macro_rules! intern {
    ($name:expr) => {{
        use std::collections::HashSet;
        use std::sync::{Mutex, OnceLock};

        static INTERNED: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();

        let name = String::from($name);
        let mut interned = INTERNED
            .get_or_init(|| Mutex::new(HashSet::new()))
            .lock()
            .unwrap();
        if let Some(&existing) = interned.get(&*name) {
            existing
        } else {
            let static_name: &'static str = Box::leak(name.into_boxed_str());
            interned.insert(static_name);
            static_name
        }
    }};
}
