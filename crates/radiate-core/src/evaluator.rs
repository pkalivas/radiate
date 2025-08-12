use crate::{Chromosome, Ecosystem, Executor, Genotype, Problem};
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

struct Batch<C: Chromosome> {
    indices: Vec<usize>,
    genotypes: Vec<Genotype<C>>,
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
        let mut pairs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype();
                pairs.push((idx, geno));
            }
        }

        // integer ceiling division to determine batch size - number of individuals per batch
        let num_workers = self.executor.num_workers();
        let batch_size = (pairs.len() + num_workers - 1) / num_workers;

        if pairs.is_empty() || batch_size == 0 {
            return 0;
        }

        let mut batches = Vec::with_capacity(num_workers);

        while !pairs.is_empty() {
            let take = pairs.len().min(batch_size);

            let mut batch_indices = Vec::with_capacity(take);
            let mut batch_genotypes = Vec::with_capacity(take);

            // drain from the end of pairs vector to avoid O(n^2) complexity
            for (idx, geno) in pairs.drain(pairs.len() - take..) {
                batch_indices.push(idx);
                batch_genotypes.push(geno);
            }

            batches.push(Batch {
                indices: batch_indices,
                genotypes: batch_genotypes,
            });
        }

        let results = self.executor.execute_batch(
            batches
                .into_iter()
                .map(|batch| {
                    let problem = Arc::clone(&problem);
                    move || {
                        let scores = problem.eval_batch(&batch.genotypes);
                        (batch.indices, scores, batch.genotypes)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let mut count = 0;

        // replace the genotypes and add their associated scores
        for (indices, scores, genotypes) in results {
            count += indices.len();
            let score_genotype_iter = scores.into_iter().zip(genotypes.into_iter());
            for (i, (score, genotype)) in score_genotype_iter.enumerate() {
                let idx = indices[i];
                ecosystem.population[idx].set_score(Some(score));
                ecosystem.population[idx].set_genotype(genotype);
            }
        }

        count
    }
}
