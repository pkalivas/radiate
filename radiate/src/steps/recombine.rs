use super::EngineStep;
use crate::domain::timer::Timer;
use crate::{
    Alter, AlterResult, Chromosome, EngineContext, GeneticEngineParams, Metric, Objective,
    Population, Select, random_provider,
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

    fn apply_alterations<T>(
        &self,
        ctx: &mut EngineContext<C, T>,
        individuals: &mut Population<C>,
    ) -> AlterResult {
        let mut alter_result = AlterResult::default();

        for alterer in &self.alters {
            alter_result.merge(alterer.alter(individuals));
        }

        if let Some(metrics) = alter_result.take_metrics() {
            for metric in metrics {
                ctx.record_metric(metric);
            }
        }

        for id in alter_result.changed() {
            individuals.get_mut(*id).invalidate(ctx.index);
        }

        let mut metr = Metric::new_value("invalidated");
        metr.add_value(alter_result.2.len() as f32);
        ctx.record_metric(metr);

        alter_result
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
    fn select_survivors<T>(&self, ctx: &mut EngineContext<C, T>) -> Population<C> {
        let timer = Timer::new();
        let result =
            self.survivor_selector
                .select(&ctx.population, &self.objective, self.survivor_count);

        ctx.record_operation(self.survivor_selector.name(), result.len() as f32, timer);

        result
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
    fn create_offspring<T>(&self, ctx: &mut EngineContext<C, T>) -> Population<C> {
        let selector = &self.offspring_selector;
        if ctx.species.is_empty() || random_provider::random::<f32>() < 0.01 {
            let timer = Timer::new();

            let mut offspring =
                selector.select(&ctx.population(), &self.objective, self.offspring_count);

            ctx.record_operation(selector.name(), offspring.len() as f32, timer);

            self.objective.sort(&mut offspring);
            self.apply_alterations(ctx, &mut offspring);
            return offspring;
        }

        let mut offspring = Vec::new();
        let mut alter_result = AlterResult::default();
        let species_count = ctx.species.len();
        for i in 0..species_count {
            let species = &ctx.species[i];
            let scale = &ctx.species[species_count - 1 - i].score().as_f32();
            let timer = Timer::new();

            let count = (scale * self.offspring_count as f32).round() as usize;

            let mut selected = selector.select(species.population(), &self.objective, count);

            ctx.record_operation(selector.name(), count as f32, timer);
            self.objective.sort(&mut selected);

            alter_result.merge(self.apply_alterations(ctx, &mut selected));
            offspring.extend(selected);
        }

        offspring.into_iter().collect()
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

    fn execute(&self, ctx: &mut EngineContext<C, T>) {
        let survivors = self.select_survivors(ctx);
        let offspring = self.create_offspring(ctx);

        ctx.population = survivors
            .into_iter()
            .chain(offspring.into_iter())
            .collect::<Population<C>>();
    }
}
