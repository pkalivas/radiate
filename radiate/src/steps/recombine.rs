use super::EngineStep;
use crate::{
    Alter, AlterResult, Chromosome, GeneticEngineParams, Metric, Objective, Population, Select,
    Species, metric_names, random_provider,
};
use std::sync::Arc;

pub struct RecombineStep<C: Chromosome> {
    survivor_selector: Arc<dyn Select<C>>,
    offspring_selector: Arc<dyn Select<C>>,
    alters: Vec<Arc<dyn Alter<C>>>,
    survivor_count: usize,
    offspring_count: usize,
    objective: Objective,
}

impl<C: Chromosome> RecombineStep<C> {
    pub fn new(
        survivor_selector: Arc<dyn Select<C>>,
        offspring_selector: Arc<dyn Select<C>>,
        alters: Vec<Arc<dyn Alter<C>>>,
        survivor_count: usize,
        offspring_count: usize,
        objective: Objective,
    ) -> Self {
        RecombineStep {
            survivor_selector,
            offspring_selector,
            alters,
            survivor_count,
            offspring_count,
            objective,
        }
    }

    fn select_offspring(
        &self,
        individuals: &Population<C>,
        count: usize,
    ) -> (Population<C>, Metric) {
        let time = std::time::Instant::now();
        let mut selected = self
            .offspring_selector
            .select(individuals, &self.objective, count);

        self.objective.sort(&mut selected);

        let length = selected.len() as f32;
        let metric = Metric::new_operations(self.offspring_selector.name(), length, time.elapsed());

        (selected, metric)
    }

    fn apply_alterations(&self, generation: usize, individuals: &mut Population<C>) -> Vec<Metric> {
        let mut metrics = Vec::new();
        let mut alter_result = AlterResult::default();

        for alterer in &self.alters {
            alter_result.merge(alterer.alter(individuals));
        }

        if let Some(mets) = alter_result.take_metrics() {
            for metric in mets {
                metrics.push(metric);
            }
        }

        for id in alter_result.changed() {
            individuals.get_mut(*id).invalidate(generation);
        }

        metrics
            .into_iter()
            .chain(vec![
                Metric::new_value(metric_names::ALTERED).with_value(alter_result.2.len() as f32),
            ])
            .collect::<Vec<Metric>>()
    }

    /// the `select_survivors` method selects the individuals that will survive
    /// to the next generation. The number of survivors is determined by the population size and the
    /// offspring fraction specified in the genetic engine parameters. The survivors
    /// are selected using the survivor selector specified in the genetic engine parameters,
    /// which is typically a selection algorithm like tournament selection
    /// or roulette wheel selection. For example, if the population size is 100 and the offspring
    /// fraction is 0.8, then 20 individuals will be selected as survivors.
    ///
    /// The reasoning behind this is to ensure that a subset of the population is retained
    /// to the next generation, while the rest of the population is replaced with new individuals created
    /// through crossover and mutation. By selecting a subset of the population to survive, the genetic algorithm
    /// can maintain some of the best solutions found so far while also introducing new genetic material/genetic diversity.
    ///
    /// This method returns a new population containing only the selected survivors.
    fn select_survivors(&self, population: &Population<C>) -> (Population<C>, Vec<Metric>) {
        let timer = std::time::Instant::now();
        let result =
            self.survivor_selector
                .select(population, &self.objective, self.survivor_count);

        let length = result.len() as f32;

        (
            result,
            vec![Metric::new_operations(
                self.survivor_selector.name(),
                length,
                timer.elapsed(),
            )],
        )
    }

    /// Create the offspring that will be used to create the next generation. The number of offspring
    /// is determined by the population size and the offspring fraction specified in the genetic
    /// engine parameters. The offspring are selected using the offspring selector specified in the
    /// genetic engine parameters, which, like the survivor selector, is typically a selection algorithm
    /// like tournament selection or roulette wheel selection. For example, if the population size is 100
    /// and the offspring fraction is 0.8, then 80 individuals will be selected as offspring which will
    /// be used to create the next generation through crossover and mutation.
    ///
    /// Once the parents are selected through the offspring selector, the `create_offspring` method
    /// will apply the mutation and crossover operations specified during engine creation to the
    /// selected parents, creating a new population of `Phenotypes` with the same size as the
    /// `offspring_fraction` specifies. This process introduces new genetic material into the population,
    /// which allows the genetic algorithm explore new solutions in the problem space and (hopefully)
    /// avoid getting stuck in local minima.
    fn create_offspring(
        &self,
        generation: usize,
        population: &Population<C>,
        species: &Vec<Species<C>>,
    ) -> (Population<C>, Vec<Metric>) {
        let mut metrics = Vec::new();

        if species.is_empty() || random_provider::random::<f32>() < 0.01 {
            let (mut offspring, metric) = self.select_offspring(&population, self.offspring_count);

            metrics.push(metric);
            metrics.extend(self.apply_alterations(generation, &mut offspring));

            return (offspring, metrics);
        }

        let mut altered_individuals = Vec::new();
        let mut species_scores = species.iter().map(|s| s.score()).collect::<Vec<_>>();

        if let Objective::Single(crate::Optimize::Minimize) = &self.objective {
            species_scores.reverse();
        }

        for (species, score) in species.iter().zip(species_scores.iter()) {
            let count = (score.as_f32() * self.offspring_count as f32).round() as usize;

            let (mut offspring, metric) = self.select_offspring(species.population(), count);

            metrics.push(metric);
            for metric in self.apply_alterations(generation, &mut offspring) {
                metrics.push(metric);
            }

            altered_individuals.extend(offspring);
        }

        (altered_individuals.into_iter().collect(), metrics)
    }
}

/// Recombines the survivors and offspring populations to create the next generation. The survivors
/// are the individuals from the previous generation that will survive to the next generation, while the
/// offspring are the individuals that were selected from the previous generation then altered.
/// This method combines the survivors and offspring populations into a single population that
/// will be used in the next iteration of the genetic algorithm.
impl<C, T> EngineStep<C, T> for RecombineStep<C>
where
    C: Chromosome,
    T: Clone + Send,
{
    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        Some(Box::new(RecombineStep {
            survivor_selector: Arc::clone(&params.survivor_selector()),
            offspring_selector: Arc::clone(&params.offspring_selector()),
            alters: params
                .alters()
                .iter()
                .map(|alter| Arc::clone(alter))
                .collect::<Vec<Arc<dyn Alter<C>>>>(),
            survivor_count: params.survivor_count(),
            offspring_count: params.offspring_count(),
            objective: params.objective().clone(),
        }))
    }

    fn execute(
        &self,
        generation: usize,
        population: &mut Population<C>,
        species: &mut Vec<Species<C>>,
    ) -> Vec<Metric> {
        let (survivors, survivor_metrics) = self.select_survivors(population);
        let (offspring, offspring_metrics) =
            self.create_offspring(generation, &population, species);

        (*population) = survivors
            .into_iter()
            .chain(offspring.into_iter())
            .collect::<Population<C>>();

        survivor_metrics
            .into_iter()
            .chain(offspring_metrics.into_iter())
            .collect::<Vec<Metric>>()
    }
}
