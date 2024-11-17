use std::sync::Arc;

use crate::engines::alterers::alter::Alter;
use crate::engines::genetic_engine_params::GeneticEngineParams;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::population::Population;
use crate::engines::optimize::Optimize;
use crate::engines::schema::timer::Timer;
use crate::engines::score::Score;

use super::codexes::Codex;
use super::engine_context::EngineContext;
use super::genome::phenotype::Phenotype;
use super::selectors::select::Select;
use super::{MetricSet, ThreadPool, METRIC_AGE, METRIC_EVALUATE, METRIC_SCORE};

pub struct GeneticEngine<'a, G, A, T>
where
    G: Gene<G, A> + Send,
    A: Send,
    T: Clone + Send + 'static,
{
    pub params: GeneticEngineParams<'a, G, A, T>,
}

impl<'a, G, A, T> GeneticEngine<'a, G, A, T>
where
    G: Gene<G, A> + Send,
    A: Send,
    T: Clone + Send,
{
    pub fn new(params: GeneticEngineParams<'a, G, A, T>) -> Self {
        GeneticEngine { params }
    }

    pub fn from_codex(
        codex: &'a (impl Codex<G, A, T> + Send + Sync),
    ) -> GeneticEngineParams<G, A, T> {
        GeneticEngineParams::new().codex(codex)
    }

    pub fn run<F>(&self, limit: F) -> EngineContext<G, A, T>
    where
        F: Fn(&EngineContext<G, A, T>) -> bool,
    {
        let mut ctx = self.start();

        loop {
            self.evaluate(&mut ctx);

            let mut survivors = self.select_survivors(&ctx.population);
            let mut offspring = self.select_offspring(&ctx.population);

            self.alter(&mut offspring, ctx.index);

            self.filter(&mut survivors, ctx.index);
            self.filter(&mut offspring, ctx.index);

            self.recombine(&mut ctx, survivors, offspring);

            self.evaluate(&mut ctx);
            self.audit(&mut ctx);

            if limit(&ctx) {
                break self.stop(&mut ctx);
            }
        }
    }

    fn evaluate(&self, handle: &mut EngineContext<G, A, T>) {
        let codex = self.codex();
        let optimize = self.optimize();
        let thread_pool = self.thread_pool();
        let timer = Timer::new();

        let mut work_results = Vec::new();
        for idx in 0..handle.population.len() {
            let individual = handle.population.get(idx);
            if !individual.score().is_some() {
                let fitness_fn = self.fitness_fn();
                let decoded = codex.decode(individual.genotype());
                let work = thread_pool.process(move || (idx, fitness_fn(decoded)));

                work_results.push(work);
            }
        }

        handle.upsert_metric(
            METRIC_EVALUATE,
            work_results.len() as f32,
            Some(timer.duration()),
        );

        for work_result in work_results {
            let (idx, score) = work_result.result();
            handle.population.get_mut(idx).set_score(Some(score));
        }

        optimize.sort(&mut handle.population);
    }

    fn select_survivors(&self, population: &Population<G, A>) -> Population<G, A> {
        let selector = self.survivor_selector();
        let count = self.survivor_count();
        let optimize = self.optimize();

        selector.select(population, optimize, count)
    }

    fn select_offspring(&self, population: &Population<G, A>) -> Population<G, A> {
        let selector = self.offspring_selector();
        let count = self.offspring_count();
        let optimize = self.optimize();

        selector.select(population, optimize, count)
    }

    fn alter(&self, population: &mut Population<G, A>, generation: i32) {
        let alterer = self.alterer();
        let optimize = self.optimize();

        alterer.alter(population, optimize, generation);
    }

    fn filter(&self, population: &mut Population<G, A>, generation: i32) {
        let max_age = self.params.max_age;
        let codex = self.codex();

        for i in 0..population.len() {
            let phenotype = population.get(i);

            if phenotype.age(generation) > max_age {
                population.set(i, Phenotype::from_genotype(codex.encode(), generation));
            } else if !phenotype.genotype().is_valid() {
                population.set(i, Phenotype::from_genotype(codex.encode(), generation));
            }
        }
    }

    fn recombine(
        &self,
        handle: &mut EngineContext<G, A, T>,
        survivors: Population<G, A>,
        offspring: Population<G, A>,
    ) {
        handle.population = survivors
            .into_iter()
            .chain(offspring.into_iter())
            .collect::<Population<G, A>>();
    }

    fn audit(&self, output: &mut EngineContext<G, A, T>) {
        let codex = self.codex();
        let optimize = self.optimize();

        if !output.population.is_sorted {
            self.optimize().sort(&mut output.population);
        }

        if let Some(current_score) = &output.score {
            if let Some(best_score) = output.population.get(0).score() {
                if optimize.is_better(best_score, &current_score) {
                    output.score = Some(best_score.clone());
                    output.best = codex.decode(&output.population.get(0).genotype());
                }
            }
        } else {
            output.score = output.population.get(0).score().clone();
            output.best = codex.decode(&output.population.get(0).genotype());
        }

        self.add_metrics(output);

        output.index += 1;
    }

    fn add_metrics(&self, output: &mut EngineContext<G, A, T>) {
        for i in 0..output.population.len() {
            let phenotype = output.population.get(i);

            let age = phenotype.age(output.index);
            let score = phenotype.score().as_ref().unwrap().as_float();

            output.metrics.upsert(METRIC_AGE, age as f32);
            output.metrics.upsert(METRIC_SCORE, score);
        }
    }

    fn survivor_selector(&self) -> &dyn Select<G, A> {
        self.params.survivor_selector.as_ref()
    }

    fn offspring_selector(&self) -> &dyn Select<G, A> {
        self.params.offspring_selector.as_ref()
    }

    fn alterer(&self) -> &impl Alter<G, A> {
        self.params.alterer.as_ref().unwrap()
    }

    fn codex(&self) -> Arc<&'a (dyn Codex<G, A, T> + Send + Sync)> {
        Arc::clone(self.params.codex.as_ref().unwrap())
    }

    fn fitness_fn(&self) -> Arc<dyn Fn(T) -> Score + Send + Sync> {
        Arc::clone(self.params.fitness_fn.as_ref().unwrap())
    }

    fn population(&self) -> &Population<G, A> {
        self.params.population.as_ref().unwrap()
    }

    fn optimize(&self) -> &Optimize {
        &self.params.optimize
    }

    fn survivor_count(&self) -> usize {
        self.params.population_size - self.offspring_count()
    }

    fn offspring_count(&self) -> usize {
        (self.params.population_size as f32 * self.params.offspring_fraction) as usize
    }

    fn thread_pool(&self) -> &ThreadPool {
        &self.params.thread_pool
    }

    fn start(&self) -> EngineContext<G, A, T> {
        let population = self.population();

        EngineContext {
            population: population.clone(),
            best: self.codex().decode(&population.get(0).genotype()),
            index: 0,
            timer: Timer::new(),
            metrics: MetricSet::new(),
            score: None,
        }
    }

    fn stop(&self, output: &mut EngineContext<G, A, T>) -> EngineContext<G, A, T> {
        output.timer.stop();
        output.clone()
    }
}
