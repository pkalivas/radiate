use crate::NodeType;
use crate::collections::GraphChromosome;
use crate::node::Node;
use radiate::engines::genome::*;
use radiate::{AlterResult, Crossover, random_provider};

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
}

impl<T> Crossover<GraphChromosome<T>> for GraphCrossover
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.crossover_rate
    }

    #[inline]
    fn cross(
        &self,
        population: &mut Population<GraphChromosome<T>>,
        indexes: &[usize],
        generation: usize,
        _: f32,
    ) -> AlterResult {
        if population.len() <= NUM_PARENTS {
            return 0.into();
        }

        let mut num_crosses = 0;

        // Open a scoped block to release read guards
        let new_geno_one = {
            let parent_one = &population[indexes[0]];
            let parent_two = &population[indexes[1]];

            let geno_one = parent_one.genotype();
            let geno_two = parent_two.genotype();

            let min = std::cmp::min(geno_one.len(), geno_two.len());
            let chromo_index = random_provider::range(0..min);

            let chromo_one = &geno_one[chromo_index];
            let chromo_two = &geno_two[chromo_index];

            let edge_indies = (0..std::cmp::min(chromo_one.len(), chromo_two.len()))
                .filter(|i| {
                    let node_one = chromo_one.get(*i);
                    let node_two = chromo_two.get(*i);

                    node_one.node_type() == NodeType::Edge
                        && node_two.node_type() == NodeType::Edge
                        && random_provider::random::<f32>() < self.crossover_parent_node_rate
                })
                .collect::<Vec<usize>>();

            if edge_indies.is_empty() {
                return num_crosses.into();
            }

            let mut new_geno_one = geno_one.clone();
            let new_chromo_one = &mut new_geno_one[chromo_index];

            for i in edge_indies {
                let node_one = chromo_one.get(i);
                let node_two = chromo_two.get(i);
                *new_chromo_one.as_mut()[node_one.index()].value_mut() = node_two.value().clone();
                num_crosses += 1;
            }

            new_geno_one
        };

        if num_crosses > 0 {
            let species_id_one = population[indexes[0]].species_id();
            population[indexes[1]] = Phenotype::from((new_geno_one, generation, species_id_one));
        }

        num_crosses.into()
    }
}
