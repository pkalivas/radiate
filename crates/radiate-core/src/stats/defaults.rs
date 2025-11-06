use crate::{MetricScope as Scope, Rollup};

pub mod metric_names {
    pub const TIME: &str = "time";

    pub const SCORES: &str = "scores";
    pub const AGE: &str = "age";

    pub const REPLACE_AGE: &str = "replace_age";
    pub const REPLACE_INVALID: &str = "replace_invalid";

    pub const GENOME_SIZE: &str = "genome_size";
    pub const FRONT_ADDITIONS: &str = "front_additions";

    pub const UNIQUE_MEMBERS: &str = "unique_members";
    pub const UNIQUE_SCORES: &str = "unique_scores";
    pub const NEW_CHILDREN: &str = "new_children";

    pub const SURVIVOR_COUNT: &str = "survivor_count";
    pub const CARRYOVER_RATE: &str = "carryover_rate";
    pub const LIFETIME_UNIQUE_MEMBERS: &str = "lifetime_unique";

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

/// Lookup the default scope for a given metric name.
pub fn default_scope(name: &'static str) -> Scope {
    match name {
        metric_names::LIFETIME_UNIQUE_MEMBERS | metric_names::TIME => Scope::Lifetime,
        _ => Scope::Generation,
    }
}

/// Lookup the default rollup mode for a given metric name.
/// These decide how values are aggregated when flushing scopes.
pub fn default_rollup(name: &'static str) -> Rollup {
    match name {
        metric_names::AGE
        | metric_names::GENOME_SIZE
        | metric_names::CARRYOVER_RATE
        | metric_names::DIVERSITY_RATIO
        | metric_names::SURVIVOR_COUNT
        | metric_names::SCORE_VOLATILITY
        | metric_names::FRONT_ADDITIONS
        | metric_names::SPECIES_AGE
        | metric_names::SPECIES_COUNT => Rollup::Mean,

        metric_names::UNIQUE_MEMBERS | metric_names::UNIQUE_SCORES => Rollup::Mean,

        metric_names::NEW_CHILDREN
        | metric_names::SPECIES_CREATED
        | metric_names::SPECIES_DIED
        | metric_names::EVALUATION_COUNT
        | metric_names::LIFETIME_UNIQUE_MEMBERS => Rollup::Sum,

        metric_names::SPECIES_DISTANCE_DIST | metric_names::SCORES => Rollup::Sum,
        metric_names::TIME => Rollup::Last,

        _ => Rollup::Sum,
    }
}
