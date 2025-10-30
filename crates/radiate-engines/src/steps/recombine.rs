use crate::steps::EngineStep;
use radiate_core::{
    Alter, Chromosome, Ecosystem, MetricSet, ModelLearner, Objective, Optimize, Phenotype,
    Population, Select, labels,
};
use std::sync::Arc;

// impl<C> EngineStep<C> for EdaStep<C>
// where C: Chromosome + PartialEq + Clone {
//     fn execute(&mut self, generation: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {
//         let survivors = Self::select(self.survivor_count, &ecosystem.population, &self.objective, metrics, &self.survivor_selector);
//         let parents   = Self::select(self.offspring_count, &ecosystem.population, &self.objective, metrics, &self.offspring_selector);

//         let t0 = std::time::Instant::now();
//         let model = self.learner.learn(&parents);
//         metrics.upsert(model.name(), ("train_time", t0.elapsed().as_secs_f32()));

//         let n_model = ((self.offspring_count as f32) * self.mix_ratio).round() as usize;
//         let n_fallback = self.offspring_count - n_model;

//         let t1 = std::time::Instant::now();
//         let mut offspring = Population::new(
//             model.sample(n_model).into_iter().map(|g| Phenotype::from((g, generation))).collect()
//         );
//         metrics.upsert(model.name(), ("sample_time", t1.elapsed().as_secs_f32()));
//         metrics.upsert("EDA", ("offspring_model", n_model as i32));

//         if n_fallback > 0 {
//             let mut fb = parents.clone();
//             fb.truncate(n_fallback);
//             self.apply_alterations(generation, &mut fb, metrics);
//             offspring.extend(fb);
//         }
//         metrics.upsert("EDA", ("offspring_fallback", n_fallback as i32));

//         ecosystem.population_mut().clear();
//         survivors.into_iter().chain(offspring.into_iter()).for_each(|ind| ecosystem.population_mut().push(ind));
//     }
// }

pub struct RecombineStep<C: Chromosome> {
    pub(crate) survivor_selector: Arc<dyn Select<C>>,
    pub(crate) offspring_selector: Arc<dyn Select<C>>,
    pub(crate) alters: Vec<Arc<dyn Alter<C>>>,
    pub(crate) survivor_count: usize,
    pub(crate) offspring_count: usize,
    pub(crate) objective: Objective,
    pub(crate) dist_learner: Option<Arc<dyn ModelLearner<C>>>,
}

impl<C: Chromosome + PartialEq> RecombineStep<C> {
    #[inline]
    pub fn select_survivors(
        &self,
        population: &Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        let selected = Self::select(
            self.survivor_count,
            &population.population,
            &self.objective,
            metrics,
            &self.survivor_selector,
        );

        let name = self.survivor_selector.name();
        metrics.add_labels(
            name,
            labels![
                "operator" => "selector",
                "type" => "survivor",
                "method" => name,
            ],
        );

        selected
    }

    #[inline]
    pub fn select_offspring(
        &self,
        count: usize,
        population: &Population<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        let selected = Self::select(
            count,
            &population,
            &self.objective,
            metrics,
            &self.offspring_selector,
        );

        let name = self.offspring_selector.name();
        metrics.add_labels(
            name,
            labels![
                "operator" => "selector",
                "type" => "offspring",
                "method" => name,
            ],
        );

        selected
    }

    #[inline]
    pub fn create_offspring(
        &self,
        generation: usize,
        ecosystem: &Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Population<C>
    where
        C: Clone,
    {
        if let Some(species) = ecosystem.species.as_ref() {
            let total_offspring = self.offspring_count as f32;
            let mut species_scores = species
                .iter()
                .filter_map(|spec| spec.score())
                .collect::<Vec<_>>();

            if let Objective::Single(Optimize::Minimize) = &self.objective {
                species_scores.reverse();
            }

            let mut offspring = Vec::with_capacity(self.offspring_count);
            for (species, score) in species.iter().zip(species_scores.iter()) {
                let count = (score.as_f32() * total_offspring).round() as usize;
                let mut selected_offspring =
                    self.select_offspring(count, &species.population, metrics);

                self.objective.sort(&mut selected_offspring);

                self.apply_alterations(generation, &mut selected_offspring, metrics);

                offspring.extend(selected_offspring);
            }

            Population::new(offspring)
        } else {
            let mut offspring =
                self.select_offspring(self.offspring_count, &ecosystem.population, metrics);

            self.objective.sort(&mut offspring);

            if let Some(learner) = &self.dist_learner {
                let model = learner.fit(&offspring);
                let n_model = (self.offspring_count as f32 * 0.95).round() as usize;
                let n_fallback = self.offspring_count - n_model;

                let mut model_offspring = Population::new(
                    model
                        .sample(n_model)
                        .into_iter()
                        .map(|g| Phenotype::from((g, generation)))
                        .collect(),
                );

                if n_fallback > 0 {
                    offspring.truncate(n_fallback);
                    self.apply_alterations(generation, &mut offspring, metrics);
                    model_offspring.extend(offspring);
                }

                self.objective.sort(&mut model_offspring);

                model_offspring
            } else {
                self.apply_alterations(generation, &mut offspring, metrics);
                offspring
            }
        }
    }

    #[inline]
    fn select(
        count: usize,
        population: &Population<C>,
        objective: &Objective,
        metrics: &mut MetricSet,
        selector: &Arc<dyn Select<C>>,
    ) -> Population<C> {
        let timer = std::time::Instant::now();
        let selected = selector.select(population, objective, count);

        metrics.upsert(selector.name(), (selected.len(), timer.elapsed()));
        metrics.upsert(
            selector.name(),
            selected
                .iter()
                .map(|p| *p.id() as f32)
                .collect::<Vec<_>>()
                .as_slice(),
        );

        selected
    }

    #[inline]
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
                    metrics.add_or_update(metric);
                });
        });
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
    ) {
        let survivors = self.select_survivors(ecosystem, metrics);
        let offspring = self.create_offspring(generation, ecosystem, metrics);

        ecosystem.population_mut().clear();

        survivors
            .into_iter()
            .chain(offspring.into_iter())
            .for_each(|individual| {
                ecosystem.population_mut().push(individual);
            });
    }
}
