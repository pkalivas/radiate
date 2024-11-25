use crate::engines::alterers::alter::Alter;
use crate::engines::genome::population::Population;
use crate::engines::optimize::Optimize;
use crate::engines::schema::subset;
use crate::{Chromosome, Metric, RandomProvider, Timer};

use super::alter::{AlterWrap, Alterer};
use super::crossovers::multipoint_crossover::MultiPointCrossover;
use super::crossovers::uniform_crossover::UniformCrossover;
use super::mutators::mutator::Mutator;
use super::mutators::swap_mutator::SwapMutator;

pub struct CompositeAlterer<C: Chromosome> {
    alterers: Vec<AlterWrap<C>>,
}

impl<C: Chromosome> CompositeAlterer<C> {
    pub fn new(alterers: Vec<Alterer<C>>) -> Self {
        let mut alterer_wraps = Vec::new();
        for alterer in alterers {
            match alterer {
                Alterer::Mutator(rate) => {
                    let mutator = Box::new(Mutator::new(rate));
                    alterer_wraps.push(AlterWrap::from_mutator(mutator, rate))
                }
                Alterer::UniformCrossover(rate) => {
                    let crossover = Box::new(UniformCrossover::new(rate));
                    alterer_wraps.push(AlterWrap::from_crossover(crossover, rate))
                }
                Alterer::SinglePointCrossover(rate) => {
                    let crossover = Box::new(MultiPointCrossover::new(rate, 1));
                    alterer_wraps.push(AlterWrap::from_crossover(crossover, rate))
                }
                Alterer::MultiPointCrossover(rate, num_points) => {
                    let crossover = Box::new(MultiPointCrossover::new(rate, num_points));
                    alterer_wraps.push(AlterWrap::from_crossover(crossover, rate))
                }
                Alterer::SwapMutator(rate) => {
                    let mutator = Box::new(SwapMutator::new(rate));
                    alterer_wraps.push(AlterWrap::from_mutator(mutator, rate))
                }
                Alterer::Mutation(mutation) => {
                    let rate = mutation.mutate_rate();
                    alterer_wraps.push(AlterWrap::from_mutator(mutation, rate))
                }
                Alterer::Crossover(crossover) => {
                    let cross_rate = crossover.cross_rate();
                    alterer_wraps.push(AlterWrap::from_crossover(crossover, cross_rate))
                }
                Alterer::Alterer(alterer) => {
                    alterer_wraps.push(AlterWrap::from_alterer(alterer, 1.0))
                }
            }
        }

        CompositeAlterer {
            alterers: alterer_wraps,
        }
    }
}

impl<C: Chromosome> Alter<C> for CompositeAlterer<C> {
    #[inline]
    fn alter(
        &self,
        population: &mut Population<C>,
        optimize: &Optimize,
        generation: i32,
    ) -> Vec<Metric> {
        optimize.sort(population);

        let mut metrics = Vec::new();
        for alterer in self.alterers.iter() {
            let timer = Timer::new();
            let mut count = 0;

            match alterer.mutator {
                Some(ref mutator) => {
                    let probability = alterer.rate.powf(1.0 / 3.0);
                    let range = ((((std::i32::MAX as i64 - (std::i32::MIN as i64)) as f32)
                        * probability)
                        + (std::i32::MIN as f32)) as i32;

                    for phenotype in population.iter_mut() {
                        if RandomProvider::random::<i32>() > range {
                            let genotype = phenotype.genotype_mut();

                            let mutation_count = mutator.mutate_genotype(genotype, range);

                            if mutation_count > 0 {
                                (*phenotype).generation = generation;
                                (*phenotype).score = None;
                                count += mutation_count;
                            }
                        }
                    }

                    let mut mutate_metric = Metric::new(mutator.name());
                    mutate_metric.add(count as f32, timer.duration());
                    metrics.push(mutate_metric);
                }
                None => (),
            };
            match alterer.crossover {
                Some(ref crossover) => {
                    for i in 0..population.len() {
                        if RandomProvider::random::<f32>() < alterer.rate {
                            let parent_indexes = subset::individual_indexes(i, population.len(), 2);
                            count += crossover.cross(population, &parent_indexes, generation);
                        }
                    }

                    let mut cross_metric = Metric::new(crossover.name());
                    cross_metric.add(count as f32, timer.duration());
                    metrics.push(cross_metric);
                }
                None => (),
            };
            match alterer.alterer {
                Some(ref alterer) => {
                    let alter_metrics = alterer.alter(population, optimize, generation);
                    for metric in alter_metrics {
                        metrics.push(metric);
                    }
                }
                None => (),
            };
        }

        metrics
    }
}
