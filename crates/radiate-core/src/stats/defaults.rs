use crate::stats::{Tag, TagType};
use radiate_utils::SmallStr;

pub mod metric_names {
    use radiate_utils::SmallStr;

    pub const TIME: SmallStr = SmallStr::from_static("time");

    pub const AGE: SmallStr = SmallStr::from_static("age");
    pub const REPLACE_AGE: SmallStr = SmallStr::from_static("age.replace");
    pub const SPECIES_AGE_FAIL: SmallStr = SmallStr::from_static("age.species.fail");
    pub const SPECIES_AGE: SmallStr = SmallStr::from_static("age.species");

    pub const SPECIES_NEW_RATIO: SmallStr = SmallStr::from_static("new.species.ratio");
    pub const FRONT_ADDITIONS: SmallStr = SmallStr::from_static("new.front");
    pub const SPECIES_CREATED: SmallStr = SmallStr::from_static("new.species");

    pub const REPLACE_INVALID: SmallStr = SmallStr::from_static("invalid.replace");
    pub const FRONT_REMOVALS: SmallStr = SmallStr::from_static("invalid.front");
    pub const SPECIES_DIED: SmallStr = SmallStr::from_static("invalid.species");

    pub const GENOME_SIZE: SmallStr = SmallStr::from_static("size.genome");
    pub const FRONT_SIZE: SmallStr = SmallStr::from_static("size.front");
    pub const SPECIES_SIZE: SmallStr = SmallStr::from_static("size.species");

    pub const FRONT_ENTROPY: SmallStr = SmallStr::from_static("front.entropy");
    pub const FRONT_COMPARISONS: SmallStr = SmallStr::from_static("front.comparisons");
    pub const FRONT_FILTERS: SmallStr = SmallStr::from_static("front.filters");

    pub const SURVIVOR_COUNT: SmallStr = SmallStr::from_static("count.survivor");
    pub const EVALUATION_COUNT: SmallStr = SmallStr::from_static("count.evaluation");
    pub const SPECIES_COUNT: SmallStr = SmallStr::from_static("count.species");

    pub const CARRYOVER_RATE: SmallStr = SmallStr::from_static("rate.carryover");
    pub const DIVERSITY_RATIO: SmallStr = SmallStr::from_static("rate.diversity");
    pub const LINEAGE_PARENTS_USED_RATIO: SmallStr =
        SmallStr::from_static("rate.lineage.parents_used");

    pub const SCORES: SmallStr = SmallStr::from_static("scores");
    pub const BEST_SCORES: SmallStr = SmallStr::from_static("scores.best");
    pub const SCORE_VOLATILITY: SmallStr = SmallStr::from_static("score.volatility");
    pub const BEST_SCORE_IMPROVEMENT: SmallStr = SmallStr::from_static("score.improvement");

    pub const INDEX: SmallStr = SmallStr::from_static("index");

    pub const SPECIES_DISTANCE_DIST: SmallStr = SmallStr::from_static("species.distance");
    pub const SPECIES_EVENNESS: SmallStr = SmallStr::from_static("species.evenness");
    pub const LARGEST_SPECIES_SHARE: SmallStr = SmallStr::from_static("species.largest_share");
    pub const SPECIES_THRESHOLD: SmallStr = SmallStr::from_static("species.threshold");

    pub const ALTER_PARENT_REUSE: SmallStr = SmallStr::from_static("alter.parent_reuse");
    pub const ALTER_WITHIN_FAMILY: SmallStr = SmallStr::from_static("alter.within_family");
    pub const ALTER_CROSS_FAMILY: SmallStr = SmallStr::from_static("alter.cross_family");

    pub const LINEAGE_EVENTS: SmallStr = SmallStr::from_static("lineage.events");
    pub const LINEAGE_PARENTS_USED_UNIQUE: SmallStr =
        SmallStr::from_static("lineage.parents_unique");
    // •	FAMILY_PAIR_ENTROPY in [0, 1]
    // •	0.0 = basically always the same pair (pairing collapse)
    // •	1.0 = pairings evenly distributed across the pairs that occurred
    pub const LINEAGE_FAMILY_PAIR_ENTROPY: SmallStr = SmallStr::from_static("lineage.family_pair");
    pub const LINEAGE_FAMILY_PAIR_UNIQUE: SmallStr =
        SmallStr::from_static("lineage.family_pair_unique");
    pub const LINEAGE_TOP1_PAIR_SHARE: SmallStr = SmallStr::from_static("lineage.top1_pair_share");

    pub const UNIQUE_MEMBERS: SmallStr = SmallStr::from_static("unique.members");
    pub const UNIQUE_SCORES: SmallStr = SmallStr::from_static("unique.scores");
    pub const NEW_CHILDREN: SmallStr = SmallStr::from_static("new.children");
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

    pub const RATE: &str = "rate";

    pub const STEP: &str = "step";

    pub const LINEAGE: &str = "lineage";

    pub const EXPR: &str = "expr";
}

const RULES: &[(&str, &[TagType])] = &[
    (metric_tags::SELECTOR, &[TagType::Selector]),
    (metric_tags::MUTATOR, &[TagType::Alterer, TagType::Mutator]),
    (
        metric_tags::CROSSOVER,
        &[TagType::Alterer, TagType::Crossover],
    ),
    (metric_tags::ALTERER, &[TagType::Alterer]),
    (metric_tags::SPECIES, &[TagType::Species]),
    (metric_tags::FAILURE, &[TagType::Failure]),
    (metric_tags::AGE, &[TagType::Age]),
    (metric_tags::FRONT, &[TagType::Front]),
    (metric_tags::DERIVED, &[TagType::Derived]),
    (metric_tags::OTHER, &[TagType::Other]),
    (metric_tags::SCORE, &[TagType::Score]),
    (metric_tags::RATE, &[TagType::Rate]),
    (metric_tags::STEP, &[TagType::Step]),
    (metric_tags::LINEAGE, &[TagType::Lineage]),
    (metric_tags::EXPR, &[TagType::Expr]),
];

const EXACT_TAGS: &[(&SmallStr, &[TagType])] = &[
    (
        &metric_names::REPLACE_AGE,
        &[TagType::Age, TagType::Failure],
    ),
    (&metric_names::REPLACE_INVALID, &[TagType::Failure]),
    //
    (&metric_names::FRONT_ADDITIONS, &[TagType::Front]),
    (&metric_names::FRONT_REMOVALS, &[TagType::Front]),
    (&metric_names::FRONT_COMPARISONS, &[TagType::Front]),
    (&metric_names::FRONT_ENTROPY, &[TagType::Front]),
    (&metric_names::FRONT_FILTERS, &[TagType::Front]),
    (&metric_names::FRONT_SIZE, &[TagType::Front]),
    //
    (
        &metric_names::SPECIES_AGE_FAIL,
        &[TagType::Species, TagType::Age, TagType::Failure],
    ),
    (
        &metric_names::SPECIES_AGE,
        &[TagType::Species, TagType::Age],
    ),
    //
    (&metric_names::UNIQUE_MEMBERS, &[TagType::Derived]),
    (&metric_names::UNIQUE_SCORES, &[TagType::Derived]),
    (&metric_names::NEW_CHILDREN, &[TagType::Derived]),
    (&metric_names::SURVIVOR_COUNT, &[TagType::Derived]),
    (&metric_names::CARRYOVER_RATE, &[TagType::Derived]),
    (&metric_names::DIVERSITY_RATIO, &[TagType::Derived]),
    (&metric_names::SCORE_VOLATILITY, &[TagType::Derived]),
    //
    (&metric_names::SCORES, &[TagType::Score]),
    //
    (
        &metric_names::ALTER_CROSS_FAMILY,
        &[TagType::Alterer, TagType::Lineage],
    ),
    (
        &metric_names::ALTER_WITHIN_FAMILY,
        &[TagType::Alterer, TagType::Lineage],
    ),
    (
        &metric_names::ALTER_PARENT_REUSE,
        &[TagType::Alterer, TagType::Lineage],
    ),
    //
    (&metric_names::LINEAGE_EVENTS, &[TagType::Lineage]),
    (
        &metric_names::LINEAGE_PARENTS_USED_UNIQUE,
        &[TagType::Lineage],
    ),
    (
        &metric_names::LINEAGE_PARENTS_USED_RATIO,
        &[TagType::Lineage],
    ),
    (
        &metric_names::LINEAGE_FAMILY_PAIR_ENTROPY,
        &[TagType::Lineage],
    ),
    (
        &metric_names::LINEAGE_FAMILY_PAIR_UNIQUE,
        &[TagType::Lineage],
    ),
    (&metric_names::LINEAGE_TOP1_PAIR_SHARE, &[TagType::Lineage]),
];

pub fn default_tags(metric_name: &SmallStr) -> Tag {
    let mut mask = Tag::empty();
    for (name, tags) in EXACT_TAGS {
        if *name == metric_name {
            for &k in *tags {
                mask.insert(k);
            }
        }
    }

    for (needle, tags) in RULES {
        if metric_name.contains(needle) {
            for &k in *tags {
                mask.insert(k);
            }
        }
    }

    mask
}
