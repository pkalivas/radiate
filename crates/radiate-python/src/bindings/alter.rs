// 8/28

use crate::{PyChromosome, PyGenotype};
use pyo3::{Py, PyAny, Python};
use radiate::{
    AlterResult, Chromosome, Crossover, Genotype, Mutate, Population, indexes, random_provider,
};
use std::{collections::HashMap, sync::Arc};

pub struct PyCrossover<C: Chromosome> {
    pub rate: f32,
    pub name: String,
    pub chromosome_crossover: Arc<Py<PyAny>>,
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome> PyCrossover<C> {
    pub fn new(rate: f32, name: String, crossover_func: Py<PyAny>) -> Self {
        PyCrossover {
            rate,
            name,
            chromosome_crossover: Arc::new(crossover_func),
            _marker: std::marker::PhantomData,
        }
    }

    fn crossover_with_cache(
        &self,
        population: &mut Population<C>,
        index_pairs: &[Vec<usize>],
        generation: usize,
    ) -> AlterResult
    where
        C: Chromosome + Clone + From<PyChromosome>,
        Genotype<C>: From<PyGenotype>,
        PyChromosome: From<C>,
        PyGenotype: From<Genotype<C>>,
    {
        let mut result = AlterResult::default();
        let mut genotype_cache = HashMap::new();

        for pair in index_pairs.iter() {
            if !genotype_cache.contains_key(&pair[0]) {
                if let Some(individual) = population.get_mut(pair[0]) {
                    genotype_cache.insert(pair[0], PyGenotype::from(individual.take_genotype()));
                }
            }

            if !genotype_cache.contains_key(&pair[1]) {
                if let Some(individual) = population.get_mut(pair[1]) {
                    genotype_cache.insert(pair[1], PyGenotype::from(individual.take_genotype()));
                }
            }

            let geno_vec = genotype_cache.get_disjoint_mut([&pair[0], &pair[1]]);

            let [geno_one, geno_two] = match geno_vec {
                [Some(g1), Some(g2)] => [g1, g2],
                _ => continue,
            };

            let min_len = std::cmp::min(geno_one.len(), geno_two.len());
            let chromosome_index = random_provider::range(0..min_len);

            let py_chrom_one = &mut geno_one.chromosomes[chromosome_index];
            let py_chrom_two = &mut geno_two.chromosomes[chromosome_index];

            let mut count = 0;
            Python::attach(|py| {
                let result = self
                    .chromosome_crossover
                    .call1(py, (py_chrom_one.clone(), py_chrom_two.clone()))
                    .expect("crossover function should not fail");
                let (mutated_one, mutated_two) = result
                    .extract::<(PyChromosome, PyChromosome)>(py)
                    .expect("should return a tuple of PyChromosome");
                *py_chrom_one = mutated_one;
                *py_chrom_two = mutated_two;

                count += 1;
            });

            result.0 += count;
        }

        for (key, val) in genotype_cache {
            if let Some(individual) = population.get_mut(key) {
                individual.set_genotype(val.into());
                individual.invalidate(generation);
            }
        }

        result
    }
}

impl<C> Crossover<C> for PyCrossover<C>
where
    C: Chromosome + Clone + Send + Sync + From<PyChromosome>,
    Genotype<C>: From<PyGenotype>,
    PyChromosome: From<C>,
    PyGenotype: From<Genotype<C>>,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn crossover(
        &self,
        population: &mut Population<C>,
        generation: usize,
        rate: f32,
    ) -> AlterResult {
        let mut pairs = Vec::with_capacity(population.len());

        for i in 0..population.len() {
            if random_provider::random::<f32>() < rate && population.len() > 3 {
                let parent_indexes = indexes::individual_indexes(i, population.len(), 2);
                pairs.push(parent_indexes);
            }
        }

        self.crossover_with_cache(population, &pairs, generation)
    }
}

pub struct PyMutator<C: Chromosome> {
    pub rate: f32,
    pub name: String,
    pub chromosome_mutator: Py<PyAny>,
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome> PyMutator<C> {
    pub fn new(rate: f32, name: String, mutate_func: Py<PyAny>) -> Self {
        PyMutator {
            rate,
            name,
            chromosome_mutator: mutate_func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C> Mutate<C> for PyMutator<C>
where
    C: Chromosome + Clone + Send + Sync + From<PyChromosome>,
    PyChromosome: From<C>,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;

        if random_provider::random::<f32>() < rate {
            Python::attach(|py| {
                let py_chromosome = PyChromosome::from(chromosome.clone());
                let result = self
                    .chromosome_mutator
                    .call1(py, (py_chromosome,))
                    .expect("mutation function should not fail");
                let mutated = result
                    .extract::<PyChromosome>(py)
                    .expect("should return a PyChromosome");
                *chromosome = mutated.into();
            });

            count += 1;
        }

        count.into()
    }
}
