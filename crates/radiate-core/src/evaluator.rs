use crate::{Chromosome, Ecosystem, Executor, Problem};
use std::sync::Arc;

pub trait Evaluator<C: Chromosome, T>: Send + Sync {
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> usize;
}

pub struct FitnessEvaluator {
    executor: Arc<Executor>,
}

impl FitnessEvaluator {
    pub fn new(executor: Arc<Executor>) -> Self {
        Self { executor }
    }
}

impl<C: Chromosome, T> Evaluator<C, T> for FitnessEvaluator
where
    C: Chromosome + 'static,
    T: 'static,
{
    #[inline]
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> usize {
        let mut jobs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype();
                jobs.push((idx, geno));
            }
        }

        let results = self.executor.execute_batch(
            jobs.into_iter()
                .map(|(idx, geno)| {
                    let problem = Arc::clone(&problem);
                    move || {
                        let score = problem.eval(&geno);
                        (idx, score, geno)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let count = results.len();
        for result in results {
            let (idx, score, genotype) = result;
            ecosystem.population[idx].set_score(Some(score));
            ecosystem.population[idx].set_genotype(genotype);
        }

        count
    }
}

impl Default for FitnessEvaluator {
    fn default() -> Self {
        Self {
            executor: Arc::new(Executor::Serial),
        }
    }
}

pub struct BatchFitnessEvaluator {
    executor: Arc<Executor>,
}

impl BatchFitnessEvaluator {
    pub fn new(executor: Arc<Executor>) -> Self {
        Self { executor }
    }
}

impl<C: Chromosome, T> Evaluator<C, T> for BatchFitnessEvaluator
where
    C: Chromosome + 'static,
    T: 'static,
{
    #[inline]
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> usize {
        let mut genotypes = Vec::new();
        let mut indices = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype();
                genotypes.push(geno);
                indices.push(idx);
            }
        }

        if genotypes.is_empty() {
            return 0;
        }

        let cloned_problem = Arc::clone(&problem);
        let results = self.executor.execute(move || {
            let scores = cloned_problem.eval_batch(&genotypes);
            (scores, genotypes)
        });

        let (scores, genotypes) = results;
        let count = scores.len();
        let score_genotype_iter = scores.into_iter().zip(genotypes.into_iter());
        for (i, (score, genotype)) in score_genotype_iter.enumerate() {
            let idx = indices[i];
            ecosystem.population[idx].set_score(Some(score));
            ecosystem.population[idx].set_genotype(genotype);
        }

        count
    }
}
