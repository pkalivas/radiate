use crate::node::{Node, NodeExt};
use crate::ops::operation::Op;
use crate::{Factory, GraphChromosome, NodeStore, NodeType, TreeChromosome};
use radiate_core::{AlterResult, Metric, Mutate, Rate, Valid, metric};
use radiate_core::{Chromosome, random_provider};

const MUT_CONST_OP_MUTATED: &str = "op_mut_const";
const FALLBACK_OP_MUTATED: &str = "op_new_inst";

pub struct OperationMutator {
    rate: Rate,
    replace_rate: f32,
}

impl OperationMutator {
    pub fn new(rate: impl Into<Rate>, replace_rate: f32) -> Self {
        let rate = rate.into();
        if !rate.is_valid() {
            panic!("rate must be between 0.0 and 1.0");
        }

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
        _rate: f32,
    ) -> Vec<Metric>
    where
        T: Clone + PartialEq + Default,
    {
        let mut result = Vec::new();
        match node.value() {
            Op::Value { .. } => {
                if let Some(new_op) = self.try_mutate_mut_const_op(node) {
                    node.set_value(new_op);
                    result.push(metric!(MUT_CONST_OP_MUTATED, 1));
                }
            }
            _ => {
                let new_op: Op<T> = store.new_instance(node.node_type());
                (new_op.arity() == node.value().arity())
                    .then_some(new_op)
                    .map(|op| {
                        node.set_value(op);
                        result.push(metric!(FALLBACK_OP_MUTATED, 1));
                    });
            }
        }

        result
    }

    #[inline]
    fn try_mutate_mut_const_op<T>(&self, node: &mut impl Node<Value = Op<T>>) -> Option<Op<T>>
    where
        T: Clone + PartialEq + Default,
    {
        match node.value_mut() {
            Op::Value(name, arity, value, operation) => {
                let new_value = if random_provider::random::<f32>() < self.replace_rate {
                    value.new_instance(())
                } else {
                    value.new_instance(value.data().clone())
                };

                Some(Op::Value(name, *arity, new_value, *operation))
            }
            _ => None,
        }
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

        AlterResult::from((mutation_indexes.len(), metrics))
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

            for idx in random_provider::cond_indices(0..root.size(), rate) {
                if let Some(node) = root.get_mut(idx) {
                    metrics.extend(self.mutate_node(node, &store, rate));
                }
            }

            return AlterResult::from((metrics.len(), metrics));
        }

        AlterResult::empty()
    }
}

// match node.value() {
//     Op::MutableConst {
//         name,
//         arity,
//         value,
//         supplier: get_value,
//         modifier,
//         operation,
//     } => {
//         let new_value = if random_provider::random::<f32>() < self.replace_rate {
//             get_value()
//         } else {
//             modifier(value)
//         };

//         Some(Op::MutableConst {
//             name,
//             arity: *arity,
//             value: new_value,
//             modifier: *modifier,
//             supplier: *get_value,
//             operation: *operation,
//         })
//     }
//     _ => None,
// }
