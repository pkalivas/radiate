use crate::collections::GraphChromosome;
use crate::node::{Node, NodeExt};
use radiate_core::genome::*;
use radiate_core::{AlterResult, Crossover, random_provider};
use std::fmt::Debug;

const NUM_PARENTS: usize = 2;

pub struct GraphCrossover {
    rate: f32,
    parent_node_rate: f32,
}

impl GraphCrossover {
    pub fn new(rate: f32, crossover_parent_node_rate: f32) -> Self {
        GraphCrossover {
            rate,
            parent_node_rate: crossover_parent_node_rate,
        }
    }
}

impl<T> Crossover<GraphChromosome<T>> for GraphCrossover
where
    T: Clone + PartialEq + Debug,
{
    fn rate(&self) -> f32 {
        self.rate
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
            return AlterResult::empty();
        }

        if let Some((parent_one, parent_two)) = population.get_pair_mut(indexes[0], indexes[1]) {
            let num_crosses = {
                let geno_one = parent_one.genotype_mut();
                let geno_two = parent_two.genotype();

                let chromo_index =
                    random_provider::range(0..std::cmp::min(geno_one.len(), geno_two.len()));

                let chromo_one = geno_one.get_mut(chromo_index).unwrap();
                let chromo_two = geno_two.get(chromo_index).unwrap();

                let node_indices = (0..std::cmp::min(chromo_one.len(), chromo_two.len()))
                    .filter(|i| {
                        let node_one = chromo_one.get(*i);
                        let node_two = chromo_two.get(*i);

                        node_one.arity() == node_two.arity()
                            && random_provider::bool(self.parent_node_rate)
                    })
                    .collect::<Vec<usize>>();

                for &i in node_indices.iter() {
                    let node_two = chromo_two.get(i);
                    chromo_one.get_mut(i).set_value(node_two.value().clone());
                }

                node_indices.len()
            };

            if num_crosses > 0 {
                parent_one.invalidate(generation);
            }

            AlterResult::from(num_crosses)
        } else {
            AlterResult::empty()
        }
    }
}
