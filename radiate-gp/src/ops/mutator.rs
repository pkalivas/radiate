use crate::node::Node;
use crate::ops::operation::Op;
use crate::{Factory, GraphChromosome, NodeType};
use radiate::engines::genome::gene::Gene;
use radiate::{AlterAction, AlterResult, Alterer, IntoAlter, Mutate};
use radiate::{Chromosome, random_provider};
use std::sync::Arc;

pub struct OperationMutator {
    rate: f32,
    replace_rate: f32,
}

impl OperationMutator {
    pub fn new(rate: f32, replace_rate: f32) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("rate must be between 0.0 and 1.0");
        }

        if replace_rate < 0.0 || replace_rate > 1.0 {
            panic!("replace_rate must be between 0.0 and 1.0");
        }

        OperationMutator { rate, replace_rate }
    }
}

/// This implementation is for the `GraphChromosome<Op<T>>` type.
/// It mutates the chromosome by changing the value of the `MutableConst` Op nodes (weights).
/// If the node is not a `MutableConst` node, it tries to replace it with a new node from the store,
/// but only if the arity of the new node is the same as the current node.
impl<T> Mutate<GraphChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<Op<T>>, rate: f32) -> AlterResult {
        let mutation_indexes = (0..chromosome.len())
            .filter(|_| random_provider::random::<f32>() < rate)
            .collect::<Vec<usize>>();

        if mutation_indexes.is_empty() {
            return 0.into();
        }

        for &i in mutation_indexes.iter() {
            let current_node = chromosome.get_gene(i);

            if current_node.node_type() == NodeType::Input
                || current_node.node_type() == NodeType::Output
            {
                continue;
            }

            match current_node.allele() {
                Op::MutableConst {
                    name,
                    arity,
                    value,
                    supplier: get_value,
                    modifier,
                    operation,
                } => {
                    let new_value = get_value();
                    let new_value = if random_provider::random::<f32>() < self.replace_rate {
                        new_value
                    } else {
                        modifier(value)
                    };

                    chromosome.set_gene(
                        i,
                        current_node.with_allele(&Op::MutableConst {
                            name,
                            arity: *arity,
                            value: new_value,
                            modifier: Arc::clone(modifier),
                            supplier: Arc::clone(get_value),
                            operation: Arc::clone(operation),
                        }),
                    );
                }

                _ => {
                    let new_op: Option<Op<T>> = chromosome
                        .store()
                        .map(|store| store.new_instance(current_node.node_type()));

                    if let Some(new_op) = new_op {
                        if new_op.arity() == current_node.arity() {
                            chromosome.set_gene(i, current_node.with_allele(&new_op));
                        }
                    }
                }
            }
        }

        mutation_indexes.len().into()
    }
}

impl<T> IntoAlter<GraphChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    fn into_alter(self) -> Alterer<GraphChromosome<Op<T>>> {
        Alterer::new(
            "OperationMutator",
            self.rate,
            AlterAction::Mutate(Box::new(self)),
        )
    }
}
