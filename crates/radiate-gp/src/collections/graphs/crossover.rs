use crate::collections::GraphChromosome;
use crate::node::{Node, NodeExt};
use radiate_core::{AlterContext, AlterResult, Crossover, RdRand, random_provider};
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
        mut population: &mut [Phenotype<GraphChromosome<T>>],
        indexes: &[usize],
        ctx: &mut AlterContext,
    ) -> AlterResult {
        if population.len() <= NUM_PARENTS {
            return AlterResult::empty();
        }

        if let Some((parent_one, parent_two)) = population.get_pair_mut(indexes[0], indexes[1]) {
            let num_crosses = {
                let geno_one = parent_one.genotype_mut();
                let geno_two = parent_two.genotype();

                // random_provider::with_rng(|rand| {
                //     let chromo_index = rand.range(0..std::cmp::min(geno_one.len(), geno_two.len()));

                //     let chromo_one = geno_one.get_mut(chromo_index).unwrap();
                //     let chromo_two = geno_two.get(chromo_index).unwrap();

                //     let mut crosses = 0;
                //     let min_len = std::cmp::min(chromo_one.len(), chromo_two.len());

                //     for i in 0..min_len {
                //         let node_one = chromo_one.get_mut(i);
                //         let node_two = chromo_two.get(i);

                //         if node_one.arity() != node_two.arity() {
                //             continue;
                //         }

                //         if !rand.bool(self.parent_node_rate) {
                //             continue;
                //         }

                //         if node_one.innovation() == node_two.innovation() {
                //             node_one.set_value(node_two.value().clone());
                //             crosses += 1;
                //         }
                //     }

                //     crosses
                // })

                random_provider::with_rng(|rand| {
                    let chromo_index = rand.range(0..std::cmp::min(geno_one.len(), geno_two.len()));
                    let chromo_one = geno_one.get_mut(chromo_index).unwrap();
                    let chromo_two = geno_two.get(chromo_index).unwrap();

                    crossover_by_innovation(chromo_one, chromo_two, rand, self.parent_node_rate)
                })
            };

            if num_crosses > 0 {
                parent_one.invalidate(ctx.generation());

                return AlterResult::from(num_crosses);
            }
        }

        AlterResult::empty()
    }
}

use crate::collections::graphs::node::InnovationId;
use std::cmp::Ordering;

fn crossover_by_innovation<T: Clone + PartialEq>(
    chromo_one: &mut GraphChromosome<T>,
    chromo_two: &GraphChromosome<T>,
    rand: &mut RdRand,
    parent_node_rate: f32,
) -> usize {
    // (innovation, position-in-chromosome). Skip unmarked nodes.
    let mut a: Vec<(InnovationId, usize)> = chromo_one
        .iter()
        .enumerate()
        .filter_map(|(i, n)| n.innovation().map(|id| (id, i)))
        .collect();
    let mut b: Vec<(InnovationId, usize)> = chromo_two
        .iter()
        .enumerate()
        .filter_map(|(i, n)| n.innovation().map(|id| (id, i)))
        .collect();

    a.sort_unstable_by_key(|(id, _)| *id);
    b.sort_unstable_by_key(|(id, _)| *id);

    let mut crosses = 0;
    let (mut ia, mut ib) = (0usize, 0usize);

    while ia < a.len() && ib < b.len() {
        let (id_a, pos_a) = a[ia];
        let (id_b, pos_b) = b[ib];

        match id_a.cmp(&id_b) {
            Ordering::Equal => {
                // Matching innovation: the only place crossover fires.
                if rand.bool(parent_node_rate) {
                    let node_two = chromo_two.get(pos_b);
                    let node_one = chromo_one.get_mut(pos_a);
                    if node_one.arity() == node_two.arity() && node_one.value() != node_two.value()
                    {
                        node_one.set_value(node_two.value().clone());
                        crosses += 1;
                    }
                }
                ia += 1;
                ib += 1;
            }
            // Disjoint/excess in parent_one — keep as-is (in-place semantics).
            Ordering::Less => ia += 1,
            // Disjoint/excess in parent_two — ignore (we don't pull new genes in).
            Ordering::Greater => ib += 1,
        }
    }

    crosses
}
