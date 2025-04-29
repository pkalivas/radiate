use radiate_core::{
    Alter, Chromosome, Ecosystem, MetricSet, Objective, Population, Select, engine::EngineStep,
    timer::Timer,
};
use std::sync::Arc;

pub struct RecombineStep<C: Chromosome> {
    pub(crate) survivor_selector: Arc<dyn Select<C>>,
    pub(crate) offspring_selector: Arc<dyn Select<C>>,
    pub(crate) alters: Vec<Arc<dyn Alter<C>>>,
    pub(crate) survivor_count: usize,
    pub(crate) offspring_count: usize,
    pub(crate) objective: Objective,
}

impl<C: Chromosome> RecombineStep<C> {
    pub fn select_survivors(
        &self,
        population: &Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        Self::select(
            self.survivor_count,
            &population.population,
            &self.objective,
            metrics,
            &self.survivor_selector,
        )
    }

    pub fn select_offspring(
        &self,
        count: usize,
        population: &Population<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        Self::select(
            count,
            &population,
            &self.objective,
            metrics,
            &self.offspring_selector,
        )
    }

    fn apply_alterations(
        &self,
        generation: usize,
        offspring: &mut Population<C>,
        metrics: &mut MetricSet,
    ) {
        self.alters.iter().for_each(|alt| {
            alt.alter(offspring, generation)
                .into_iter()
                .for_each(|metric| {
                    metrics.upsert(metric);
                });
        });
    }

    pub fn create_offspring(
        &self,
        generation: usize,
        ecosystem: &Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        if let Some(species) = ecosystem.species.as_ref() {
            let mut species_scores = species
                .iter()
                .filter_map(|spec| spec.score())
                .collect::<Vec<_>>();

            if let Objective::Single(crate::Optimize::Minimize) = &self.objective {
                species_scores.reverse();
            }
            let mut offspring = Vec::new();
            for (species, score) in species.iter().zip(species_scores.iter()) {
                let count = (score.as_f32() * self.offspring_count as f32).round() as usize;
                let mut selected_offspring =
                    self.select_offspring(count, &species.population, metrics);

                self.apply_alterations(generation, &mut selected_offspring, metrics);

                for individual in selected_offspring.into_iter() {
                    offspring.push(individual);
                }
            }

            Population::new(offspring)
        } else {
            let mut offspring =
                self.select_offspring(self.offspring_count, &ecosystem.population, metrics);

            self.objective.sort(&mut offspring);

            self.apply_alterations(generation, &mut offspring, metrics);
            offspring
        }
    }

    fn select(
        count: usize,
        population: &Population<C>,
        objective: &Objective,
        metrics: &mut MetricSet,
        selector: &Arc<dyn Select<C>>,
    ) -> Population<C> {
        let timer = Timer::new();
        let selected = selector.select(population, objective, count);

        metrics.upsert_operations(selector.name(), selected.len() as f32, timer);
        selected
    }
}

impl<C> EngineStep<C> for RecombineStep<C>
where
    C: Chromosome + 'static,
{
    fn execute(
        &self,
        generation: usize,
        metrics: &mut radiate_core::MetricSet,
        ecosystem: &mut radiate_core::Ecosystem<C>,
    ) {
        let survivors = self.select_survivors(ecosystem, metrics);
        let offspring = self.create_offspring(generation, ecosystem, metrics);

        ecosystem.population = survivors
            .into_iter()
            .chain(offspring.into_iter())
            .collect::<Population<C>>();
    }
}
