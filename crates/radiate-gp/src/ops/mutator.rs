use crate::node::{Node, NodeExt};
use crate::ops::operation::Op;
use crate::{Factory, GraphChromosome, NodeStore, NodeType, TreeChromosome};
use radiate_core::{AlterContext, AlterResult, Expr, Mutate, RateSet, SmallStr};
use radiate_core::{Chromosome, random_provider};

const OP_MUTATED: SmallStr = SmallStr::from_static("mutator.op.mutated");
const OP_NEW_INSTANCE: SmallStr = SmallStr::from_static("mutator.op.new");
const OP_MUTATE_NEW_INST: SmallStr = SmallStr::from_static("mutator.op.rate.replace");

#[derive(Default)]
struct OpMutateMetrics {
    op_mutate: usize,
    op_new_instance: usize,
}

impl OpMutateMetrics {
    fn len(&self) -> usize {
        self.op_mutate + self.op_new_instance
    }
}

pub struct OperationMutator {
    rate: Expr,
    replace_rate: Expr,
}

impl OperationMutator {
    pub fn new(rate: impl Into<Expr>, replace_rate: impl Into<Expr>) -> Self {
        OperationMutator {
            rate: rate.into(),
            replace_rate: replace_rate.into(),
        }
    }

    #[inline]
    fn mutate_node<T>(
        &self,
        node: &mut impl Node<Value = Op<T>>,
        store: &NodeStore<Op<T>>,
        replace_rate: f32,
        metrics: &mut OpMutateMetrics,
    ) where
        T: Clone + PartialEq + Default,
    {
        match node.value() {
            Op::Value { .. } => {
                if let Some(new_op) = self.mutate_value_op(node, replace_rate) {
                    node.set_value(new_op);
                    metrics.op_mutate += 1;
                }
            }
            _ => {
                let new_op: Op<T> = store.new_instance(node.node_type());
                if let Some(op) = (new_op.arity() == node.value().arity()).then_some(new_op) {
                    node.set_value(op);
                    metrics.op_new_instance += 1;
                }
            }
        }
    }

    #[inline]
    fn mutate_value_op<T>(
        &self,
        node: &mut impl Node<Value = Op<T>>,
        replace_rate: f32,
    ) -> Option<Op<T>>
    where
        T: Clone + PartialEq + Default,
    {
        match node.value_mut() {
            Op::Value(name, arity, params, operation) => {
                let new_value = if random_provider::random::<f32>() < replace_rate {
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
    fn name(&self) -> String {
        "mutator.op".to_string()
    }

    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone()).add(self.replace_rate.clone().alias(OP_MUTATE_NEW_INST))
    }

    #[inline]
    fn mutate_chromosome(
        &mut self,
        chromosome: &mut GraphChromosome<Op<T>>,
        ctx: &mut AlterContext,
    ) -> AlterResult {
        let mutation_indexes = random_provider::cond_indices(0..chromosome.len(), ctx.rate());
        let store = chromosome.store().cloned();

        let mut metrics = OpMutateMetrics {
            op_mutate: 0,
            op_new_instance: 0,
        };

        let replace_rate = ctx.internal_rate(0);

        for i in mutation_indexes.iter() {
            if let Some(node) = chromosome.get_mut(*i) {
                if matches!(node.node_type(), NodeType::Input | NodeType::Output) {
                    continue;
                }

                if let Some(store) = store.as_ref() {
                    self.mutate_node(node, store, replace_rate, &mut metrics);
                }
            }
        }

        AlterResult::from(metrics.len())
    }
}

impl<T> Mutate<TreeChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone()).add(self.replace_rate.clone().alias(OP_MUTATE_NEW_INST))
    }

    #[inline]
    fn mutate_chromosome(
        &mut self,
        chromosome: &mut TreeChromosome<Op<T>>,
        ctx: &mut AlterContext,
    ) -> AlterResult {
        let store = chromosome.get_store();
        let mut metrics = OpMutateMetrics::default();
        if let Some(store) = store {
            let root = chromosome.root_mut();
            let replace_rate = ctx.internal_rate(0);

            for idx in random_provider::cond_indices(0..root.size(), ctx.rate()) {
                if let Some(node) = root.get_mut(idx) {
                    self.mutate_node(node, &store, replace_rate, &mut metrics);
                }
            }

            ctx.upsert(OP_MUTATED, metrics.op_mutate);
            ctx.upsert(OP_NEW_INSTANCE, metrics.op_new_instance);

            return AlterResult::from(metrics.len());
        }

        AlterResult::empty()
    }
}
