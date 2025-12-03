use crate::steps::EngineStep;
use radiate_core::{
    Alter, Chromosome, Ecosystem, MetricSet, Objective, Optimize, Population, Select,
};
use radiate_error::Result;
use std::sync::Arc;

pub struct RecombineStep<C: Chromosome> {
    pub(crate) survivor_handle: SurvivorRecombineHandle<C>,
    pub(crate) offspring_handle: OffspringRecombineHandle<C>,
}

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
    pub fn select(&self, population: &Population<C>, metrics: &mut MetricSet) -> Population<C> {
        let time = std::time::Instant::now();
        let survivors = self
            .selector
            .select(&population, &self.objective, self.count);

        metrics.upsert(self.selector.name(), (survivors.len(), time.elapsed()));
        survivors
    }
}

pub struct OffspringRecombineHandle<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) objective: Objective,
    pub(crate) selector: Arc<dyn Select<C>>,
    pub(crate) alters: Vec<Arc<dyn Alter<C>>>,
}

impl<C> OffspringRecombineHandle<C>
where
    C: Chromosome + PartialEq + Clone,
{
    #[inline]
    pub fn create(
        &self,
        generation: usize,
        ecosystem: &Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        if let Some(species) = ecosystem.species() {
            let total_offspring = self.count as f32;
            let mut species_scores = species
                .iter()
                .filter_map(|spec| spec.score())
                .collect::<Vec<_>>();

            if let Objective::Single(Optimize::Minimize) = &self.objective {
                species_scores.reverse();
            }

            let mut next_population = Vec::with_capacity(self.count);

            for (species, score) in species.iter().zip(species_scores.iter()) {
                let count = (score.as_f32() * total_offspring).round() as usize;
                let time = std::time::Instant::now();
                let mut offspring =
                    self.selector
                        .select(&species.population(), &self.objective, count);
                metrics.upsert(self.selector.name(), (offspring.len(), time.elapsed()));

                self.objective.sort(&mut offspring);

                self.alters.iter().for_each(|alt| {
                    alt.alter(&mut offspring, generation)
                        .into_iter()
                        .for_each(|metric| {
                            metrics.add_or_update(metric);
                        });
                });

                next_population.extend(offspring);
            }

            Population::new(next_population)
        } else {
            let timer = std::time::Instant::now();
            let mut offspring =
                self.selector
                    .select(&ecosystem.population(), &self.objective, self.count);

            metrics.upsert(self.selector.name(), (offspring.len(), timer.elapsed()));

            self.objective.sort(&mut offspring);

            self.alters.iter().for_each(|alt| {
                alt.alter(&mut offspring, generation)
                    .into_iter()
                    .for_each(|metric| {
                        metrics.add_or_update(metric);
                    });
            });

            offspring
        }
    }
}

impl<C> EngineStep<C> for RecombineStep<C>
where
    C: Chromosome + PartialEq + Clone,
{
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) -> Result<()> {
        let survivors = self
            .survivor_handle
            .select(&ecosystem.population(), metrics);
        let offspring = self.offspring_handle.create(generation, ecosystem, metrics);

        ecosystem.population_mut().clear();
        ecosystem.population_mut().extend(survivors);
        ecosystem.population_mut().extend(offspring);

        Ok(())
    }
}
