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
    pub const EVOLUTION_TIME: &str = "Time";
    pub const SCORE: &str = "Score";
    pub const AGE: &str = "Age";

    pub const REPLACE_AGE: &str = "Replace(Age)";
    pub const REPLACE_INVALID: &str = "Replace(Invalid)";

    pub const GENOME_SIZE: &str = "Genome Size";
    pub const FRONT: &str = "Front";

    pub const UNIQUE_MEMBERS: &str = "Unique(members)";
    pub const UNIQUE_SCORES: &str = "Unique(scores)";

    pub const FITNESS: &str = "Fitness";

    pub const SPECIES_COUNT: &str = "Species(Count)";
    pub const SPECIES_AGE_FAIL: &str = "Species(Age Removed)";
    pub const SPECIES_DISTANCE_DIST: &str = "Species(Distance)";
    pub const SPECIES_CREATED: &str = "Species(Created)";
    pub const SPECIES_DIED: &str = "Species(Died)";
    pub const SPECIES_AGE: &str = "Species(Age)";
}
