use crate::node::{Node, NodeExt};
use crate::ops::operation::Op;
use crate::{Factory, GraphChromosome, NodeStore, NodeType, TreeChromosome};
use radiate_core::{AlterResult, Metric, Mutate, Rate, Valid, metric};
use radiate_core::{Chromosome, random_provider};

const OP_MUTATED: &str = "op_mutate";
const OP_NEW_INSTANCE: &str = "op_new";

#[derive(Default)]
struct OpMutateMetrics {
    op_mutate: usize,
    op_new_instance: usize,
}

impl OpMutateMetrics {
    fn entries(&self) -> Vec<Metric> {
        let mut result = Vec::with_capacity(2);
        if self.op_mutate > 0 {
            result.push(metric!(OP_MUTATED.into(), self.op_mutate));
        }
        if self.op_new_instance > 0 {
            result.push(metric!(OP_NEW_INSTANCE.into(), self.op_new_instance));
        }

        result
    }

    fn len(&self) -> usize {
        self.op_mutate + self.op_new_instance
    }
}

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
        metrics: &mut OpMutateMetrics,
    ) where
        T: Clone + PartialEq + Default,
    {
        match node.value() {
            Op::Value { .. } => {
                if let Some(new_op) = self.mutate_value_op(node) {
                    node.set_value(new_op);
                    metrics.op_mutate += 1;
                }
            }
            _ => {
                let new_op: Op<T> = store.new_instance(node.node_type());
                (new_op.arity() == node.value().arity())
                    .then_some(new_op)
                    .map(|op| {
                        node.set_value(op);
                        metrics.op_new_instance += 1;
                    });
            }
        }
    }

    #[inline]
    fn mutate_value_op<T>(&self, node: &mut impl Node<Value = Op<T>>) -> Option<Op<T>>
    where
        T: Clone + PartialEq + Default,
    {
        match node.value_mut() {
            Op::Value(name, arity, params, operation) => {
                let new_value = if random_provider::random::<f32>() < self.replace_rate {
                    params.new_instance(())
                } else {
                    let modifier = params.modifier();
                    modifier(params.data_mut());
                    params.clone()
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

        let mut metrics = OpMutateMetrics {
            op_mutate: 0,
            op_new_instance: 0,
        };
        for i in mutation_indexes.iter() {
            let node = chromosome.get_mut(*i);

            if matches!(node.node_type(), NodeType::Input | NodeType::Output) {
                continue;
            }

            if let Some(store) = store.as_ref() {
                self.mutate_node(node, store, &mut metrics);
            }
        }

        AlterResult::from((mutation_indexes.len(), metrics.entries()))
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
        let mut metrics = OpMutateMetrics::default();
        if let Some(store) = store {
            let root = chromosome.root_mut();

            for idx in random_provider::cond_indices(0..root.size(), rate) {
                if let Some(node) = root.get_mut(idx) {
                    self.mutate_node(node, &store, &mut metrics);
                }
            }

            return AlterResult::from((metrics.len(), metrics.entries()));
        }

        AlterResult::empty()
    }
}
