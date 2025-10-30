use crate::{BitChromosome, BitGene, Chromosome, Gene, Genotype, Population, random_provider};

pub trait ProbabilisticModel<C: Chromosome>: Send + Sync {
    fn sample(&self, n: usize) -> Vec<Genotype<C>>;
}

pub trait ModelLearner<C: Chromosome>: Send + Sync {
    fn fit(&self, parents: &Population<C>) -> Box<dyn ProbabilisticModel<C>>;
}

pub struct UmdaBitLearner {
    pub alpha: f64,
}

impl UmdaBitLearner {
    pub fn new(alpha: f64) -> Self {
        UmdaBitLearner { alpha }
    }
}

pub struct UmdaBitModel {
    pub probs: Vec<Vec<f64>>,
}

impl ModelLearner<BitChromosome> for UmdaBitLearner {
    fn fit(
        &self,
        parents: &Population<BitChromosome>,
    ) -> Box<dyn ProbabilisticModel<BitChromosome>> {
        let mut ones = Vec::new();
        let mut zeros = Vec::new();

        for ind in parents.iter() {
            for (c_idx, chromosome) in ind.genotype().iter().enumerate() {
                if ones.len() <= c_idx {
                    ones.push(vec![self.alpha; chromosome.len()]);
                    zeros.push(vec![self.alpha; chromosome.len()]);
                }

                for (g_idx, gene) in chromosome.iter().enumerate() {
                    if *gene.allele() {
                        ones[c_idx][g_idx] += 1.0;
                    } else {
                        zeros[c_idx][g_idx] += 1.0;
                    }
                }
            }
        }

        Box::new(UmdaBitModel {
            probs: ones
                .iter()
                .zip(zeros.iter())
                .map(|(one_vec, zero_vec)| {
                    one_vec
                        .iter()
                        .zip(zero_vec.iter())
                        .map(|(a, b)| a / (a + b))
                        .collect()
                })
                .collect(),
        })
    }
}

impl ProbabilisticModel<BitChromosome> for UmdaBitModel {
    fn sample(&self, n: usize) -> Vec<Genotype<BitChromosome>> {
        let mut genotypes = Vec::with_capacity(n);
        for _ in 0..n {
            genotypes.push(
                self.probs
                    .iter()
                    .map(|p_vec| {
                        p_vec
                            .iter()
                            .map(|&p| BitGene::from(random_provider::random::<f64>() < p))
                            .collect::<BitChromosome>()
                    })
                    .collect::<Genotype<BitChromosome>>(),
            );
        }

        genotypes
    }
}
