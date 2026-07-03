use crate::collections::GraphChromosome;
use crate::node::{Node, NodeExt};
use radiate_core::{AlterContext, AlterResult, Crossover, RdRand, random_provider};
use radiate_core::{Rate, genome::*};
use std::cmp::Ordering;
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
            let is_speciated = !parent_one.species().is_empty() && !parent_two.species().is_empty();
            let num_crosses = {
                let geno_one = parent_one.genotype_mut();
                let geno_two = parent_two.genotype();

                random_provider::with_rng(|rand| {
                    let chromo_index = rand.range(0..std::cmp::min(geno_one.len(), geno_two.len()));
                    let chromo_one = geno_one.get_mut(chromo_index).unwrap();
                    let chromo_two = geno_two.get(chromo_index).unwrap();

                    if is_speciated {
                        crossover_speciated(chromo_one, chromo_two, self.parent_node_rate, rand)
                    } else {
                        crossover_uniform(chromo_one, chromo_two, self.parent_node_rate, rand)
                    }
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

fn crossover_uniform<T>(
    chromo_one: &mut GraphChromosome<T>,
    chromo_two: &GraphChromosome<T>,
    rate: f32,
    rand: &mut RdRand,
) -> usize
where
    T: Clone + PartialEq,
{
    let mut crosses = 0;
    let min_len = std::cmp::min(chromo_one.len(), chromo_two.len());

    for i in 0..min_len {
        let node_one = chromo_one.get_mut(i);
        let node_two = chromo_two.get(i);

        if let Some((node_one, node_two)) = node_one.zip(node_two) {
            if node_one.arity() != node_two.arity() {
                continue;
            }

            if !rand.bool(rate) {
                continue;
            }

            if node_one.value() != node_two.value() {
                node_one.set_value(node_two.value().clone());
                crosses += 1;
            }
        }
    }

    crosses
}

fn crossover_speciated<T>(
    chromo_one: &mut GraphChromosome<T>,
    chromo_two: &GraphChromosome<T>,
    rate: f32,
    rand: &mut RdRand,
) -> usize
where
    T: Clone + PartialEq + Debug,
{
    let mut crosses = 0;
    let (mut ia, mut ib) = (0, 0);

    while ia < chromo_one.len() && ib < chromo_two.len() {
        let gene_one = chromo_one.get(ia);
        let gene_two = chromo_two.get(ib);

        let Some((gene_one, gene_two)) = gene_one.zip(gene_two) else {
            break;
        };

        match gene_one.innovation().cmp(&gene_two.innovation()) {
            Ordering::Equal => {
                if rand.bool(rate) {
                    // let node_two = chromo_two.get(ib);
                    let node_one = chromo_one.get_mut(ia);

                    if let Some(node_one) = node_one
                        && node_one.arity() == gene_two.arity()
                        && node_one.value() != gene_two.value()
                    {
                        node_one.set_value(gene_two.value().clone());
                        crosses += 1;
                    }

                    // if node_one.arity() == gene_two.arity() && node_one.value() != gene_two.value()
                    // {
                    //     node_one.set_value(gene_two.value().clone());
                    //     crosses += 1;
                    // }
                }

                ia += 1;
                ib += 1;
            }
            Ordering::Less => ia += 1,
            Ordering::Greater => ib += 1,
        }
    }

    crosses
}
