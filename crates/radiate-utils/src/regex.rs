use crate::LruCache;
use regex::{Regex, RegexBuilder};
use std::cell::RefCell;

#[macro_export]
macro_rules! cached_regex {
    () => {};
    ($vis:vis static $name:ident = $regex:expr; $($rest:tt)*) => {
        #[allow(clippy::disallowed_methods)]
        $vis static $name: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| regex::Regex::new($regex).unwrap());
        $crate::cached_regex!($($rest)*);
    };
}

thread_local! {
    static LOCAL_REGEX_CACHE: RefCell<RegexCache> = RefCell::new(RegexCache::new());
}

pub fn compile_regex(re: &str) -> Result<Regex, regex::Error> {
    LOCAL_REGEX_CACHE.with_borrow_mut(|cache| cache.compile(re).cloned())
}

pub fn with_regex_cache<R, F: FnOnce(&mut RegexCache) -> R>(f: F) -> R {
    LOCAL_REGEX_CACHE.with_borrow_mut(f)
}

fn get_size_limit() -> Option<usize> {
    Some(
        std::env::var("RADIATE_REGEX_SIZE_LIMIT")
            .ok()
            .filter(|l| !l.is_empty())?
            .parse()
            .expect("invalid RADIATE_REGEX_SIZE_LIMIT"),
    )
}

/// A cache for compiled regular expressions.
pub struct RegexCache {
    cache: LruCache<String, Regex>,
    size_limit: Option<usize>,
}

impl RegexCache {
    fn new() -> Self {
        Self {
            cache: LruCache::with_capacity(32),
            size_limit: get_size_limit(),
        }
    }

    pub fn compile(&mut self, re: &str) -> Result<&Regex, regex::Error> {
        let r = self.cache.try_get_or_insert_with(re, |re| {
            // We do this little loop to only check RADIATE_REGEX_SIZE_LIMIT when
            // a regex fails to compile due to the size limit.
            loop {
                let mut builder = RegexBuilder::new(re);
                if let Some(bytes) = self.size_limit {
                    builder.size_limit(bytes);
                }
                match builder.build() {
                    err @ Err(regex::Error::CompiledTooBig(_)) => {
                        let new_size_limit = get_size_limit();
                        if new_size_limit != self.size_limit {
                            self.size_limit = new_size_limit;
                            continue; // Try to compile again.
                        }
                        break err;
                    }
                    r => break r,
                };
            }
        });
        Ok(&*r?)
    }
}

#[cfg(test)]
mod tests {
    use super::RegexCache;

    #[test]
    fn caches_regexes() {
        let mut cache = RegexCache::new();

        let r1 = cache.compile(r"^\d+$").unwrap();
        assert!(r1.is_match("123"));
        let r1_ptr = r1 as *const _;

        // Should hit cache
        let r2 = cache.compile(r"^\d+$").unwrap();
        assert!(std::ptr::eq(r1_ptr, r2 as *const _));

        let r3 = cache.compile(r"^[a-z]+$").unwrap();
        assert!(r3.is_match("abc"));
        assert!(!r3.is_match("123"));
    }
}
