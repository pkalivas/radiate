use crate::stats::{Tag, TagKind};

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
    pub const FRONT_FILTERS: &str = "front_filters";

    pub const UNIQUE_MEMBERS: &str = "unique_members";
    pub const UNIQUE_SCORES: &str = "unique_scores";
    pub const NEW_CHILDREN: &str = "new_children";

    pub const SURVIVOR_COUNT: &str = "survivor_count";
    pub const CARRYOVER_RATE: &str = "carryover_rate";

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

    pub const SCORE: &str = "score";
}

const RULES: &[(&str, &[TagKind])] = &[
    (metric_tags::SELECTOR, &[TagKind::Selector]),
    (metric_tags::MUTATOR, &[TagKind::Alterer, TagKind::Mutator]),
    (
        metric_tags::CROSSOVER,
        &[TagKind::Alterer, TagKind::Crossover],
    ),
    (metric_tags::ALTERER, &[TagKind::Alterer]),
    (metric_tags::SPECIES, &[TagKind::Species]),
    (metric_tags::FAILURE, &[TagKind::Failure]),
    (metric_tags::AGE, &[TagKind::Age]),
    (metric_tags::FRONT, &[TagKind::Front]),
    (metric_tags::DERIVED, &[TagKind::Derived]),
    (metric_tags::OTHER, &[TagKind::Other]),
    (metric_tags::SCORE, &[TagKind::Score]),
];

pub fn default_tags(name: &str) -> Tag {
    let mut mask = Tag::empty();

    // Exact-name mappings first
    match name {
        metric_names::REPLACE_AGE => {
            mask.insert(TagKind::Age);
            mask.insert(TagKind::Failure);
        }
        metric_names::REPLACE_INVALID => {
            mask.insert(TagKind::Failure);
        }
        metric_names::FRONT_ADDITIONS
        | metric_names::FRONT_REMOVALS
        | metric_names::FRONT_COMPARISONS
        | metric_names::FRONT_ENTROPY
        | metric_names::FRONT_FILTERS
        | metric_names::FRONT_SIZE => {
            mask.insert(TagKind::Front);
        }
        metric_names::SPECIES_AGE_FAIL => {
            mask.insert(TagKind::Species);
            mask.insert(TagKind::Age);
            mask.insert(TagKind::Failure);
        }
        metric_names::SPECIES_AGE => {
            mask.insert(TagKind::Species);
            mask.insert(TagKind::Age);
        }

        // “Derived” metrics
        metric_names::UNIQUE_MEMBERS
        | metric_names::UNIQUE_SCORES
        | metric_names::NEW_CHILDREN
        | metric_names::SURVIVOR_COUNT
        | metric_names::CARRYOVER_RATE
        | metric_names::DIVERSITY_RATIO
        | metric_names::SCORE_VOLATILITY => {
            mask.insert(TagKind::Derived);
        }

        metric_names::SCORES => {
            mask.insert(TagKind::Score);
        }

        _ => {}
    }

    mask
}

pub fn try_add_tag_from_str(metric: &mut crate::stats::Metric) {
    let s = metric.name();
    let mut tags = Tag::empty();

    for (needle, kinds) in RULES {
        if s.contains(needle) {
            for &k in *kinds {
                tags.insert(k);
            }
        }
    }

    if !tags.is_empty() {
        metric.with_tags(tags);
    }
}
