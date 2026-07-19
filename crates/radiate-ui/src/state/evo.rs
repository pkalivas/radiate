use super::chart::{ChartState, MetricChartType};
use crate::chart::RollingLineChart;
use crate::widgets::num_pairs;
use radiate_engines::{
    Chromosome, Ecosystem, Front, MetricSet, Objective, Optimize, Phenotype, Score, Species,
};
use radiate_utils::WindowBuffer;
use std::sync::{Arc, RwLock};

const MAX_IMPROVEMENT_LOG: usize = 100;

pub struct ImprovementEntry {
    pub generation: usize,
    pub score: f32,
    pub delta: f32,
}

pub struct FrontEventEntry {
    pub generation: usize,
    pub front_size: usize,
    pub additions: usize,
    pub removals: usize,
    pub comparisons: usize,
    pub filtered: bool,
}

pub struct ObjectiveState {
    pub objective: Objective,
    pub charts_visible: usize,
    pub chart_start_index: usize,
    pub objective_index: usize,
}

#[allow(dead_code)]
pub struct EvoState<C: Chromosome> {
    pub best_phenotype: Option<Phenotype<C>>,
    pub ecosystem: Option<Ecosystem<C>>,
    pub front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub metrics: MetricSet,
    pub charts: ChartState,
    pub index: usize,
    pub score: Score,
    pub best_score: Score,
    pub pareto: ObjectiveState,
    pub improvement_log: WindowBuffer<ImprovementEntry>,
    pub front_event_log: WindowBuffer<FrontEventEntry>,
}

impl<C: Chromosome> EvoState<C> {
    pub fn update_score(&mut self, new_score: Score) {
        if self.score.is_empty() {
            self.score = new_score.clone();
            return;
        }

        if self.pareto.objective.is_single() {
            let prev = self.score.as_f32();
            let next = new_score.as_f32();
            let delta = match &self.pareto.objective {
                Objective::Single(Optimize::Minimize) => prev - next,
                _ => next - prev,
            };

            if delta > 0.0 {
                self.best_score = new_score.clone();
                self.improvement_log.push_front(ImprovementEntry {
                    generation: self.index,
                    score: next,
                    delta,
                });
            }
        }

        self.score = new_score;
    }

    pub fn update_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn update_ecosystem(&mut self, ecosystem: Ecosystem<C>)
    where
        C: Clone,
    {
        self.ecosystem = Some(ecosystem);
        let phenotype = self
            .ecosystem
            .as_ref()
            .and_then(|eco| eco.get_phenotype(0))
            .cloned();
        self.best_phenotype = phenotype;
    }

    pub fn update_metrics(&mut self, metrics: MetricSet) {
        for metric in metrics.iter() {
            self.charts.update_from_metric(metric);
        }

        self.metrics = metrics;
        self.update_front_events();
    }

    fn update_front_events(&mut self) {
        if self.pareto.objective.is_single() {
            return;
        }

        let additions = self
            .metrics
            .front_additions()
            .map(|m| m.last_value() as usize)
            .unwrap_or(0);

        if additions == 0 {
            return;
        }
        let removals = self
            .metrics
            .front_removals()
            .map(|m| m.last_value() as usize)
            .unwrap_or(0);

        let front_size = self
            .metrics
            .front_size()
            .map(|m| m.last_value() as usize)
            .unwrap_or(0);

        let front_comparisons = self
            .metrics
            .front_comparisons()
            .map(|m| m.last_value() as usize)
            .unwrap_or(0);

        let front_filters = self
            .metrics
            .front_filters()
            .map(|m| m.last_value() > 0.0)
            .unwrap_or(false);

        self.front_event_log.push_front(FrontEventEntry {
            generation: self.index,
            front_size,
            additions,
            removals,
            comparisons: front_comparisons,
            filtered: front_filters,
        });
    }

    pub fn get_chart_by_key(
        &self,
        key: &str,
        chart_type: MetricChartType,
    ) -> Option<&RollingLineChart> {
        self.charts.get_line_chart(key, chart_type)
    }

    pub fn get_species(&self) -> Option<&Vec<Species<C>>> {
        self.ecosystem.as_ref().and_then(|eco| eco.species())
    }

    pub fn is_multi(&self) -> bool {
        !self.pareto.objective.is_single()
    }

    pub fn has_species(&self) -> bool {
        self.get_species()
            .is_some_and(|species| !species.is_empty())
    }

    pub fn set_objective_index(&mut self, index: usize) {
        if index < self.pareto.objective.dims() {
            self.pareto.objective_index = index;
        }
    }

    pub fn expand_objective_pairs(&mut self) {
        self.pareto.charts_visible = self
            .pareto
            .charts_visible
            .saturating_add(1)
            .min(num_pairs(self.pareto.objective.dims()));
    }

    pub fn shrink_objective_pairs(&mut self) {
        if self.pareto.charts_visible > 1 {
            self.pareto.charts_visible -= 1;
        }
    }

    pub fn next_objective_pair_page(&mut self) {
        let step = self.pareto.charts_visible.max(1);
        let total = num_pairs(self.pareto.objective.dims());
        let current = self.pareto.chart_start_index;
        if current + step < total {
            self.pareto.chart_start_index += step;
        }
    }

    pub fn previous_objective_pair_page(&mut self) {
        let step = self.pareto.charts_visible.max(1);
        let current = self.pareto.chart_start_index;
        if current >= step {
            self.pareto.chart_start_index -= step;
        } else {
            self.pareto.chart_start_index = 0;
        }
    }
}

impl<C: Chromosome> Default for EvoState<C> {
    fn default() -> Self {
        Self {
            best_phenotype: None,
            front: Arc::new(RwLock::new(Front::default())),
            metrics: MetricSet::new(),
            charts: ChartState::new(),
            ecosystem: None,
            index: 0,
            score: Score::default(),
            best_score: Score::default(),
            improvement_log: WindowBuffer::with_capacity(MAX_IMPROVEMENT_LOG),
            front_event_log: WindowBuffer::with_capacity(MAX_IMPROVEMENT_LOG),
            pareto: ObjectiveState {
                objective: Objective::Single(Optimize::Maximize),
                charts_visible: 2,
                chart_start_index: 0,
                objective_index: 0,
            },
        }
    }
}
