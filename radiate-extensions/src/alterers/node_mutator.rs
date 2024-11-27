use std::ops::{Add, Mul, Sub};
use std::sync::Arc;

use crate::operations::op::Ops;
use crate::NodeChromosome;
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

impl<T> Alter<NodeChromosome<T>> for NodeMutator<T>
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
    fn mutate_chromosome(&self, chromosome: &mut NodeChromosome<T>, range: i32) -> i32 {
        let mut count = 0;

        for i in 0..chromosome.len() {
            if random_provider::random::<i32>() < range {
                count += 1;
                let temp_node = chromosome.new_node(i, chromosome.get_gene(i).node_type);
                let current_node = chromosome.get_gene(i);

                match current_node.allele() {
                    Ops::MutableConst(name, arity, value, supplier, operation) => {
                        let random_value = random_provider::random::<T>() * T::from(2).unwrap()
                            - T::from(1).unwrap();

                        if random_provider::random::<f32>() < self.replace_rate {
                            chromosome.set_gene(
                                i,
                                current_node.from_allele(&Ops::MutableConst(
                                    name,
                                    *arity,
                                    random_value,
                                    Arc::clone(supplier),
                                    Arc::clone(operation),
                                )),
                            );
                        } else {
                            let new_value = random_value + *value;
                            chromosome.set_gene(
                                i,
                                current_node.from_allele(&Ops::MutableConst(
                                    name,
                                    *arity,
                                    new_value,
                                    Arc::clone(supplier),
                                    Arc::clone(operation),
                                )),
                            );
                        }
                    }
                    _ => {
                        if temp_node.value.arity() == current_node.value.arity() {
                            chromosome.set_gene(i, current_node.from_allele(temp_node.allele()));
                        }
                    }
                }
            }
        }

        count
    }
}
