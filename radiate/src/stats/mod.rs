pub mod distribution;
pub mod metrics;
pub mod statistics;
pub mod time_statistic;

pub use distribution::*;
pub use metric_names::*;
pub use metrics::*;
pub use statistics::*;
pub use time_statistic::*;

pub mod metric_names {
    pub const SCORE: &str = "Score";
    pub const AGE: &str = "Age";
    pub const EVALUATION: &str = "Evaluation";
    pub const FILTER_AGE: &str = "Filter(Age)";
    pub const FILTER_INVALID: &str = "Filter(Invalid)";
    pub const UNIQUE_SCORES: &str = "Unique(scores)";
    pub const GENOME_SIZE: &str = "Genome Size";
    pub const FRONT: &str = "Front";
    pub const UNIQUE_MEMBERS: &str = "Unique(members)";
}
