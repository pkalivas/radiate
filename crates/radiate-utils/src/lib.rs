mod arena;
mod fmt;
mod intern;
mod lru;
mod regex;
#[allow(dead_code)]
mod registry;
mod str;

pub use arena::{Arena, ArenaKey};
pub use fmt::{ToSnakeCase, intern_name_as_snake_case};
pub use lru::LruCache;
pub use regex::{RegexCache, compile_regex, with_regex_cache};
pub use str::SmallStr;

pub use intern::{
    ARC_STRING_INTERN_CACHE, SNAKE_CASE_INTERN_CACHE, STR_INTERN_CACHE, is_arc_string_interned,
    is_snake_case_interned, is_str_interned,
};
