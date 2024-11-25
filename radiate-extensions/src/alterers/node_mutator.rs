use std::ops::{Add, Mul, Sub};
use std::sync::Arc;

use num_traits::Float;
use radiate::engines::alterers::mutators::mutate::Mutate;
use radiate::engines::genome::genes::gene::Gene;
use radiate::{Alterer, RandomProvider};
use rand::distributions::uniform::SampleUniform;
use rand::{distributions::Standard, prelude::Distribution};

use crate::architects::node_collections::node::Node;
use crate::architects::node_collections::node_factory::NodeFactory;
use crate::NodeChromosome;
use crate::operations::op::Ops;

pub struct NodeMutator<T>
where
    Standard: Distribution<T>,
    T: Clone + PartialEq + Default + Float,
{
    pub rate: f32,
    pub replace_rate: f32,
    pub factory: NodeFactory<T>,
}

impl<T> NodeMutator<T>
where
    Standard: Distribution<T>,
    T: Clone + PartialEq + Default + Float + SampleUniform + 'static,
{
    pub fn new(rate: f32, replace_rate: f32, factory: NodeFactory<T>) -> Self {
        Self {
            rate,
            replace_rate,
            factory,
        }
    }
    pub fn alterer(
        factory: NodeFactory<T>,
        rate: f32,
        replace_rate: f32,
    ) -> Alterer<NodeChromosome<T>> {
        Alterer::Mutation(Box::new(Self {
            rate,
            replace_rate,
            factory,
        }))
    }
}

impl<T> Mutate<NodeChromosome<T>> for NodeMutator<T>
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
    fn mutate_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "OpMutator"
    }

    #[inline]
    fn mutate_gene(&self, gene: &Node<T>) -> Node<T> {
        match gene.allele() {
            Ops::MutableConst(name, arity, value, supplier, operation) => {
                let random_value =
                    RandomProvider::random::<T>() * T::from(2).unwrap() - T::from(1).unwrap();

                if RandomProvider::random::<f32>() < self.replace_rate {
                    gene.from_allele(&Ops::MutableConst(
                        name,
                        *arity,
                        random_value,
                        Arc::clone(supplier),
                        Arc::clone(operation),
                    ))
                } else {
                    let new_value = random_value + *value;
                    gene.from_allele(&Ops::MutableConst(
                        name,
                        *arity,
                        new_value,
                        Arc::clone(supplier),
                        Arc::clone(operation),
                    ))
                }
            }
            _ => {
                let temp_node = self.factory.new_node(gene.index, gene.node_type);
                if temp_node.value.arity() == gene.value.arity() {
                    return gene.from_allele(temp_node.allele());
                }

                gene.clone()
            }
        }
    }
}
