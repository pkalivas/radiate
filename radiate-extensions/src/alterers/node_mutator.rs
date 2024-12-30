use std::ops::{Add, Mul, Sub};
use std::sync::Arc;

use crate::collections::GraphChromosome;
use crate::ops::operation::Operation;
use num_traits::Float;
use radiate::alter::AlterType;
use radiate::engines::genome::genes::gene::Gene;
use radiate::{random_provider, Alter, Chromosome};
use rand::distributions::uniform::SampleUniform;
use rand::{distributions::Standard, prelude::Distribution};

pub struct NodeMutator<T>
where
    Standard: Distribution<T>,
    T: Clone + PartialEq + Default + Float,
{
    pub rate: f32,
    pub replace_rate: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NodeMutator<T>
where
    Standard: Distribution<T>,
    T: Clone + PartialEq + Default + Float + SampleUniform + 'static,
{
    pub fn new(rate: f32, replace_rate: f32) -> Self {
        Self {
            rate,
            replace_rate,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Alter<GraphChromosome<T>> for NodeMutator<T>
where
    T: Clone
        + PartialEq
        + Default
        + Mul<Output = T>
        + Sub<Output = T>
        + Add<Output = T>
        + Float
        + SampleUniform,
    Standard: Distribution<T>,
{
    fn name(&self) -> &'static str {
        "OpMutator"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Mutator
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<T>) -> i32 {
        let two = T::from(0.1).unwrap();
        let one = T::from(0.05).unwrap();

        let mutation_indexes = (0..chromosome.len())
            .filter(|_| random_provider::random::<f32>() < self.rate)
            .collect::<Vec<usize>>();

        if mutation_indexes.is_empty() {
            return 0;
        }

        for &i in mutation_indexes.iter() {
            let curreent_node = chromosome.get_gene(i);

            match curreent_node.allele() {
                Operation::MutableConst {
                    name,
                    arity,
                    value,
                    get_value,
                    operation,
                } => {
                    let random_value = random_provider::random::<T>() * two - one;

                    let new_op = Operation::MutableConst {
                        name,
                        arity: *arity,
                        value: if random_provider::random::<f32>() < self.replace_rate {
                            random_value
                        } else {
                            random_value + *value
                        },
                        get_value: Arc::clone(get_value),
                        operation: Arc::clone(operation),
                    };

                    chromosome.set_gene(i, curreent_node.with_allele(&new_op));
                }
                _ => {
                    let temp_node = chromosome.new_node(i, curreent_node.node_type);

                    if temp_node.value.arity() == curreent_node.value.arity() {
                        chromosome.set_gene(i, curreent_node.with_allele(temp_node.allele()));
                    }
                }
            }
        }

        mutation_indexes.len() as i32
    }
}
