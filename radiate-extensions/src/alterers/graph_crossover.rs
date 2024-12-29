use std::collections::HashMap;

use crate::architects::schema::node_types::NodeType;
use crate::{GraphNode, NodeChrom, NodeChromosome};
use radiate::alter::AlterType;
use radiate::engines::alterers::Alter;
use radiate::engines::genome::*;
use radiate::timer::Timer;
use radiate::{random_provider, Metric};

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

    #[inline]
    pub fn cross(
        &self,
        population: &Population<NodeChrom<GraphNode<T>>>,
        indexes: &[usize],
        generation: i32,
    ) -> Option<Phenotype<NodeChrom<GraphNode<T>>>> {
        let parent_one = &population[indexes[0]];
        let parent_two = &population[indexes[1]];

        let geno_one = parent_one.genotype();
        let geno_two = parent_two.genotype();

        let chromo_index =
            random_provider::random::<usize>() % std::cmp::min(geno_one.len(), geno_two.len());

        let chromo_one = &geno_one[chromo_index];
        let chromo_two = &geno_two[chromo_index];

        let mut new_chromo_one = chromo_one.clone();
        let mut num_crosses = 0;

        for i in 0..std::cmp::min(chromo_one.len(), chromo_two.len()) {
            let node_one = chromo_one.get_gene(i);
            let node_two = chromo_two.get_gene(i);

            if node_one.cell.value.name() != "w" {
                continue;
            }

            if node_two.cell.value.name() != "w" {
                continue;
            }

            // if node_one.cell.value ==

            // if node_one.cell.value.arity() != node_two.cell.value.arity() {
            //     continue;
            // }

            if random_provider::random::<f32>() < self.crossover_parent_node_rate {
                new_chromo_one.set_gene(node_one.index, node_one.with_allele(node_two.allele()));
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
            let index = random_provider::random::<usize>() % limit;
            if !subset.contains(&index) {
                subset.push(index);
            }
        }

        subset.sort();
        subset
    }
}

impl<T> Alter<NodeChrom<GraphNode<T>>> for GraphCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    fn name(&self) -> &'static str {
        "GraphCrossover"
    }

    fn rate(&self) -> f32 {
        self.crossover_rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Alterer
    }

    #[inline]
    fn alter(
        &self,
        population: &mut Population<NodeChrom<GraphNode<T>>>,
        generation: i32,
    ) -> Vec<Metric> {
        let timer = Timer::new();
        let mut count = 0;
        let mut new_phenotypes = HashMap::new();
        for index in 0..population.len() {
            if random_provider::random::<f32>() < self.crossover_rate
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
            population[index] = phenotype;
        }

        let mut metric = Metric::new_operations("Graph Crossover");
        metric.add_value(count as f32);
        metric.add_duration(timer.duration());

        vec![metric]
    }
}
