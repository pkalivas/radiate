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

    pub const REPLACE_AGE: &str = "Replace(Age)";
    pub const REPLACE_INVALID: &str = "Replace(Invalid)";

    pub const UNIQUE_SCORES: &str = "Unique(Scores)";
    pub const UNIQUE_INDIVIDUALS: &str = "Unique(Individuals)";

    pub const GENOME_SIZE: &str = "Genome Size";

    pub const FRONT: &str = "Pareto Front";

    pub const FITNESS: &str = "Fitness";
    pub const ALTERED: &str = "Altered";

    pub const SPECIES_COUNT: &str = "Species(Count)";
    pub const SPECIES_AGE_FAIL: &str = "Species(Age Removed)";
    pub const SPECIES_DISTANCE_DIST: &str = "Species(Distance)";
    pub const SPECIES_CREATED: &str = "Species(Created)";
}
