use crate::intern;

#[inline]
pub fn intern_name_as_snake_case(name: &'static str) -> &'static str {
    crate::intern_snake_case!(name)
}

pub trait ToSnakeCase<O> {
    fn to_snake_case(&self) -> O;
}

impl ToSnakeCase<String> for &'_ str {
    fn to_snake_case(&self) -> String {
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
