use std::sync::Arc;
use std::ops::{Add, Mul, Sub};

use num_traits::Float;
use radiate::engines::alterers::mutators::mutate::Mutate;
use radiate::engines::genome::genes::gene::Gene;
use radiate::Alterer;
use rand::{distributions::Standard, prelude::Distribution, random};

use crate::architects::node_collections::node::Node;
use crate::architects::node_collections::node_factory::NodeFactory;
use crate::operations::op::Ops;

pub struct OpMutator<T>
where
    Standard: Distribution<T>,
    T: Clone + PartialEq + Default + Float,
{
    pub rate: f32,
    pub replace_rate: f32,
    pub factory: NodeFactory<T>,
}

impl<T> OpMutator<T>
where
    Standard: Distribution<T>,
    T: Clone + PartialEq + Default + Float + 'static,
{
    pub fn alterer(
        factory: NodeFactory<T>,
        rate: f32,
        replace_rate: f32,
    ) -> Alterer<Node<T>, Ops<T>> {
        Alterer::Mutation(Box::new(Self {
            rate,
            replace_rate,
            factory,
        }))
    }
}

impl<T> Mutate<Node<T>, Ops<T>> for OpMutator<T>
where
    T: Clone + PartialEq + Default + Mul<Output = T> + Sub<Output = T> + Add<Output = T> + Float,
    Standard: Distribution<T>,
{
    fn mutate_rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_gene(&self, gene: &Node<T>) -> Node<T> {
        match gene.allele() {
            Ops::MutableConst(name, arity, value, supplier, operation) => {
                let random_value = random::<T>() * T::from(2).unwrap() - T::from(1).unwrap();

                if random::<f32>() < self.replace_rate {
                    gene.from_allele(&Ops::MutableConst(
                        &name,
                        *arity,
                        random_value,
                        Arc::clone(supplier),
                        Arc::clone(operation),
                    ))
                } else {
                    let new_value = random_value + value.clone();
                    gene.from_allele(&Ops::MutableConst(
                        &name,
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
