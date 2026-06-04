use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

thread_local! {
    pub static STR_INTERN_CACHE: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
    pub static STR_CACHE: RefCell<HashMap<&'static str, &'static str>> = RefCell::new(HashMap::new());

}

pub fn is_str_interned(s: &str) -> bool {
    STR_INTERN_CACHE.with(|interned| interned.borrow().contains(s))
}

pub fn is_str_cached(s: &str) -> bool {
    STR_CACHE.with(|cache| cache.borrow().contains_key(s))
}

pub fn try_get_interned_str(s: &str) -> Option<&'static str> {
    STR_CACHE.with(|interned| interned.borrow().get(s).cloned())
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

#[macro_export]
macro_rules! intern_str_cache {
    ($name:expr, $value:expr) => {{
        if $crate::is_str_cached($name) {
            $crate::STR_CACHE.with(|cache| {
                let cache = cache.borrow();
                *cache.get($name).unwrap()
            })
        } else {
            let intered_name = intern!($name);
            let intered_value = intern!($value);
            $crate::STR_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                if let Some(existing) = cache.get(&*intered_name) {
                    existing
                } else {
                    cache.insert(intered_name, intered_value);
                    intered_value
                }
            })
        }
    }};
}
