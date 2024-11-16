use crate::engines::alterers::alter::Alter;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::population::Population;
use crate::engines::optimize::Optimize;
use crate::engines::schema::subset;

use super::alter::{AlterWrap, Alterer};
use super::crossovers::multipoint_crossover::MultiPointCrossover;
use super::crossovers::uniform_crossover::UniformCrossover;
use super::mutators::mutator::Mutator;
use super::mutators::swap_mutator::SwapMutator;

pub struct CompositeAlterer<G, A>
where
    G: Gene<G, A>,
{
    alterers: Vec<AlterWrap<G, A>>,
}

impl<G, A> CompositeAlterer<G, A>
where
    G: Gene<G, A>,
{
    pub fn new(alterers: Vec<Alterer<G, A>>) -> Self {
        let mut alterer_wraps = Vec::new();
        for alterer in alterers {
            match alterer {
                Alterer::Mutator(rate) => {
                    alterer_wraps.push(AlterWrap {
                        rate,
                        mutator: Some(Box::new(Mutator::new(rate))),
                        crossover: None,
                        alterer: None,
                    });
                }
                Alterer::UniformCrossover(rate) => {
                    alterer_wraps.push(AlterWrap {
                        rate,
                        mutator: None,
                        crossover: Some(Box::new(UniformCrossover::new(rate))),
                        alterer: None,
                    });
                }
                Alterer::SinglePointCrossover(rate) => {
                    alterer_wraps.push(AlterWrap {
                        rate,
                        mutator: None,
                        crossover: Some(Box::new(MultiPointCrossover::new(rate, 1))),
                        alterer: None,
                    });
                }
                Alterer::MultiPointCrossover(rate, num_points) => {
                    alterer_wraps.push(AlterWrap {
                        rate,
                        mutator: None,
                        crossover: Some(Box::new(MultiPointCrossover::new(rate, num_points))),
                        alterer: None,
                    });
                }
                Alterer::SwapMutator(rate) => {
                    alterer_wraps.push(AlterWrap {
                        rate,
                        mutator: Some(Box::new(SwapMutator::new(rate))),
                        crossover: None,
                        alterer: None,
                    });
                }
                Alterer::Mutation(mutation) => {
                    alterer_wraps.push(AlterWrap {
                        rate: mutation.mutate_rate(),
                        mutator: Some(mutation),
                        crossover: None,
                        alterer: None,
                    });
                }
                Alterer::Crossover(crossover) => {
                    alterer_wraps.push(AlterWrap {
                        rate: crossover.cross_rate(),
                        mutator: None,
                        crossover: Some(crossover),
                        alterer: None,
                    });
                }
                Alterer::Alterer(alterer) => {
                    alterer_wraps.push(AlterWrap {
                        rate: 1.0,
                        mutator: None,
                        crossover: None,
                        alterer: Some(alterer),
                    });
                }
            }
        }

        CompositeAlterer {
            alterers: alterer_wraps,
        }
    }
}

impl<G, A> Alter<G, A> for CompositeAlterer<G, A>
where
    G: Gene<G, A>,
{
    #[inline]
    fn alter(&self, population: &mut Population<G, A>, optimize: &Optimize, generation: i32) {
        optimize.sort(population);

        for alterer in self.alterers.iter() {
            match alterer.mutator {
                Some(ref mutator) => {
                    let probability = alterer.rate.powf(1.0 / 3.0);
                    let range = ((((std::i32::MAX as i64 - (std::i32::MIN as i64)) as f32)
                        * probability)
                        + (std::i32::MIN as f32)) as i32;

                    for phenotype in population.iter_mut() {
                        if rand::random::<i32>() > range {
                            let mut genotype = phenotype.genotype_mut();

                            let mutation_count = mutator.mutate_genotype(&mut genotype, range);

                            if mutation_count > 0 {
                                (*phenotype).generation = generation;
                                (*phenotype).score = None;
                            }
                        }
                    }
                }
                None => (),
            };
            match alterer.crossover {
                Some(ref crossover) => {
                    let mut random = rand::thread_rng();

                    for i in 0..population.len() {
                        if rand::random::<f32>() < alterer.rate {
                            let parent_indexes =
                                subset::individual_indexes(&mut random, i, population.len(), 2);
                            crossover.cross(population, &parent_indexes, generation);
                        }
                    }
                }
                None => (),
            };
            match alterer.alterer {
                Some(ref alterer) => {
                    alterer.alter(population, optimize, generation);
                }
                None => (),
            };
        }
    }
}
