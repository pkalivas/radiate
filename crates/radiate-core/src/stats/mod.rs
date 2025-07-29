pub mod distribution;
pub mod histogram;
pub mod metrics;
pub mod query;
pub mod statistics;
pub mod time_statistic;

pub use distribution::*;
pub use histogram::*;
pub use metric_names::*;
pub use metrics::*;
pub use query::*;
pub use statistics::*;
pub use time_statistic::*;

pub mod metric_names {
    pub const TIME: &str = "time";

    pub const SCORE_IMPROVEMENT_RATE: &str = "score_improv_rate";
    pub const SCORES: &str = "scores";
    pub const AGE: &str = "age";

    pub const REPLACE_AGE: &str = "replace_age";
    pub const REPLACE_INVALID: &str = "replace_invalid";

    pub const GENOME_SIZE: &str = "genome_size";
    pub const FRONT_ADDITIONS: &str = "front_additions";

    pub const UNIQUE_MEMBERS: &str = "unique_members";
    pub const UNIQUE_SCORES: &str = "unique_scores";

    pub const EVALUATION_COUNT: &str = "evaluation_count";

    pub const DIVERSITY_RATIO: &str = "diversity_ratio";
    pub const SCORE_VOLATILITY: &str = "score_volatility";

    pub const SPECIES_COUNT: &str = "species_count";
    pub const SPECIES_AGE_FAIL: &str = "species_age_fail";
    pub const SPECIES_DISTANCE_DIST: &str = "species_distance_dist";
    pub const SPECIES_CREATED: &str = "species_created";
    pub const SPECIES_DIED: &str = "species_died";
    pub const SPECIES_AGE: &str = "species_age";
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
