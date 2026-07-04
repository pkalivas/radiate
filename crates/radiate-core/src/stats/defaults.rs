use crate::stats::{Tag, TagType};
use radiate_utils::SmallStr;

pub mod metric_names {
    use radiate_utils::SmallStr;

    pub const INDEX: SmallStr = SmallStr::from_static("index");
    pub const TIME: SmallStr = SmallStr::from_static("time");

    pub const AGE: SmallStr = SmallStr::from_static("age");

    pub const REPLACE_AGE: SmallStr = SmallStr::from_static("replace.age");
    pub const REPLACE_INVALID: SmallStr = SmallStr::from_static("replace.invalid");

    pub const FILTER_UNIQUE_SCORES: SmallStr = SmallStr::from_static("filter.unique.scores");

    pub const SPECIES_AGE: SmallStr = SmallStr::from_static("species.age");
    pub const SPECIES_AGE_FAIL: SmallStr = SmallStr::from_static("species.fail.age");
    pub const SPECIES_NEW_RATIO: SmallStr = SmallStr::from_static("species.new.ratio");
    pub const SPECIES_CREATED: SmallStr = SmallStr::from_static("species.new");
    pub const SPECIES_DIED: SmallStr = SmallStr::from_static("species.fail.empty");
    pub const SPECIES_SIZE: SmallStr = SmallStr::from_static("species.size");
    pub const SPECIES_DISTANCE_DIST: SmallStr = SmallStr::from_static("species.distance");
    pub const SPECIES_EVENNESS: SmallStr = SmallStr::from_static("species.evenness");
    pub const LARGEST_SPECIES_SHARE: SmallStr = SmallStr::from_static("species.largest_share");
    pub const SPECIES_THRESHOLD: SmallStr = SmallStr::from_static("species.threshold");
    pub const SPECIES_COUNT: SmallStr = SmallStr::from_static("species.count");
    pub const SPECIES_ERROR: SmallStr = SmallStr::from_static("species.error");

    pub const FRONT_ADDITIONS: SmallStr = SmallStr::from_static("front.additions");
    pub const FRONT_REMOVALS: SmallStr = SmallStr::from_static("front.removals");
    pub const FRONT_ENTROPY: SmallStr = SmallStr::from_static("front.entropy");
    pub const FRONT_COMPARISONS: SmallStr = SmallStr::from_static("front.comparisons");
    pub const FRONT_FILTERS: SmallStr = SmallStr::from_static("front.filters");
    pub const FRONT_SIZE: SmallStr = SmallStr::from_static("front.size");

    pub const GENOME_SIZE: SmallStr = SmallStr::from_static("genome.size");

    pub const SURVIVOR_COUNT: SmallStr = SmallStr::from_static("count.survivor");
    pub const EVALUATION_COUNT: SmallStr = SmallStr::from_static("count.evaluation");
    pub const STAGNATION_COUNT: SmallStr = SmallStr::from_static("count.stagnation");

    pub const CARRYOVER_RATE: SmallStr = SmallStr::from_static("rate.carryover");
    pub const DIVERSITY_RATIO: SmallStr = SmallStr::from_static("rate.diversity");

    pub const SCORES: SmallStr = SmallStr::from_static("scores");
    pub const BEST_SCORES: SmallStr = SmallStr::from_static("scores.best");
    pub const SCORES_TREND: SmallStr = SmallStr::from_static("scores.trend");

    /// Pielou evenness of the population's fitness distribution, in `[0, 1]`.
    /// Pairs with [`UNIQUE_SCORES`] (richness — *how many* distinct scores) to
    /// describe *how the population is spread across* those scores.
    ///
    /// - `~1.0`: every distinct score is held by roughly the same number of
    ///   members — fitness is spread evenly, a healthy, exploring population.
    /// - `~0.0`: nearly all members share a few (or one) score — the population
    ///   has collapsed onto a plateau (premature convergence). This can read
    ///   low even when `stddev` looks fine, because a handful of outliers
    ///   inflate the spread while the bulk is piled up.
    ///
    /// Defined as Shannon entropy of the per-score frequencies normalized by
    /// `ln(unique_scores)`; reported as `0.0` when there is only one distinct
    /// score (no diversity to be even about).
    pub const SCORES_EVENNESS: SmallStr = SmallStr::from_static("scores.evenness");

    /// Gini coefficient of the population's fitness distribution, in `[0, 1]` —
    /// a measure of fitness *inequality* / effective selection pressure.
    ///
    /// - `0.0`: perfect equality — every member has the same score (fully
    ///   converged, or pre-evaluation).
    /// - low (e.g. `~0.1`): fitness is fairly uniform across the population.
    /// - high (e.g. `~0.7+`): a small elite holds most of the "fitness mass"
    ///   while the rest trail far behind — steep selection pressure, the regime
    ///   where a few individuals dominate reproduction.
    ///
    /// Independent of scale and (via an internal floor-shift) of sign, so it is
    /// comparable across runs and works for both maximization and minimization.
    /// Complements [`SCORE_VOLATILITY`] (coefficient of variation): both gauge
    /// spread, but Gini is distribution-shape based rather than mean-relative.
    pub const SCORES_GINI: SmallStr = SmallStr::from_static("scores.gini");

    pub const SCORE_VOLATILITY: SmallStr = SmallStr::from_static("score.volatility");
    pub const BEST_SCORE_IMPROVEMENT: SmallStr = SmallStr::from_static("score.improvement");

    /// Pearson correlation between genome size and fitness across the
    /// population, in `[-1, 1]` — the bloat signal. Only emitted when genome
    /// length actually varies (variable-length GP genomes); for fixed-length
    /// genomes there is no size variance and the metric is omitted.
    ///
    /// Sign is relative to the score scale, so interpret it together with the
    /// objective's direction. Reading it as "are bigger genomes scoring
    /// better?":
    /// - `~+0.9`: size and score move together almost lockstep — if bigger is
    ///   *not* buying proportionally better fitness, this is the classic bloat
    ///   signature (genomes growing without payoff).
    /// - `~0.0`: size and fitness are decoupled — growth (or shrinkage) is not
    ///   tracking quality.
    /// - `~-0.9`: smaller genomes score better — parsimony pressure is winning,
    ///   or large genomes are being penalized.
    ///
    /// Computed only over scored members so it never desyncs from skipped /
    /// unevaluated individuals.
    pub const SIZE_SCORE_CORR: SmallStr = SmallStr::from_static("genome.size.score.corr");

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
    (
        &metric_names::FILTER_UNIQUE_SCORES,
        &[TagType::Derived, TagType::Failure],
    ),
    //
    (&metric_names::UNIQUE_MEMBERS, &[TagType::Derived]),
    (&metric_names::UNIQUE_SCORES, &[TagType::Derived]),
    (&metric_names::NEW_CHILDREN, &[TagType::Derived]),
    (&metric_names::SURVIVOR_COUNT, &[TagType::Derived]),
    (&metric_names::CARRYOVER_RATE, &[TagType::Derived]),
    (&metric_names::DIVERSITY_RATIO, &[TagType::Derived]),
    (&metric_names::SCORE_VOLATILITY, &[TagType::Derived]),
    (&metric_names::SCORES_EVENNESS, &[TagType::Derived]),
    (&metric_names::SCORES_GINI, &[TagType::Derived]),
    (&metric_names::SIZE_SCORE_CORR, &[TagType::Derived]),
    //
    (&metric_names::SCORES, &[TagType::Score]),
    //
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
