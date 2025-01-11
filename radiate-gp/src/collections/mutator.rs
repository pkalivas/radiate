use std::sync::Arc;

use super::{Factory, GraphChromosome};

use radiate::{random_provider, Chromosome};
use radiate::{Alter, AlterAction, EngineCompoment, Mutate};

use crate::ops::operation::Op;

use radiate::engines::genome::genes::gene::Gene;

pub struct OperationMutator {
    pub rate: f32,
    pub replace_rate: f32,
}

impl OperationMutator {
    pub fn new(rate: f32, replace_rate: f32) -> Self {
        Self { rate, replace_rate }
    }
}

impl EngineCompoment for OperationMutator {
    fn name(&self) -> &'static str {
        "OpMutator"
    }
}

impl<T> Alter<GraphChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<GraphChromosome<Op<T>>> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<T> Mutate<GraphChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<Op<T>>) -> i32 {
        let mutation_indexes = (0..chromosome.len())
            .filter(|_| random_provider::random::<f32>() < self.rate)
            .collect::<Vec<usize>>();

        if mutation_indexes.is_empty() {
            return 0;
        }

        for &i in mutation_indexes.iter() {
            let curreent_node = chromosome.get_gene(i);

            match curreent_node.allele() {
                Op::MutableConst {
                    name,
                    arity,
                    value,
                    get_value,
                    modifier,
                    operation,
                } => {
                    let new_value = get_value();
                    let modified_value = modifier(value);

                    let new_op = Op::MutableConst {
                        name,
                        arity: *arity,
                        value: if random_provider::random::<f32>() < self.replace_rate {
                            new_value
                        } else {
                            modified_value
                        },
                        modifier: Arc::clone(modifier),
                        get_value: Arc::clone(get_value),
                        operation: Arc::clone(operation),
                    };

                    chromosome.set_gene(i, curreent_node.with_allele(&new_op));
                }
                _ => {
                    if let Some(store) = chromosome.store.as_ref() {
                        let new_op = store.borrow().new_instance((i, curreent_node.node_type()));

                        if new_op.value().arity() == curreent_node.value().arity() {
                            chromosome.set_gene(i, curreent_node.with_allele(new_op.allele()));
                        }
                    }
                }
            }
        }

        mutation_indexes.len() as i32
    }
}
