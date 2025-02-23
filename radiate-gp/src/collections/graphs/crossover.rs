use crate::NodeType;
use crate::collections::GraphChromosome;
use crate::node::Node;
use radiate::engines::genome::*;
use radiate::timer::Timer;
use radiate::{Alter, AlterAction, Crossover, EngineCompoment, Metric, indexes, random_provider};
use std::collections::HashMap;

const NUM_PARENTS: usize = 2;

pub struct GraphCrossover {
    crossover_rate: f32,
    crossover_parent_node_rate: f32,
}

impl GraphCrossover {
    pub fn new(crossover_rate: f32, crossover_parent_node_rate: f32) -> Self {
        GraphCrossover {
            crossover_rate,
            crossover_parent_node_rate,
        }
    }

    #[inline]
    pub fn cross<T>(
        &self,
        population: &Population<GraphChromosome<T>>,
        indexes: &[usize],
        generation: i32,
    ) -> Option<Phenotype<GraphChromosome<T>>>
    where
        T: Clone + PartialEq + Default,
    {
        let parent_one = &population[indexes[0]];
        let parent_two = &population[indexes[1]];

        let geno_one = parent_one.genotype();
        let geno_two = parent_two.genotype();

        let chromo_index =
            random_provider::random_range(0..std::cmp::min(geno_one.len(), geno_two.len()));

        let chromo_one = &geno_one[chromo_index];
        let chromo_two = &geno_two[chromo_index];

        let mut new_chromo_one = chromo_one.clone();
        let mut num_crosses = 0;

        let edge_indecies = (0..std::cmp::min(chromo_one.len(), chromo_two.len()))
            .filter(|i| {
                let node_one = chromo_one.get_gene(*i);
                let node_two = chromo_two.get_gene(*i);

                node_one.node_type() == NodeType::Edge && node_two.node_type() == NodeType::Edge
            })
            .collect::<Vec<usize>>();

        if edge_indecies.is_empty() {
            return None;
        }

        for i in edge_indecies {
            let node_one = chromo_one.get_gene(i);
            let node_two = chromo_two.get_gene(i);

            if random_provider::random::<f32>() < self.crossover_parent_node_rate {
                new_chromo_one.set_gene(node_one.index(), node_one.with_allele(node_two.allele()));
                num_crosses += 1;
            }
        }

        if num_crosses > 0 {
            return Some(Phenotype::from_chromosomes(
                vec![new_chromo_one],
                generation,
            ));
        }

        None
    }
}

impl EngineCompoment for GraphCrossover {
    fn name(&self) -> &'static str {
        "GraphCrossover"
    }
}

impl<T> Alter<GraphChromosome<T>> for GraphCrossover
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.crossover_rate
    }

    fn to_alter(self) -> AlterAction<GraphChromosome<T>> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<T> Crossover<GraphChromosome<T>> for GraphCrossover
where
    T: Clone + PartialEq + Default,
{
    fn crossover(
        &self,
        population: &mut Population<GraphChromosome<T>>,
        generation: i32,
    ) -> Vec<Metric> {
        let timer = Timer::new();
        let mut count = 0;
        let mut new_phenotypes = HashMap::new();

        for index in 0..population.len() {
            if random_provider::random::<f32>() < self.crossover_rate
                && population.len() > NUM_PARENTS
            {
                let parent_indexes = indexes::individual_indexes(index, population.len(), 2);

                if let Some(phenotype) = self.cross(population, &parent_indexes, generation) {
                    new_phenotypes.insert(index, phenotype);
                    count += 1;
                }
            }
        }

        for (index, phenotype) in new_phenotypes.into_iter() {
            population[index] = phenotype;
        }

        vec![Metric::new_operations(
            self.name(),
            count as f32,
            timer.duration(),
        )]
    }
}
