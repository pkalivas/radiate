use std::collections::HashMap;

use radiate::engines::alterers::Alter;
use radiate::engines::genome::*;
use radiate::engines::optimize::Optimize;
use radiate::{Alterer, Metric, RandomProvider, Timer};

use crate::architects::schema::node_types::NodeType;
use crate::node::Node;
use crate::operations::op::Ops;
use crate::NodeChromosome;

const NUM_PARENTS: usize = 2;

pub struct GraphCrossover<T>
where
    T: Clone + PartialEq + Default,
{
    pub crossover_rate: f32,
    pub crossover_parent_node_rate: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> GraphCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub fn new(crossover_rate: f32, crossover_parent_node_rate: f32) -> Self {
        Self {
            crossover_rate,
            crossover_parent_node_rate,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn alterer(
        crossover_rate: f32,
        crossover_parent_node_rate: f32,
    ) -> Alterer<NodeChromosome<T>> {
        Alterer::Alterer(Box::new(GraphCrossover::<T>::new(
            crossover_rate,
            crossover_parent_node_rate,
        )))
    }

    #[inline]
    pub fn cross(
        &self,
        population: &Population<NodeChromosome<T>>,
        indexes: &[usize],
        generation: i32,
    ) -> Option<Phenotype<NodeChromosome<T>>> {
        let parent_one = population.get(indexes[0]);
        let parent_two = population.get(indexes[1]);

        let geno_one = parent_one.genotype();
        let geno_two = parent_two.genotype();

        let chromo_index =
            RandomProvider::random::<usize>() % std::cmp::min(geno_one.len(), geno_two.len());

        let chromo_one = geno_one.get_chromosome(chromo_index);
        let chromo_two = geno_two.get_chromosome(chromo_index);

        let mut new_chromo_one = chromo_one.clone();
        let mut num_crosses = 0;

        for i in 0..std::cmp::min(chromo_one.len(), chromo_two.len()) {
            let node_one = chromo_one.get_gene(i);
            let node_two = chromo_two.get_gene(i);

            if node_one.node_type != NodeType::Weight || node_two.node_type != NodeType::Weight {
                continue;
            }

            if RandomProvider::random::<f32>() < self.crossover_parent_node_rate {
                new_chromo_one.set_gene(node_one.index, node_one.from_allele(node_two.allele()));
                num_crosses += 1;
            }
        }

        if num_crosses > 0 {
            let new_genotype_one = Genotype {
                chromosomes: vec![new_chromo_one],
            };
            let new_phenotype = Phenotype::from_genotype(new_genotype_one, generation);

            return Some(new_phenotype);
        }

        None
    }

    pub fn distinct_subset(limit: usize) -> Vec<usize> {
        let mut subset = Vec::with_capacity(NUM_PARENTS);

        while subset.len() < NUM_PARENTS {
            let index = RandomProvider::random::<usize>() % limit;
            if !subset.contains(&index) {
                subset.push(index);
            }
        }

        subset.sort();
        subset
    }
}

impl<T> Alter<NodeChromosome<T>> for GraphCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    #[inline]
    fn alter(
        &self,
        population: &mut Population<NodeChromosome<T>>,
        optimize: &Optimize,
        generation: i32,
    ) -> Vec<Metric> {
        optimize.sort(population);

        let timer = Timer::new();
        let mut count = 0;
        let mut new_phenotypes = HashMap::new();
        for index in 0..population.len() {
            if RandomProvider::random::<f32>() < self.crossover_rate
                && population.len() > NUM_PARENTS
            {
                let parent_indexes = GraphCrossover::<T>::distinct_subset(population.len());

                if let Some(phenotype) = self.cross(population, &parent_indexes, generation) {
                    new_phenotypes.insert(index, phenotype);
                    count += 1;
                }
            }
        }

        for (index, phenotype) in new_phenotypes.into_iter() {
            population.set(index, phenotype);
        }

        let mut metric = Metric::new("Graph Crossover");
        metric.add(count as f32, timer.duration());

        vec![metric]
    }
}
