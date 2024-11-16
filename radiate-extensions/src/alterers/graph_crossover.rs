use std::collections::HashMap;

use radiate::engines::alterers::Alter;
use radiate::engines::genome::*;
use radiate::engines::optimize::Optimize;
use radiate::Alterer;

use crate::architects::node_collections::*;
use crate::architects::schema::node_types::NodeType;
use crate::operations::op::Ops;

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
    ) -> Alterer<Node<T>, Ops<T>> {
        Alterer::Alterer(Box::new(GraphCrossover::<T>::new(
            crossover_rate,
            crossover_parent_node_rate,
        )))
    }

    #[inline]
    pub fn cross(
        &self,
        population: &Population<Node<T>, Ops<T>>,
        indexes: &[usize],
        generation: i32,
    ) -> Option<Phenotype<Node<T>, Ops<T>>> {
        let parent_one = population.get(indexes[0]);
        let parent_two = population.get(indexes[1]);

        let geno_one = parent_one.genotype();
        let geno_two = parent_two.genotype();

        let chromo_index = rand::random::<usize>() % std::cmp::min(geno_one.len(), geno_two.len());

        let chromo_one = geno_one.get_chromosome(chromo_index);
        let chromo_two = geno_two.get_chromosome(chromo_index);

        let mut new_chromo_one = chromo_one.clone();
        let mut num_crosses = 0;

        for indentity in 0..std::cmp::min(chromo_one.len(), chromo_two.len()) {
            let node_one = chromo_one.get_gene(indentity);
            let node_two = chromo_two.get_gene(indentity);

            if node_one.node_type != NodeType::Weight || node_two.node_type != NodeType::Weight {
                continue;
            }

            if rand::random::<f32>() < self.crossover_parent_node_rate {
                new_chromo_one
                    .set_gene(*node_one.index(), node_one.from_allele(&node_two.allele()));
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
        let mut subset = Vec::new();

        while subset.len() < NUM_PARENTS {
            let index = rand::random::<usize>() % limit;
            if !subset.contains(&index) {
                subset.push(index);
            }
        }

        subset.sort();
        subset
    }
}

impl<T> Alter<Node<T>, Ops<T>> for GraphCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    #[inline]
    fn alter(
        &self,
        population: &mut Population<Node<T>, Ops<T>>,
        optimize: &Optimize,
        generation: i32,
    ) {
        optimize.sort(population);

        let mut new_phenotypes = HashMap::new();
        for index in 0..population.len() {
            if rand::random::<f32>() < self.crossover_rate && population.len() > NUM_PARENTS {
                let parent_indexes = GraphCrossover::<T>::distinct_subset(population.len());

                if let Some(phenotype) = self.cross(population, &parent_indexes, generation) {
                    new_phenotypes.insert(index, phenotype);
                }
            }
        }

        for (index, phenotype) in new_phenotypes.into_iter() {
            population.set(index, phenotype);
        }
    }
}
