use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::Arc,
};

thread_local! {
    pub static STR_INTERN_CACHE: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
    pub static ARC_STRING_INTERN_CACHE: RefCell<HashMap<&'static str, Arc<String>>> = RefCell::new(HashMap::new());
    pub static SNAKE_CASE_INTERN_CACHE: RefCell<HashMap<&'static str, &'static str>> = RefCell::new(HashMap::new());
}

pub fn is_str_interned(s: &str) -> bool {
    STR_INTERN_CACHE.with(|interned| interned.borrow().contains(s))
}

pub fn is_arc_string_interned(s: &str) -> bool {
    ARC_STRING_INTERN_CACHE.with(|interned| interned.borrow().contains_key(s))
}

pub fn is_snake_case_interned(s: &str) -> bool {
    SNAKE_CASE_INTERN_CACHE.with(|interned| interned.borrow().contains_key(s))
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
macro_rules! cache_arc_string {
    ($name:expr) => {{
        use std::cell::RefCell;
        use std::collections::HashMap;
        use std::sync::Arc;

        $crate::ARC_STRING_INTERN_CACHE.with(|interned| {
            let mut interned = interned.borrow_mut();
            if let Some(existing) = interned.get(&*$name) {
                Arc::clone(existing)
            } else {
                let name_as_string = String::from($name);
                let static_name: &'static str = Box::leak(name_as_string.into_boxed_str());
                let result = Arc::new(String::from(static_name));
                interned.insert(static_name, Arc::clone(&result));
                result
            }
        })
    }};
}

#[macro_export]
macro_rules! intern_snake_case {
    ($name:expr) => {{
        if $crate::is_snake_case_interned($name) {
            $crate::SNAKE_CASE_INTERN_CACHE.with(|interned| {
                let interned = interned.borrow();
                *interned.get($name).unwrap()
            })
        } else {
            let name = intern!($name);
            let snake_case_name = intern!(name.to_snake_case());
            $crate::SNAKE_CASE_INTERN_CACHE.with(|interned| {
                let mut interned = interned.borrow_mut();
                if let Some(existing) = interned.get(&*name) {
                    existing
                } else {
                    interned.insert(name, snake_case_name);
                    snake_case_name
                }
            })
        }
    }};
}
