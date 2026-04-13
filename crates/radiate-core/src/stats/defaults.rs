use crate::stats::{Tag, TagKind};

pub mod metric_names {
    pub const TIME: &str = "time";

    pub const AGE: &str = "age";
    pub const REPLACE_AGE: &str = "age.replace";
    pub const SPECIES_AGE_FAIL: &str = "age.species.fail";
    pub const SPECIES_AGE: &str = "age.species";

    pub const SPECIES_NEW_RATIO: &str = "new.species.ratio";
    pub const FRONT_ADDITIONS: &str = "new.front";
    pub const SPECIES_CREATED: &str = "new.species";

    pub const REPLACE_INVALID: &str = "invalid.replace";
    pub const FRONT_REMOVALS: &str = "invalid.front";
    pub const SPECIES_DIED: &str = "invalid.species";

    pub const GENOME_SIZE: &str = "size.genome";
    pub const FRONT_SIZE: &str = "size.front";
    pub const SPECIES_SIZE: &str = "size.species";

    pub const FRONT_ENTROPY: &str = "front.entropy";
    pub const FRONT_COMPARISONS: &str = "front.comparisons";
    pub const FRONT_FILTERS: &str = "front.filters";

    pub const SURVIVOR_COUNT: &str = "count.survivor";
    pub const EVALUATION_COUNT: &str = "count.evaluation";
    pub const SPECIES_COUNT: &str = "count.species";

    pub const CARRYOVER_RATE: &str = "rate.carryover";
    pub const DIVERSITY_RATIO: &str = "rate.diversity";
    pub const LINEAGE_PARENTS_USED_RATIO: &str = "rate.lineage.parents_used";

    pub const SCORES: &str = "scores";
    pub const BEST_SCORES: &str = "scores.best";
    pub const SCORE_VOLATILITY: &str = "score.volatility";
    pub const BEST_SCORE_IMPROVEMENT: &str = "score.improvement";

    pub const INDEX: &str = "index";

    pub const SPECIES_DISTANCE_DIST: &str = "species.distance";
    pub const SPECIES_EVENNESS: &str = "species.evenness";
    pub const LARGEST_SPECIES_SHARE: &str = "species.largest_share";
    pub const SPECIES_THRESHOLD: &str = "species.threshold";

    pub const ALTER_PARENT_REUSE: &str = "alter.parent_reuse";
    pub const ALTER_WITHIN_FAMILY: &str = "alter.within_family";
    pub const ALTER_CROSS_FAMILY: &str = "alter.cross_family";

    pub const LINEAGE_EVENTS: &str = "lineage.events";
    pub const LINEAGE_PARENTS_USED_UNIQUE: &str = "lineage.parents_unique";
    // •	FAMILY_PAIR_ENTROPY in [0, 1]
    // •	0.0 = basically always the same pair (pairing collapse)
    // •	1.0 = pairings evenly distributed across the pairs that occurred
    pub const LINEAGE_FAMILY_PAIR_ENTROPY: &str = "lineage.family_pair";
    pub const LINEAGE_FAMILY_PAIR_UNIQUE: &str = "lineage.family_pair_unique";
    pub const LINEAGE_TOP1_PAIR_SHARE: &str = "lineage.top1_pair_share";

    pub const UNIQUE_MEMBERS: &str = "unique.members";
    pub const UNIQUE_SCORES: &str = "unique.scores";
    pub const NEW_CHILDREN: &str = "new.children";
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
    (metric_tags::RATE, &[TagKind::Rate]),
    (metric_tags::STEP, &[TagKind::Step]),
    (metric_tags::LINEAGE, &[TagKind::Lineage]),
    (metric_tags::EXPR, &[TagKind::Expr]),
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

        metric_names::ALTER_CROSS_FAMILY
        | metric_names::ALTER_WITHIN_FAMILY
        | metric_names::ALTER_PARENT_REUSE => {
            mask.insert(TagKind::Alterer);
            mask.insert(TagKind::Lineage);
        }

        metric_names::LINEAGE_EVENTS
        | metric_names::LINEAGE_PARENTS_USED_UNIQUE
        | metric_names::LINEAGE_PARENTS_USED_RATIO
        | metric_names::LINEAGE_FAMILY_PAIR_ENTROPY
        | metric_names::LINEAGE_TOP1_PAIR_SHARE => {
            mask.insert(TagKind::Lineage);
        }

        x if x.contains(metric_tags::STEP) => {
            mask.insert(TagKind::Step);
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
        metric.add_tags(tags);
    }
}
