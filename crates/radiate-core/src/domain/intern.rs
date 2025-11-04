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
