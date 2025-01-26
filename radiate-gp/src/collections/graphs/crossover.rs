use std::collections::HashMap;

use crate::collections::GraphChromosome;
use crate::{NodeCell, NodeType};

use radiate::engines::genome::*;
use radiate::timer::Timer;
use radiate::{indexes, random_provider, Alter, AlterAction, Crossover, EngineCompoment, Metric};

const NUM_PARENTS: usize = 2;

pub struct GraphCrossover<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    crossover_rate: f32,
    crossover_parent_node_rate: f32,
    _marker: std::marker::PhantomData<C>,
}

impl<C> GraphCrossover<C>
where
    C: NodeCell + Clone + PartialEq + Default + 'static,
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
        population: &Population<GraphChromosome<C>>,
        indexes: &[usize],
        generation: i32,
    ) -> Option<Phenotype<GraphChromosome<C>>> {
        let parent_one = &population[indexes[0]];
        let parent_two = &population[indexes[1]];

        let geno_one = parent_one.genotype();
        let geno_two = parent_two.genotype();

        let rand_idx = random_provider::random::<usize>();
        let chromo_index = rand_idx % std::cmp::min(geno_one.len(), geno_two.len());

        let chromo_one = &geno_one[chromo_index];
        let chromo_two = &geno_two[chromo_index];

        let mut new_chromo_one = chromo_one.clone();
        let mut num_crosses = 0;

        let edge_indexes = (0..std::cmp::min(chromo_one.len(), chromo_two.len()))
            .filter(|i| {
                let node_one = chromo_one.get_gene(*i);
                let node_two = chromo_two.get_gene(*i);

                node_one.node_type() == NodeType::Edge && node_two.node_type() == NodeType::Edge
            })
            .collect::<Vec<usize>>();

        if edge_indexes.is_empty() {
            return None;
        }

        for i in edge_indexes {
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

impl<C> EngineCompoment for GraphCrossover<C>
where
    C: NodeCell + Clone + PartialEq + Default + 'static,
{
    fn name(&self) -> &'static str {
        "GraphCrossover"
    }
}

impl<C> Alter<GraphChromosome<C>> for GraphCrossover<C>
where
    C: NodeCell + Clone + PartialEq + Default + 'static,
{
    fn rate(&self) -> f32 {
        self.crossover_rate
    }

    fn to_alter(self) -> radiate::AlterAction<GraphChromosome<C>> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<C> Crossover<GraphChromosome<C>> for GraphCrossover<C>
where
    C: NodeCell + Clone + PartialEq + Default + 'static,
{
    fn crossover(
        &self,
        population: &mut Population<GraphChromosome<C>>,
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

        let mut metric = Metric::new_operations(self.name());
        metric.add_value(count as f32);
        metric.add_duration(timer.duration());

        vec![metric]
    }
}
