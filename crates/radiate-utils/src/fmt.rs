use crate::intern;

#[inline]
pub fn intern_name_as_snake_case(name: &'static str) -> &'static str {
    crate::intern_snake_case!(name)
}

#[inline]
pub fn intern_kv_pair(name: &'static str, value: &'static str) -> &'static str {
    crate::intern_str_cache!(name, value)
}

#[inline]
pub fn short_type_name<T: ?Sized>() -> String {
    // Walk the full type name and strip the module path of every segment, not
    // just the head. Segments are delimited by `<`, `>`, `,`, or space; each
    // segment becomes whatever follows its last `::`. So
    // `module::Foo<other::Bar<u8>>` renders as `Foo<Bar<u8>>`.
    let full = std::any::type_name::<T>();
    let mut out = String::with_capacity(full.len());
    let mut segment_start = 0usize;
    for (i, c) in full.char_indices() {
        if matches!(c, '<' | '>' | ',' | ' ') {
            out.push_str(strip_path(&full[segment_start..i]));
            out.push(c);
            segment_start = i + c.len_utf8();
        }
    }

    if segment_start < full.len() {
        out.push_str(strip_path(&full[segment_start..]));
    }

    out
}

fn strip_path(segment: &str) -> &str {
    match segment.rfind("::") {
        Some(idx) => &segment[idx + 2..],
        None => segment,
    }
}

pub trait ToSnakeCase<O> {
    fn to_snake_case(&self) -> O;
}

impl ToSnakeCase<String> for &'_ str {
    fn to_snake_case(&self) -> String {
        if self
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
        {
            return self.to_string();
        }

        let mut snake_case = String::new();

        for (i, c) in self.chars().enumerate() {
            if c.is_uppercase() {
                if i != 0 {
                    snake_case.push('_');
                }
                for lower_c in c.to_lowercase() {
                    snake_case.push(lower_c);
                }
            } else {
                snake_case.push(c);
            }
        }
        snake_case
    }
}

impl ToSnakeCase<String> for String {
    fn to_snake_case(&self) -> String {
        if self
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
        {
            return self.to_string();
        }

        let mut snake_case = String::new();

        for (i, c) in self.chars().enumerate() {
            if c.is_uppercase() {
                if i != 0 {
                    snake_case.push('_');
                }
                for lower_c in c.to_lowercase() {
                    snake_case.push(lower_c);
                }
            } else {
                snake_case.push(c);
            }
        }
        snake_case
    }
}
