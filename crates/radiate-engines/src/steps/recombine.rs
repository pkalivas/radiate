use crate::steps::EngineStep;
use radiate_core::{
    Alterer, Chromosome, Ecosystem, MetricSet, Objective, Optimize, Population, Score, Select,
};
use radiate_error::Result;
use std::sync::Arc;

pub struct RecombineStep<C: Chromosome> {
    pub(crate) survivor_handle: SurvivorRecombineHandle<C>,
    pub(crate) offspring_handle: OffspringRecombineHandle<C>,
}

impl<C> EngineStep<C> for RecombineStep<C>
where
    C: Chromosome + PartialEq + Clone,
{
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        let survivors = self.survivor_handle.select(ecosystem, metrics);
        let offspring = self.offspring_handle.create(generation, ecosystem, metrics);

        let population = ecosystem.population_mut();

        population.clear();
        population.extend(survivors);
        population.extend(offspring);

        Ok(())
    }
}

#[derive(Clone)]
pub struct SurvivorRecombineHandle<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) objective: Objective,
    pub(crate) selector: Arc<dyn Select<C>>,
}

impl<C> SurvivorRecombineHandle<C>
where
    C: Chromosome + Clone,
{
    #[inline]
    pub fn select(&self, ecosystem: &Ecosystem<C>, metrics: &mut MetricSet) -> Population<C> {
        let time = std::time::Instant::now();
        let survivors = self
            .selector
            .select(&ecosystem.population(), &self.objective, self.count);
        metrics.upsert((self.selector.name(), (survivors.len(), time.elapsed())));
        survivors
    }
}

#[derive(Clone)]
pub struct OffspringRecombineHandle<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) objective: Objective,
    pub(crate) selector: Arc<dyn Select<C>>,
    pub(crate) alters: Vec<Alterer<C>>,
}

impl<C> OffspringRecombineHandle<C>
where
    C: Chromosome + PartialEq + Clone,
{
    #[inline]
    pub fn create<'a>(
        &self,
        generation: usize,
        ecosystem: &Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        if let Some(species) = ecosystem.species() {
            let mut species_scores = species
                .iter()
                .filter_map(|spec| spec.score())
                .collect::<Vec<_>>();

            if let Objective::Single(Optimize::Minimize) = &self.objective {
                species_scores.reverse();
            }

            let quotas = self.quotas_from_scores(&species_scores);

            let mut next_population = Population::with_capacity(self.count);
            for (species, count) in species.iter().zip(quotas.iter()) {
                let time = std::time::Instant::now();
                let mut offspring =
                    self.selector
                        .select(species.population(), &self.objective, *count);

                metrics.upsert((self.selector.name(), (offspring.len(), time.elapsed())));

                self.objective.sort(&mut offspring);

                self.alters.iter().for_each(|alt| {
                    metrics.upsert(alt.alter(&mut offspring, generation));
                });

                next_population.extend(offspring);
            }

            next_population
        } else {
            let timer = std::time::Instant::now();
            let mut offspring =
                self.selector
                    .select(ecosystem.population(), &self.objective, self.count);

            metrics.upsert((self.selector.name(), (offspring.len(), timer.elapsed())));

            self.objective.sort(&mut offspring);

            self.alters.iter().for_each(|alt| {
                metrics.upsert(alt.alter(&mut offspring, generation));
            });

            offspring
        }
    }

    fn quotas_from_scores(&self, scores: &[&Score]) -> Vec<usize> {
        let n = scores.len();
        if n == 0 || self.count == 0 {
            return vec![0; n];
        }

        let raw_scores = scores.iter().map(|s| s.as_f32()).collect::<Vec<f32>>();
        let mut min_score = raw_scores.iter().cloned().fold(f32::INFINITY, f32::min);
        if !min_score.is_finite() {
            min_score = 0.0;
        }

        let shifted = raw_scores
            .iter()
            .map(|s| (s - min_score).max(0.0))
            .collect::<Vec<f32>>();

        let sum = shifted.iter().sum::<f32>();

        if sum <= f32::EPSILON {
            let base = self.count / n;
            let mut quotas = vec![base; n];
            let mut remaining = self.count - base * n;
            let mut i = 0;
            while remaining > 0 {
                quotas[i] += 1;
                remaining -= 1;
                i += 1;
            }
            return quotas;
        }

        let total = self.count as f32;

        let mut quotas = Vec::with_capacity(n);
        let mut fracs = Vec::with_capacity(n);
        let mut assigned = 0;

        for (idx, w) in shifted.iter().enumerate() {
            let p = *w / sum;
            let exact = p * total;
            let base = exact.floor() as usize;
            let frac = exact - base as f32;

            quotas.push(base);
            fracs.push((frac, idx));
            assigned += base;
        }

        let remaining = self.count.saturating_sub(assigned);
        fracs.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, idx) in fracs.iter().take(remaining) {
            quotas[*idx] += 1;
        }

        quotas
    }
}
