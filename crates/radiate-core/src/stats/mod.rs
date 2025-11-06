mod defaults;
mod distribution;
mod fmt;
mod metrics;
mod set;
mod statistics;
mod time_statistic;

pub use defaults::metric_names;
pub use distribution::*;
pub use fmt::{render_dashboard, render_full, render_metric_rows};
pub use metrics::*;
pub use set::MetricSet;
pub use statistics::*;
pub use time_statistic::*;

#[inline]
fn normalize_name(name: &'static str) -> &'static str {
    let is_snake = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');

    if is_snake {
        return name;
    }

    crate::intern!(name.to_snake_case())
}

pub trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl ToSnakeCase for &'_ str {
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

impl ToSnakeCase for String {
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
