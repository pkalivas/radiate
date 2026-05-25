use crate::{Metric, MetricSet};

impl MetricSet {
    pub fn time(&self) -> Option<&Metric> {
        self.get(super::metric_names::TIME)
    }

    pub fn score(&self) -> Option<&Metric> {
        self.get(super::metric_names::SCORES)
    }

    pub fn improvements(&self) -> Option<&Metric> {
        self.get(super::metric_names::BEST_SCORE_IMPROVEMENT)
    }

    pub fn age(&self) -> Option<&Metric> {
        self.get(super::metric_names::AGE)
    }

    pub fn replace_age(&self) -> Option<&Metric> {
        self.get(super::metric_names::REPLACE_AGE)
    }

    pub fn replace_invalid(&self) -> Option<&Metric> {
        self.get(super::metric_names::REPLACE_INVALID)
    }

    pub fn genome_size(&self) -> Option<&Metric> {
        self.get(super::metric_names::GENOME_SIZE)
    }

    pub fn front_size(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_SIZE)
    }

    pub fn front_comparisons(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_COMPARISONS)
    }

    pub fn front_removals(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_REMOVALS)
    }

    pub fn front_additions(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_ADDITIONS)
    }

    pub fn front_entropy(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_ENTROPY)
    }

    pub fn unique_members(&self) -> Option<&Metric> {
        self.get(super::metric_names::UNIQUE_MEMBERS)
    }

    pub fn unique_scores(&self) -> Option<&Metric> {
        self.get(super::metric_names::UNIQUE_SCORES)
    }

    pub fn new_children(&self) -> Option<&Metric> {
        self.get(super::metric_names::NEW_CHILDREN)
    }

    pub fn survivor_count(&self) -> Option<&Metric> {
        self.get(super::metric_names::SURVIVOR_COUNT)
    }

    pub fn carryover_rate(&self) -> Option<&Metric> {
        self.get(super::metric_names::CARRYOVER_RATE)
    }

    pub fn evaluation_count(&self) -> Option<&Metric> {
        self.get(super::metric_names::EVALUATION_COUNT)
    }

    pub fn diversity_ratio(&self) -> Option<&Metric> {
        self.get(super::metric_names::DIVERSITY_RATIO)
    }

    pub fn score_volatility(&self) -> Option<&Metric> {
        self.get(super::metric_names::SCORE_VOLATILITY)
    }

    pub fn species_count(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_COUNT)
    }

    pub fn species_age_fail(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_AGE_FAIL)
    }

    pub fn species_distance_dist(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_DISTANCE_DIST)
    }

    pub fn species_created(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_CREATED)
    }

    pub fn species_died(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_DIED)
    }

    pub fn species_age(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_AGE)
    }

    pub fn species_size(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_SIZE)
    }

    pub fn species_evenness(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_EVENNESS)
    }

    pub fn largest_species_share(&self) -> Option<&Metric> {
        self.get(super::metric_names::LARGEST_SPECIES_SHARE)
    }

    pub fn species_new_ratio(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_NEW_RATIO)
    }

    pub fn front_filters(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_FILTERS)
    }

    pub fn best_scores(&self) -> Option<&Metric> {
        self.get(super::metric_names::BEST_SCORES)
    }

    pub fn index(&self) -> Option<&Metric> {
        self.get(super::metric_names::INDEX)
    }

    pub fn species_threshold(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_THRESHOLD)
    }
}
