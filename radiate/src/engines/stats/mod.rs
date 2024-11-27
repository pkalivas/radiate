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
    pub const AGE_FILTER: &str = "Age Filter";
    pub const INVALID_FILTER: &str = "Invalid Filter";
    pub const UNIQUE: &str = "Unique";
    pub const GENOME_SIZE: &str = "Genome Size";
    pub const FRONT: &str = "Front";
}
