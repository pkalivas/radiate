#![allow(dead_code)]

use radiate_alters::{BlendCrossover, GaussianMutator, UniformCrossover, UniformMutator};
use radiate_core::{
    Alterer, BitChromosome, Chromosome, Codec, Crossover, Ecosystem, Executor, FloatChromosome,
    FloatCodec, Genotype, IntChromosome, Lineage, Mutate, Objective, Optimize, Phenotype,
    Population, Rate, Score, Species, alters, diversity::Diversity,
};
use radiate_engines::{OffspringConfig, RecombineStep, SelectConfig, SpeciateStep, SurvivorConfig};
use radiate_selectors::{BoltzmannSelector, TournamentSelector};
use std::sync::{Arc, RwLock};

/// Builder for a deterministic mock ecosystem. Each builder call decides
/// one axis of the test scenario; the final `.build()` returns an
/// `Ecosystem<C>` ready to feed to a step.
///
/// Example:
/// ```ignore
/// let eco = MockEcosystem::new(FloatCodec::vector(2, 0.0..1.0))
///     .pop_size(20)
///     .scores_linear()              // pop[i].score() = i as f32
///     .with_species(&[10, 6, 4])    // 3 species of those sizes
///     .build();
/// ```
pub struct MockEcosystem<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome + Clone> MockEcosystem<C> {
    pub fn new<T>(codec: impl Codec<C, T> + 'static) -> MockEcosystemBuilder<C> {
        MockEcosystemBuilder {
            pop_size: 0,
            score_fn: None,
            species_sizes: None,
            genotype_fn: Box::new(move || codec.encode()),
        }
    }
}

pub struct MockEcosystemBuilder<C: Chromosome> {
    pop_size: usize,
    score_fn: Option<Box<dyn Fn(usize) -> Score>>,
    species_sizes: Option<Vec<usize>>,
    genotype_fn: Box<dyn Fn() -> Genotype<C>>,
}

impl<C: Chromosome + Clone> MockEcosystemBuilder<C> {
    pub fn pop_size(mut self, n: usize) -> Self {
        self.pop_size = n;
        self
    }

    pub fn scores(mut self, score_fn: impl Fn(usize) -> Score + 'static) -> Self {
        self.score_fn = Some(Box::new(score_fn));
        self
    }

    pub fn scores_linear(self) -> Self {
        self.scores(|i| Score::from(i as f32))
    }

    pub fn scores_uniform(self, value: f32) -> Self {
        self.scores(move |_| Score::from(value))
    }

    pub fn with_species(mut self, sizes: &[usize]) -> Self {
        self.species_sizes = Some(sizes.to_vec());
        self
    }

    pub fn build(self) -> Ecosystem<C> {
        let mut phenotypes = (0..self.pop_size)
            .map(|i| {
                let mut p = Phenotype::from(((self.genotype_fn)(), 0));
                if let Some(score_fn) = &self.score_fn {
                    p.set_score(Some(score_fn(i)));
                }
                p
            })
            .collect::<Vec<_>>();

        let species_list = self.species_sizes.map(|sizes| {
            assert_eq!(
                sizes.iter().sum::<usize>(),
                self.pop_size,
                "species sizes must sum to pop_size",
            );

            let mut species_vec = Vec::with_capacity(sizes.len());
            let mut start = 0usize;
            for &size in &sizes {
                if size == 0 {
                    continue;
                }
                let mascot = phenotypes[start].clone();
                let species = Species::new(0, &mascot);
                let species_id = species.id();
                for p in &mut phenotypes[start..start + size] {
                    p.set_species(species_id);
                }
                species_vec.push(species);
                start += size;
            }
            species_vec
        });

        let mut eco = Ecosystem::new(Population::new(phenotypes));
        if let Some(species_vec) = species_list {
            for s in species_vec {
                eco.push_species(s);
            }
        }
        eco
    }
}

pub fn mock_recombine_step<C: Chromosome + Clone>(
    survivor_count: usize,
    offspring_count: usize,
    objective: Objective,
    alters: Vec<Alterer<C>>,
) -> RecombineStep<C> {
    let survivor = SurvivorConfig::new(SelectConfig::new(
        survivor_count,
        Arc::new(TournamentSelector::new(3)),
        ("test.survivor", "test.survivor.time"),
    ));
    let offspring = OffspringConfig::new(
        SelectConfig::new(
            offspring_count,
            Arc::new(BoltzmannSelector::new(4.0)),
            ("test.offspring", "test.offspring.time"),
        ),
        alters,
    );

    RecombineStep::new(
        survivor,
        offspring,
        objective,
        Arc::new(RwLock::new(Lineage::default())),
    )
}

pub fn default_float_alters() -> Vec<Alterer<FloatChromosome<f32>>> {
    alters![BlendCrossover::new(0.5, 0.5), GaussianMutator::new(0.1)]
}

/// Stock alters for `IntChromosome<i32>` tests.
pub fn default_int_alters() -> Vec<Alterer<IntChromosome<i32>>> {
    alters![UniformCrossover::new(0.7), UniformMutator::new(0.05)]
}

/// Stock alters for `BitChromosome` tests.
pub fn default_bit_alters() -> Vec<Alterer<BitChromosome>> {
    alters![UniformCrossover::new(0.7), UniformMutator::new(0.05)]
}

pub fn mock_speciate_step<C: Chromosome>(
    threshold: f32,
    distance: impl Diversity<C> + 'static,
) -> SpeciateStep<C> {
    SpeciateStep::new(
        Rate::from(threshold),
        minimize(),
        Arc::new(distance),
        Arc::new(Executor::Serial),
    )
}

pub fn minimize() -> Objective {
    Objective::Single(Optimize::Minimize)
}

pub fn maximize() -> Objective {
    Objective::Single(Optimize::Maximize)
}
