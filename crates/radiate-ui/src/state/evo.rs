use std::sync::Arc;

use super::chart::{ChartState, LineChartType};
use crate::chart::RollingLineChart;
use crate::widgets::num_pairs;
use radiate_engines::{
    Chromosome, ContextAudit, Ecosystem, Front, MetricSet, Objective, Optimize, Phenotype, Score,
    Species,
};

pub struct ObjectiveState {
    pub objective: Objective,
    pub charts_visible: usize,
    pub chart_start_index: usize,
    pub objective_index: usize,
}

pub struct EvoState<C: Chromosome> {
    pub best_phenotype: Option<Phenotype<C>>,
    pub ecosystem: Option<Arc<Ecosystem<C>>>,
    pub front: Option<Front<Phenotype<C>>>,
    pub metrics: MetricSet,
    pub charts: ChartState,
    pub index: usize,
    pub score: Score,
    pub pareto: ObjectiveState,
}

impl<C: Chromosome> EvoState<C> {
    pub fn update_ecosystem(&mut self, ecosystem: Arc<Ecosystem<C>>) {
        self.ecosystem = Some(ecosystem);
    }

    pub fn update_metrics(&mut self, metrics: MetricSet) {
        for metric in metrics.iter() {
            self.charts.update_from_metric(metric.1);
        }
        self.metrics = metrics;
    }

    pub fn update_audits(&mut self, audits: Option<Vec<ContextAudit>>)
    where
        C: Clone,
    {
        if let Some(audits) = audits {
            for audit in audits.iter() {
                if let ContextAudit::NewBest = audit {
                    let phenotype = self
                        .ecosystem
                        .as_ref()
                        .and_then(|eco| eco.get_phenotype(0))
                        .cloned();
                    self.best_phenotype = phenotype;
                }
            }
        }
    }

    pub fn get_chart_by_key(
        &self,
        key: &str,
        chart_type: LineChartType,
    ) -> Option<&RollingLineChart> {
        self.charts.get_line_chart(key, chart_type)
    }

    pub fn get_species(&self) -> Option<&Vec<Species<C>>> {
        self.ecosystem.as_ref().and_then(|eco| eco.species())
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
            front: None,
            metrics: MetricSet::new(),
            charts: ChartState::new(),
            ecosystem: None,
            index: 0,
            score: Score::default(),
            pareto: ObjectiveState {
                objective: Objective::Single(Optimize::Maximize),
                charts_visible: 2,
                chart_start_index: 0,
                objective_index: 0,
            },
        }
    }
}
