use crate::node::{Node, NodeExt};
use crate::ops::operation::Op;
use crate::{Factory, GraphChromosome, NodeStore, NodeType, TreeChromosome};
use radiate_core::{AlterResult, Metric, Mutate, Rate, metric};
use radiate_core::{Chromosome, random_provider};

const MUT_CONST_OP_MUTATED: &str = "op_mut_const";
#[cfg(feature = "pgm")]
const PGM_OP_MUTATED: &str = "op_pgm";
const FALLBACK_OP_MUTATED: &str = "op_new_inst";

pub struct OperationMutator {
    rate: Rate,
    replace_rate: f32,
}

impl OperationMutator {
    pub fn new(rate: impl Into<Rate>, replace_rate: f32) -> Self {
        let rate = rate.into();
        // if !(0.0..=1.0).contains(&rate.0) {
        //     panic!("rate must be between 0.0 and 1.0");
        // }

        if !(0.0..=1.0).contains(&replace_rate) {
            panic!("replace_rate must be between 0.0 and 1.0");
        }

        OperationMutator { rate, replace_rate }
    }

    #[inline]
    fn mutate_node<T>(
        &self,
        node: &mut impl Node<Value = Op<T>>,
        store: &NodeStore<Op<T>>,
        rate: f32,
    ) -> Vec<(&'static str, usize)>
    where
        T: Clone + PartialEq + Default,
    {
        let mut result = Vec::new();
        match node.value() {
            Op::MutableConst { .. } => {
                if let Some(new_op) = self.try_mutate_mut_const_op(node) {
                    node.set_value(new_op);
                    result.push((MUT_CONST_OP_MUTATED, 1));
                }
            }
            #[cfg(feature = "pgm")]
            Op::PGM(..) => result.extend(self.try_mutate_pga_op(node, store, rate)),
            _ => {
                let new_op: Op<T> = store.new_instance(node.node_type());
                (new_op.arity() == node.value().arity())
                    .then_some(new_op)
                    .map(|op| {
                        node.set_value(op);
                        result.push((FALLBACK_OP_MUTATED, 1));
                    });
            }
        }

        result
    }

    #[inline]
    fn try_mutate_mut_const_op<T>(&self, node: &impl Node<Value = Op<T>>) -> Option<Op<T>>
    where
        T: Clone + PartialEq + Default,
    {
        match node.value() {
            Op::MutableConst {
                name,
                arity,
                value,
                supplier: get_value,
                modifier,
                operation,
            } => {
                let new_value = if random_provider::random::<f32>() < self.replace_rate {
                    get_value()
                } else {
                    modifier(value)
                };

                Some(Op::MutableConst {
                    name,
                    arity: *arity,
                    value: new_value,
                    modifier: *modifier,
                    supplier: *get_value,
                    operation: *operation,
                })
            }
            _ => None,
        }
    }

    #[inline]
    #[cfg(feature = "pgm")]
    fn try_mutate_pga_op<T>(
        &self,
        node: &mut impl Node<Value = Op<T>>,
        store: &NodeStore<Op<T>>,
        rate: f32,
    ) -> Vec<(&'static str, usize)>
    where
        T: Clone + PartialEq + Default,
    {
        use std::sync::Arc;

        let mut result = Vec::new();
        if let Op::PGM(_, _, programs, _) = node.value_mut() {
            let programs_mut = Arc::make_mut(programs);

            for prog in programs_mut.iter_mut() {
                for idx in random_provider::cond_indices(0..prog.size(), rate) {
                    if let Some(child_node) = prog.get_mut(idx) {
                        result.extend(self.mutate_node(child_node, store, rate));
                    }
                }
            }
        }

        result.push((PGM_OP_MUTATED, result.len()));
        result
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
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<Op<T>>, rate: f32) -> AlterResult {
        let mutation_indexes = random_provider::cond_indices(0..chromosome.len(), rate);
        let store = chromosome.store().map(|store| store.clone());

        let mut metrics = Vec::new();
        for i in mutation_indexes.iter() {
            let node = chromosome.get_mut(*i);

            if matches!(node.node_type(), NodeType::Input | NodeType::Output) {
                continue;
            }

            if let Some(store) = store.as_ref() {
                metrics.extend(self.mutate_node(node, store, rate));
            }
        }

        // let grouped = metrics
        //     .into_iter()
        //     .fold(Vec::new(), |mut acc: Vec<(&'static str, usize)>, item| {
        //         if let Some((_, count)) = acc.iter_mut().find(|(name, _)| *name == item.0) {
        //             *count += item.1;
        //         } else {
        //             acc.push(item);
        //         }
        //         acc
        //     })
        //     .into_iter()
        //     .map(|(name, count)| metric!(name, count))
        //     .collect::<Vec<Metric>>();
        let grouped = metrics
            .into_iter()
            .map(|(name, count)| metric!(name, count))
            .collect::<Vec<Metric>>();

        AlterResult::from((mutation_indexes.len(), grouped))
    }
}

impl<T> Mutate<TreeChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut TreeChromosome<Op<T>>, rate: f32) -> AlterResult {
        let store = chromosome.get_store().map(|store| store.clone());
        let mut metrics = Vec::new();
        if let Some(store) = store {
            let root = chromosome.root_mut();
            let mut count = 0;
            for idx in random_provider::cond_indices(0..root.size(), rate) {
                if let Some(node) = root.get_mut(idx) {
                    metrics.extend(self.mutate_node(node, &store, rate));
                    count += 1;
                }
            }

            let grouped = metrics
                .into_iter()
                .fold(Vec::new(), |mut acc: Vec<(&'static str, usize)>, item| {
                    if let Some((_, count)) = acc.iter_mut().find(|(name, _)| *name == item.0) {
                        *count += item.1;
                    } else {
                        acc.push(item);
                    }
                    acc
                })
                .into_iter()
                .map(|(name, count)| metric!(name, count))
                .collect::<Vec<Metric>>();

            return AlterResult::from((count, grouped));
        }

        AlterResult::empty()
    }
}
