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
    pub const EVOLUTION_TIME: &str = "time";
    pub const SCORE: &str = "score";
    pub const AGE: &str = "age";

    pub const REPLACE_AGE: &str = "replace_age";
    pub const REPLACE_INVALID: &str = "replace_invalid";

    pub const GENOME_SIZE: &str = "genome_size";
    pub const FRONT: &str = "front";

    pub const UNIQUE_MEMBERS: &str = "unique_members";
    pub const UNIQUE_SCORES: &str = "unique_scores";

    pub const FITNESS: &str = "fitness";

    pub const SPECIES_COUNT: &str = "species_count";
    pub const SPECIES_AGE_FAIL: &str = "species_age_fail";
    pub const SPECIES_DISTANCE_DIST: &str = "species_distance_dist";
    pub const SPECIES_CREATED: &str = "species_created";
    pub const SPECIES_DIED: &str = "species_died";
    pub const SPECIES_AGE: &str = "species_age";
}
