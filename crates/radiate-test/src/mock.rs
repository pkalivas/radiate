#![allow(dead_code)]

use radiate_alters::{BlendCrossover, GaussianMutator, UniformCrossover, UniformMutator};
use radiate_core::{
    Alterer, BitChromosome, Chromosome, Codec, Crossover, Ecosystem, Executor, FloatChromosome,
    FloatCodec, Gene, Genotype, IntChromosome, Mutate, Objective, Optimize, Phenotype, Population,
    Rate, Score, Species, alters, diversity::Diversity, random_provider,
};
use radiate_engines::{OffspringConfig, RecombineStep, SelectConfig, SpeciateStep, SurvivorConfig};
use radiate_selectors::{BoltzmannSelector, TournamentSelector};
use std::{
    ops::Range,
    sync::{Arc, RwLock},
};

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
    score_fn: Option<Box<dyn Fn(usize, &Genotype<C>) -> Score>>,
    species_sizes: Option<Vec<usize>>,
    genotype_fn: Box<dyn Fn() -> Genotype<C>>,
}

impl<C: Chromosome + Clone> MockEcosystemBuilder<C> {
    pub fn pop_size(mut self, n: usize) -> Self {
        self.pop_size = n;
        self
    }

    /// Set the per-phenotype score from index *and* genotype. Pass a
    /// closure reading the genotype when score must be derived from gene
    /// alleles (e.g. selector tests whose downstream metric reads gene
    /// values and must agree with the score order).
    pub fn scores(mut self, score_fn: impl Fn(usize, &Genotype<C>) -> Score + 'static) -> Self {
        self.score_fn = Some(Box::new(score_fn));
        self
    }

    pub fn scores_linear(self) -> Self {
        self.scores(|i, _| Score::from(i as f32))
    }

    pub fn scores_uniform(self, value: f32) -> Self {
        self.scores(move |_, _| Score::from(value))
    }

    pub fn scores_random(self, range: Range<f32>) -> Self {
        self.scores(move |_, _| {
            let allele = random_provider::range(range.clone());
            Score::from(allele)
        })
    }

    pub fn with_species(mut self, sizes: &[usize]) -> Self {
        self.species_sizes = Some(sizes.to_vec());
        self
    }

    pub fn build_population(self) -> Population<C> {
        Population::new(self.build_phenotypes())
    }

    fn build_phenotypes(&self) -> Vec<Phenotype<C>> {
        (0..self.pop_size)
            .map(|i| {
                let geno = (self.genotype_fn)();
                let mut p = Phenotype::from((geno.clone(), 0));
                if let Some(score_fn) = &self.score_fn {
                    p.set_score(Some(score_fn(i, &geno)));
                }
                p
            })
            .collect()
    }

    pub fn build(self) -> Ecosystem<C> {
        let mut phenotypes = (0..self.pop_size)
            .map(|i| {
                let geno = (self.genotype_fn)();
                let mut p = Phenotype::from((geno.clone(), 0));
                if let Some(score_fn) = &self.score_fn {
                    p.set_score(Some(score_fn(i, &geno)));
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
                let species = Species::new(0, mascot);
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

    RecombineStep::new(survivor, offspring, objective)
}

pub fn default_float_alters() -> Vec<Alterer<FloatChromosome<f32>>> {
    alters![BlendCrossover::new(0.5, 0.5), GaussianMutator::new(0.1)]
}

pub fn default_int_alters() -> Vec<Alterer<IntChromosome<i32>>> {
    alters![UniformCrossover::new(0.7), UniformMutator::new(0.05)]
}

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

pub fn float_population(num: usize) -> Population<FloatChromosome<f32>> {
    MockEcosystem::new(FloatCodec::vector(1, 0.0..1.0))
        .pop_size(num)
        .scores_linear()
        .build_population()
}

pub fn random_float_population(num: usize) -> Population<FloatChromosome<f32>> {
    MockEcosystem::new(FloatCodec::vector(1, 0.0..100.0))
        .pop_size(num)
        .scores(|_, g| Score::from(*g[0].as_slice()[0].allele()))
        .build_population()
}

pub fn multi_obj_population(scores: Vec<Vec<f32>>) -> Population<FloatChromosome<f32>> {
    let n = scores.len();
    MockEcosystem::new(FloatCodec::vector(1, 0.0..1.0))
        .pop_size(n)
        .scores(move |i, _| Score::from(scores[i].clone()))
        .build_population()
}
