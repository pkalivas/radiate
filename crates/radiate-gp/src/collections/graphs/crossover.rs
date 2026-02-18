use crate::collections::GraphChromosome;
use crate::node::{Node, NodeExt};
use radiate_core::{AlterResult, Crossover, LineageUpdate, random_provider};
use radiate_core::{Rate, genome::*};
use std::fmt::Debug;

const NUM_PARENTS: usize = 2;

pub struct GraphCrossover {
    rate: Rate,
    parent_node_rate: f32,
}

impl GraphCrossover {
    pub fn new(rate: impl Into<Rate>, crossover_parent_node_rate: f32) -> Self {
        GraphCrossover {
            rate: rate.into(),
            parent_node_rate: crossover_parent_node_rate,
        }
    }
}

impl<T> Crossover<GraphChromosome<T>> for GraphCrossover
where
    T: Clone + PartialEq + Debug,
{
    fn rate(&self) -> Rate {
        self.rate.clone()
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

                random_provider::with_rng(|rand| {
                    let chromo_index = rand.range(0..std::cmp::min(geno_one.len(), geno_two.len()));

                    let chromo_one = geno_one.get_mut(chromo_index).unwrap();
                    let chromo_two = geno_two.get(chromo_index).unwrap();

                    let mut crosses = 0;
                    let min_len = std::cmp::min(chromo_one.len(), chromo_two.len());

                    for i in 0..min_len {
                        let node_one = chromo_one.get_mut(i);
                        let node_two = chromo_two.get(i);

                        if node_one.arity() != node_two.arity() {
                            continue;
                        }

                        if !rand.bool(self.parent_node_rate) {
                            continue;
                        }

                        if node_one.value() != node_two.value() {
                            node_one.set_value(node_two.value().clone());
                            crosses += 1;
                        }
                    }

                    crosses
                })
            };

            if num_crosses > 0 {
                let parent_lineage = (parent_one.family(), parent_two.family());
                let parent_ids = (parent_one.id(), parent_two.id());
                parent_one.invalidate(generation);
                return AlterResult::from((
                    num_crosses,
                    LineageUpdate::from((parent_lineage, parent_ids, parent_one.id())),
                ));
            }
        }

        AlterResult::empty()
    }
}
