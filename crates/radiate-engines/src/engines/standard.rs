use radiate_core::{Codex, Engine, thread_pool::WaitGroup};
use radiate_core::{Ecosystem, EngineStep, MetricSet, Phenotype, Problem};

use crate::builder::GeneticEngineBuilder;
use crate::domain::timer::Timer;
use crate::genome::population::Population;
use crate::objectives::Objective;
use crate::steps::RecombineStep;
use crate::{
    Chromosome, EngineContext, EvaluateStep, GeneticEngineParams, Metric, Valid, metric_names,
};
use std::sync::{Arc, RwLock};

pub struct StandardEngine<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    evaluate: EvaluateStep<C, T>,
    recombine: RecombineStep<C>,
    params: GeneticEngineParams<C, T>,
}

impl<C, T> StandardEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    pub fn new(
        evaluate: EvaluateStep<C, T>,
        recombine: RecombineStep<C>,
        params: GeneticEngineParams<C, T>,
    ) -> Self {
        StandardEngine {
            evaluate,
            recombine,
            params,
        }
    }

    pub fn builder() -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default()
    }

    pub fn from_codex(codex: impl Codex<C, T> + 'static) -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default().codex(codex)
    }

    pub fn from_problem(problem: impl Problem<C, T> + 'static) -> GeneticEngineBuilder<C, T> {
        GeneticEngineBuilder::default().problem(problem)
    }

    /// Executes the genetic algorithm. The algorithm continues until a specified
    /// stopping condition, 'limit', is met, such as reaching a target fitness score or
    /// exceeding a maximum number of generations. When 'limit' returns true, the algorithm stops.
    pub fn run<F>(&self, limit: F) -> EngineContext<C, T>
    where
        F: Fn(&EngineContext<C, T>) -> bool,
    {
        let mut ctx = self.start();

        loop {
            self.next(&mut ctx);

            if limit(&ctx) {
                break self.stop(&mut ctx);
            }
        }
    }

    pub fn next(&self, ctx: &mut EngineContext<C, T>) {
        let generation = ctx.index;
        let mut metrics = &ctx.metrics;

        self.evaluate
            .execute(ctx.index, &mut ctx.metrics, &mut ctx.ecosystem);

        let survivors = self.select_survivors(ctx);
        let offspring = self.create_offspring(ctx);

        self.recombine(ctx, survivors, offspring);

        self.filter(ctx);
        self.evaluate(ctx);
        self.update_front(ctx);
        self.audit(ctx);
    }

    fn evaluate(&self, ctx: &mut EngineContext<C, T>) {
        let objective = self.params.objective();
        let thread_pool = self.params.thread_pool();
        let timer = Timer::new();

        let mut work_results = Vec::new();
        for idx in 0..ctx.ecosystem.population.len() {
            let individual = &mut ctx.ecosystem.population[idx];
            if individual.score().is_some() {
                continue;
            } else {
                let problem = self.params.problem();
                let geno = individual.take_genotype();
                let work = thread_pool.submit_with_result(move || {
                    let score = problem.eval(&geno);
                    (idx, score, geno)
                });

                work_results.push(work);
            }
        }

        let count = work_results.len() as f32;
        for work_result in work_results {
            let (idx, score, genotype) = work_result.result();
            ctx.ecosystem.population[idx].set_score(Some(score));
            ctx.ecosystem.population[idx].set_genotype(genotype);
        }

        ctx.upsert_operation(metric_names::EVALUATION, count, timer);

        objective.sort(&mut ctx.ecosystem.population);
    }

    fn select_survivors(&self, ctx: &mut EngineContext<C, T>) -> Population<C> {
        let selector = self.params.survivor_selector();
        let count = self.params.survivor_count();
        let objective = self.params.objective();

        let timer = Timer::new();
        let result = selector.select(&ctx.ecosystem.population, objective, count);

        ctx.upsert_operation(selector.name(), count as f32, timer);

        result
    }

    fn create_offspring(&self, ctx: &mut EngineContext<C, T>) -> Population<C> {
        let selector = self.params.offspring_selector();
        let count = self.params.offspring_count();
        let objective = self.params.objective();
        let alters = self.params.alters();

        let timer = Timer::new();
        let mut offspring = selector.select(&ctx.ecosystem.population, objective, count);

        ctx.upsert_operation(selector.name(), count as f32, timer);
        objective.sort(&mut offspring);

        alters.iter().for_each(|alt| {
            alt.alter(&mut offspring, ctx.index)
                .into_iter()
                .for_each(|metric| {
                    ctx.upsert_metric(metric);
                });
        });

        offspring
    }

    fn filter(&self, ctx: &mut EngineContext<C, T>) {
        let max_age = self.params.max_age();

        let generation = ctx.index;
        let population = &mut ctx.ecosystem.population;

        let timer = Timer::new();
        let mut age_count = 0_f32;
        let mut invalid_count = 0_f32;
        for i in 0..population.len() {
            let phenotype = &population[i];

            let mut removed = false;
            if phenotype.age(generation) > max_age {
                removed = true;
                age_count += 1_f32;
            } else if !phenotype.genotype().is_valid() {
                removed = true;
                invalid_count += 1_f32;
            }

            if removed {
                let replacement = self.params.replacement_strategy();
                let problem = self.params.problem();
                let encoder = Arc::new(move || problem.encode());

                let new_genotype = replacement.replace(population, encoder);
                population[i] = Phenotype::from((new_genotype, generation));
            }
        }

        let duration = timer.duration();
        ctx.upsert_operation(metric_names::FILTER_AGE, age_count, duration);
        ctx.upsert_operation(metric_names::FILTER_INVALID, invalid_count, duration);
    }

    fn recombine(
        &self,
        ctx: &mut EngineContext<C, T>,
        survivors: Population<C>,
        offspring: Population<C>,
    ) {
        ctx.ecosystem.population = survivors
            .into_iter()
            .chain(offspring)
            .collect::<Population<C>>();
    }

    fn audit(&self, ctx: &mut EngineContext<C, T>) {
        let audits = self.params.audits();
        let problem = self.params.problem();
        let optimize = self.params.objective();

        optimize.sort(&mut ctx.ecosystem.population);

        let audit_metrics = audits
            .iter()
            .flat_map(|audit| audit.audit(ctx.index(), &ctx.ecosystem.population))
            .collect::<Vec<Metric>>();

        for metric in audit_metrics {
            ctx.upsert_metric(metric);
        }

        if let Some(current_best) = ctx.ecosystem.population.get(0) {
            if let (Some(best), Some(current)) = (current_best.score(), &ctx.score) {
                if optimize.is_better(best, current) {
                    ctx.score = Some(best.clone());
                    ctx.best = problem.decode(current_best.genotype());
                }
            } else {
                ctx.score = Some(current_best.score().unwrap().clone());
                ctx.best = problem.decode(current_best.genotype());
            }
        }

        ctx.index += 1;
    }

    fn update_front(&self, ctx: &mut EngineContext<C, T>) {
        let objective = self.params.objective();
        let thread_pool = self.params.thread_pool();

        if let Objective::Multi(_) = objective {
            // TODO: Examine the clones here - it seems like we can reduce the number of clones of
            // the population. The front is a cheap clone (the values are wrapped in an Arc), but
            // the population is not. But at the same time - this is still pretty damn fast.
            let timer = Timer::new();
            let wg = WaitGroup::new();

            let new_individuals = ctx
                .ecosystem
                .population
                .iter()
                .filter(|pheno| pheno.generation() == ctx.index)
                .collect::<Vec<&Phenotype<C>>>();

            let front = Arc::new(RwLock::new(ctx.front.clone()));
            let dominates_vector = Arc::new(RwLock::new(vec![false; new_individuals.len()]));
            let remove_vector = Arc::new(RwLock::new(Vec::new()));

            for (idx, member) in new_individuals.iter().enumerate() {
                let pheno = Phenotype::clone(member);
                let front_clone = Arc::clone(&front);
                let doms_vector = Arc::clone(&dominates_vector);
                let remove_vector = Arc::clone(&remove_vector);

                thread_pool.group_submit(&wg, move || {
                    let (dominates, to_remove) = front_clone.read().unwrap().dominates(&pheno);

                    if dominates {
                        doms_vector.write().unwrap().get_mut(idx).map(|v| *v = true);
                        remove_vector
                            .write()
                            .unwrap()
                            .extend(to_remove.iter().map(Arc::clone));
                    }
                });
            }

            let count = wg.wait();

            let dominates_vector = dominates_vector
                .read()
                .unwrap()
                .iter()
                .enumerate()
                .filter(|(_, is_dominating)| **is_dominating)
                .map(|(idx, _)| new_individuals[idx])
                .collect::<Vec<&Phenotype<C>>>();
            let mut remove_vector = remove_vector.write().unwrap();

            remove_vector.dedup();

            ctx.front.clean(dominates_vector, remove_vector.as_slice());

            ctx.upsert_operation(metric_names::FRONT, count as f32, timer);
        }
    }

    fn start(&self) -> EngineContext<C, T> {
        let population = self.params.population().clone();

        EngineContext {
            // population: population.clone(),
            ecosystem: Ecosystem::new(population.clone()),
            best: self.params.problem().decode(population[0].genotype()),
            index: 0,
            timer: Timer::new(),
            metrics: MetricSet::new(),
            score: None,
            front: self.params.front().clone(),
        }
    }

    fn stop(&self, output: &mut EngineContext<C, T>) -> EngineContext<C, T> {
        output.timer.stop();
        output.clone()
    }
}

impl<C, T> Engine<C, T> for StandardEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    type Epoch = EngineContext<C, T>;

    fn next(&mut self) {}

    fn run<F>(&mut self, limit: F) -> Self::Epoch
    where
        F: Fn(&Self::Epoch) -> bool,
    {
        let mut ctx = self.start();

        loop {
            // self.next(&mut ctx);

            panic!("Not implemented yet");

            if limit(&ctx) {
                break self.stop(&mut ctx);
            }
        }
    }
}
