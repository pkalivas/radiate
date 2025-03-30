use crate::collections::GraphChromosome;
use crate::node::Node;
use radiate::genome::*;
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

    // #[inline]
    // fn cross_chromosomes(
    //     &self,
    //     chrom_one: &mut GraphChromosome<T>,
    //     chrom_two: &mut GraphChromosome<T>,
    //     rate: f32,
    // ) -> AlterResult {
    //     let mut result = AlterResult::default();
    //     let mut num_crosses = 0;

    //     let edge_indies = (0..std::cmp::min(chrom_one.len(), chrom_two.len()))
    //         .filter(|i| {
    //             let node_one = chrom_one.get(*i);
    //             let node_two = chrom_two.get(*i);

    //             node_one.arity() == node_two.arity()
    //                 && random_provider::random::<f32>() < self.crossover_parent_node_rate
    //         })
    //         .collect::<Vec<usize>>();

    //     if edge_indies.is_empty() {
    //         return num_crosses.into();
    //     }

    //     for i in edge_indies {
    //         let node_two = chrom_two.get(i);

    //         *chrom_one.as_mut()[i].value_mut() = node_two.value().clone();
    //         num_crosses += 1;
    //     }

    //     if num_crosses > 0 {
    //         result.add_count(num_crosses);
    //         result.mark_changed(chrom_one.id());
    //         result.mark_changed(chrom_two.id()); // Mark the second parent as well
    //     }

    //     result
    // }

    #[inline]
    fn cross(
        &self,
        population: &mut Population<GraphChromosome<T>>,
        indexes: &[usize],
        _: usize,
        _: f32,
    ) -> AlterResult {
        let mut result = AlterResult::default();
        if population.len() <= NUM_PARENTS {
            return 0.into();
        }

        let (parent_one, parent_two) = population.get_pair_mut(indexes[0], indexes[1]);

        let num_crosses = {
            let mut geno_one = parent_one.genotype_mut();
            let geno_two = parent_two.genotype();

            let chromo_index =
                random_provider::range(0..std::cmp::min(geno_one.len(), geno_two.len()));

            let chromo_one = geno_one.get_mut(chromo_index);
            let chromo_two = geno_two.get(chromo_index);

            let mut num_crosses = 0;

            let edge_indies = (0..std::cmp::min(chromo_one.len(), chromo_two.len()))
                .filter(|i| {
                    let node_one = chromo_one.get(*i);
                    let node_two = chromo_two.get(*i);

                    node_one.arity() == node_two.arity()
                        && random_provider::random::<f32>() < self.crossover_parent_node_rate
                })
                .collect::<Vec<usize>>();

            if edge_indies.is_empty() {
                return num_crosses.into();
            }

            for i in edge_indies {
                let node_two = chromo_two.get(i);

                *chromo_one.as_mut()[i].value_mut() = node_two.value().clone();
                num_crosses += 1;
            }

            num_crosses
        };

        if num_crosses > 0 {
            result.add_count(num_crosses);
            result.mark_changed(indexes[0]);
        }

        result
    }
}
