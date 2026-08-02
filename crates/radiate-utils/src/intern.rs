use std::{cell::RefCell, collections::HashSet};

thread_local! {
    pub static STR_INTERN_CACHE: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());

}

pub fn is_str_interned(s: &str) -> bool {
    STR_INTERN_CACHE.with(|interned| interned.borrow().contains(s))
}

#[macro_export]
macro_rules! intern {
    ($name:expr) => {{
        $crate::STR_INTERN_CACHE.with(|interned| {
            let mut interned = interned.borrow_mut();
            if let Some(&existing) = interned.get(&*$name) {
                existing
            } else {
                let name = String::from($name);
                let static_name: &'static str = Box::leak(name.into_boxed_str());
                interned.insert(static_name);
                static_name
            }
        })
    }};
}
