#[macro_export]
macro_rules! intern {
    ($name:expr) => {{
        use std::cell::RefCell;
        use std::collections::HashSet;

        thread_local! {
            static INTERNED: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
        }

        let name = String::from($name);
        INTERNED.with(|interned| {
            let mut interned = interned.borrow_mut();
            if let Some(&existing) = interned.get(&*name) {
                existing
            } else {
                let static_name: &'static str = Box::leak(name.into_boxed_str());
                interned.insert(static_name);
                static_name
            }
        })
    }};
}

#[macro_export]
macro_rules! cache_string {
    ($name:expr) => {{
        use std::cell::RefCell;
        use std::collections::HashMap;
        use std::sync::Arc;

        thread_local! {
            static INTERNED: RefCell<HashMap<&'static str, Arc<String>>> = RefCell::new(HashMap::new());
        }

        let name_as_string = String::from($name);
        INTERNED.with(|interned| {
            let mut interned = interned.borrow_mut();
            if let Some(existing) = interned.get(&*name_as_string) {
                Arc::clone(existing)
            } else {
                let static_name: &'static str = Box::leak(name_as_string.into_boxed_str());
                let result = Arc::new(String::from(static_name));
                interned.insert(static_name, Arc::clone(&result));
                result
            }
        })
    }};
}
