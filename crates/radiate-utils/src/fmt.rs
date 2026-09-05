#[inline]
pub fn short_type_name<T: ?Sized>() -> String {
    let full = std::any::type_name::<T>();
    let mut out = String::with_capacity(full.len());
    let mut segment_start = 0;
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

pub fn generate_metric_key<T: ?Sized>(category: &str) -> String {
    let struct_type_name = std::any::type_name::<T>();
    let head = struct_type_name
        .split('<')
        .next()
        .unwrap_or(struct_type_name);
    let base = head.rsplit("::").next().unwrap_or(head).trim();

    let mut parts = words(base);
    parts.retain(|w| w != category);

    let mut result = vec![category.to_string()];
    result.extend(parts);
    result.join(".")
}

fn words(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut words = Vec::new();
    let mut current = String::new();

    for (i, &c) in chars.iter().enumerate() {
        if c == '_' || c == '-' || c.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        if c.is_uppercase() {
            let prev_lower_or_digit =
                i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let prev_upper = i > 0 && chars[i - 1].is_uppercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            let boundary = prev_lower_or_digit || (prev_upper && next_lower);
            if boundary && !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        }
        current.push(c.to_ascii_lowercase());
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

// /// Turn a type's name into a dot-separated metric key.
// ///
// /// `GraphMutator<Op<T>>`              -> "graph.mutator"
// /// `my_crate::FnMutator<FloatGene, F>` -> "fn.mutator"
// /// `HTTPServerMutator`                 -> "http.server.mutator"
// ///
// /// Steps: drop everything from the first `<` onward (generic params carry no
// /// naming information we want here), drop the module path (keep the last
// /// `::` segment), then split the remaining PascalCase identifier at case
// /// boundaries and lowercase+join with `.`.
// pub fn type_name_to_dot_notation<T: ?Sized>() -> String {
//     let type_name = std::any::type_name::<T>();
//     str_to_dot_notation(type_name)
// }

// pub fn str_to_dot_notation(s: &str) -> String {
//     let head = s.split('<').next().unwrap_or(s);
//     let base = head.rsplit("::").next().unwrap_or(head).trim();
//     words(base).join(".")
// }

// fn words(s: &str) -> Vec<String> {
//     let chars: Vec<char> = s.chars().collect();
//     let mut words = Vec::new();
//     let mut current = String::new();

//     for (i, &c) in chars.iter().enumerate() {
//         if c == '_' || c == '-' || c.is_whitespace() {
//             if !current.is_empty() {
//                 words.push(std::mem::take(&mut current));
//             }
//             continue;
//         }

//         if c.is_uppercase() {
//             let prev_lower_or_digit =
//                 i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
//             let prev_upper = i > 0 && chars[i - 1].is_uppercase();
//             let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

//             // New word if the previous char was lowercase/digit (normal
//             // PascalCase boundary), or if we're at the tail of an acronym
//             // run about to hand off to a new word ("HTTPServer" -> boundary
//             // right before "Server", not before every letter of "HTTP").
//             let boundary = prev_lower_or_digit || (prev_upper && next_lower);

//             if boundary && !current.is_empty() {
//                 words.push(std::mem::take(&mut current));
//             }
//         }

//         current.push(c.to_ascii_lowercase());
//     }

//     if !current.is_empty() {
//         words.push(current);
//     }

//     words
// }
