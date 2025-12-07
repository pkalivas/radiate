use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use crate::Rollup;
use crate::stats::Tag;

pub mod metric_names {
    pub const TIME: &str = "time";

    pub const SCORES: &str = "scores";
    pub const AGE: &str = "age";

    pub const REPLACE_AGE: &str = "replace_age";
    pub const REPLACE_INVALID: &str = "replace_invalid";

    pub const GENOME_SIZE: &str = "genome_size";

    pub const FRONT_ADDITIONS: &str = "front_additions";
    pub const FRONT_ENTROPY: &str = "front_entropy";
    pub const FRONT_REMOVALS: &str = "front_removals";
    pub const FRONT_COMPARISONS: &str = "front_comparisons";
    pub const FRONT_SIZE: &str = "front_size";

    pub const UNIQUE_MEMBERS: &str = "unique_members";
    pub const UNIQUE_SCORES: &str = "unique_scores";
    pub const NEW_CHILDREN: &str = "new_children";

    pub const SURVIVOR_COUNT: &str = "survivor_count";
    pub const CARRYOVER_RATE: &str = "carryover_rate";
    pub const LIFETIME_UNIQUE_MEMBERS: &str = "lifetime_unique";

    pub const EVALUATION_COUNT: &str = "evaluation_count";

    pub const DIVERSITY_RATIO: &str = "diversity_ratio";
    pub const SCORE_VOLATILITY: &str = "score_volatility";

    pub const BEST_SCORE_IMPROVEMENT: &str = "best_score_improvement";

    pub const SPECIES_COUNT: &str = "species_count";
    pub const SPECIES_AGE_FAIL: &str = "species_age_fail";
    pub const SPECIES_DISTANCE_DIST: &str = "species_distance_dist";
    pub const SPECIES_CREATED: &str = "species_created";
    pub const SPECIES_DIED: &str = "species_died";
    pub const SPECIES_AGE: &str = "species_age";
    pub const SPECIES_SIZE: &str = "species_size";
    pub const SPECIES_EVENNESS: &str = "species_evenness";
    pub const LARGEST_SPECIES_SHARE: &str = "largest_species_share";
    pub const SPECIES_NEW_RATIO: &str = "species_new_ratio";
}

pub mod metric_tags {

    pub const SELECTOR: &str = "selector";

    pub const ALTERER: &str = "alterer";
    pub const MUTATOR: &str = "mutator";
    pub const CROSSOVER: &str = "crossover";

    pub const SPECIES: &str = "species";
    pub const FAILURE: &str = "failure";

    pub const AGE: &str = "age";

    pub const FRONT: &str = "front";

    pub const DERIVED: &str = "derived";

    pub const OTHER: &str = "other";

    pub const STATISTIC: &str = "statistic";
    pub const TIME: &str = "time";
    pub const DISTRIBUTION: &str = "distribution";
}

pub fn default_tags<'a>(name: &str) -> Option<Arc<Vec<Tag>>> {
    match name {
        n if DEFAULT_TAG_CACHE.contains_key(n) => DEFAULT_TAG_CACHE.get(n).cloned(),

        _ if name.contains("selector") => DEFAULT_TAG_CACHE.get(metric_tags::SELECTOR).cloned(),
        _ if name.contains("mutator") => DEFAULT_TAG_CACHE.get(metric_tags::MUTATOR).cloned(),
        _ if name.contains("crossover") => DEFAULT_TAG_CACHE.get(metric_tags::CROSSOVER).cloned(),
        _ if name.contains("species") => DEFAULT_TAG_CACHE.get(metric_tags::SPECIES).cloned(),
        _ if name.contains("failure") => DEFAULT_TAG_CACHE.get(metric_tags::FAILURE).cloned(),
        _ if name.contains("age") => DEFAULT_TAG_CACHE.get(metric_tags::AGE).cloned(),
        _ if name.contains("front") => DEFAULT_TAG_CACHE.get(metric_tags::FRONT).cloned(),

        _ => DEFAULT_TAG_CACHE.get(metric_tags::OTHER).cloned(),
    }
}

/// Lookup the default rollup mode for a given metric name.
/// These decide how values are aggregated when flushing scopes.
pub fn default_rollup(name: &str) -> Rollup {
    match name {
        metric_names::AGE
        | metric_names::GENOME_SIZE
        | metric_names::CARRYOVER_RATE
        | metric_names::DIVERSITY_RATIO
        | metric_names::SURVIVOR_COUNT
        | metric_names::SCORE_VOLATILITY
        | metric_names::FRONT_ADDITIONS
        | metric_names::FRONT_REMOVALS
        | metric_names::FRONT_COMPARISONS
        | metric_names::FRONT_ENTROPY
        | metric_names::UNIQUE_MEMBERS
        | metric_names::UNIQUE_SCORES
        | metric_names::SPECIES_AGE
        | metric_names::SPECIES_SIZE
        | metric_names::SPECIES_COUNT => Rollup::Mean,

        metric_names::TIME => Rollup::Last,

        _ => Rollup::Sum,
    }
}

pub static DEFAULT_TAG_CACHE: LazyLock<HashMap<&'static str, Arc<Vec<Tag>>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            metric_names::REPLACE_AGE,
            Arc::new(vec![
                Tag::from(metric_tags::AGE),
                Tag::from(metric_tags::FAILURE),
            ]),
        );

        m.insert(
            metric_names::REPLACE_INVALID,
            Arc::new(vec![Tag::from(metric_tags::FAILURE)]),
        );

        m.insert(
            metric_names::FRONT_ADDITIONS,
            Arc::new(vec![Tag::from(metric_tags::FRONT)]),
        );
        m.insert(
            metric_names::FRONT_ENTROPY,
            Arc::new(vec![Tag::from(metric_tags::FRONT)]),
        );
        m.insert(
            metric_names::SPECIES_AGE_FAIL,
            Arc::new(vec![
                Tag::from(metric_tags::SPECIES),
                Tag::from(metric_tags::AGE),
                Tag::from(metric_tags::FAILURE),
            ]),
        );

        m.insert(
            metric_names::SPECIES_AGE,
            Arc::new(vec![
                Tag::from(metric_tags::SPECIES),
                Tag::from(metric_tags::AGE),
            ]),
        );

        m.insert(
            metric_tags::SPECIES,
            Arc::new(vec![Tag::from(metric_tags::SPECIES)]),
        );

        m.insert(
            metric_tags::MUTATOR,
            Arc::new(vec![
                Tag::from(metric_tags::MUTATOR),
                Tag::from(metric_tags::ALTERER),
            ]),
        );

        m.insert(
            metric_tags::CROSSOVER,
            Arc::new(vec![
                Tag::from(metric_tags::CROSSOVER),
                Tag::from(metric_tags::ALTERER),
            ]),
        );

        m.insert(
            metric_tags::SELECTOR,
            Arc::new(vec![Tag::from(metric_tags::SELECTOR)]),
        );

        m.insert(
            metric_tags::FAILURE,
            Arc::new(vec![Tag::from(metric_tags::FAILURE)]),
        );

        m.insert(
            metric_tags::AGE,
            Arc::new(vec![Tag::from(metric_tags::AGE)]),
        );

        m.insert(
            metric_tags::OTHER,
            Arc::new(vec![Tag::from(metric_tags::OTHER)]),
        );

        m.insert(
            metric_tags::DISTRIBUTION,
            Arc::new(vec![Tag::from(metric_tags::DISTRIBUTION)]),
        );

        for name in [
            metric_names::UNIQUE_MEMBERS,
            metric_names::UNIQUE_SCORES,
            metric_names::NEW_CHILDREN,
            metric_names::SURVIVOR_COUNT,
            metric_names::CARRYOVER_RATE,
            metric_names::LIFETIME_UNIQUE_MEMBERS,
            metric_names::DIVERSITY_RATIO,
            metric_names::SCORE_VOLATILITY,
        ] {
            m.insert(name, Arc::new(vec![Tag::from(metric_tags::DERIVED)]));
        }

        for name in [metric_names::SCORES, metric_names::SPECIES_DISTANCE_DIST] {
            m.insert(name, Arc::new(vec![Tag::from(metric_tags::DISTRIBUTION)]));
        }

        for name in [
            metric_names::FRONT_ADDITIONS,
            metric_names::FRONT_REMOVALS,
            metric_names::FRONT_COMPARISONS,
            metric_names::FRONT_ENTROPY,
            metric_names::FRONT_SIZE,
        ] {
            m.insert(name, Arc::new(vec![Tag::from(metric_tags::FRONT)]));
        }

        m
    });
