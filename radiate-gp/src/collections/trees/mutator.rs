use super::TreeChromosome;
use crate::{Op, TreeNode};
use radiate::{random_provider, Alter, AlterAction, EngineCompoment, Gene, Mutate};
use std::sync::{Arc, RwLock};

pub struct TreeMutator {
    rate: f32,
}

impl TreeMutator {
    pub fn new(rate: f32) -> Self {
        TreeMutator { rate }
    }

    fn mutate_node<T>(
        &self,
        node: &mut TreeNode<T>,
        leafs: &Arc<RwLock<Vec<Op<T>>>>,
        gates: &Arc<RwLock<Vec<Op<T>>>>,
    ) -> i32
    where
        T: Clone + PartialEq + Default,
    {
        let mut count = 0;

        if node.is_leaf() {
            if random_provider::random::<f32>() < self.rate {
                let leaf_values = leafs.read().unwrap();
                (*node) = node.with_allele(&random_provider::choose(&leaf_values));
                count += 1;
            }
        } else {
            if random_provider::random::<f32>() < self.rate {
                let gate_values = gates.read().unwrap();
                let new_gate = random_provider::choose(&gate_values);

                if new_gate.arity() == node.value().arity() {
                    (*node) = node.with_allele(&new_gate);
                    count += 1;
                }
            }

            for child in node.children_mut().unwrap() {
                count += self.mutate_node(child, leafs, gates);
            }
        }

        count
    }
}

impl EngineCompoment for TreeMutator {
    fn name(&self) -> &'static str {
        "TreeMutator"
    }
}

impl<T> Alter<TreeChromosome<T>> for TreeMutator
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<TreeChromosome<T>> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<T> Mutate<TreeChromosome<T>> for TreeMutator
where
    T: Clone + PartialEq + Default,
{
    fn mutate_chromosome(&self, chromosome: &mut TreeChromosome<T>) -> i32 {
        let leafs = chromosome.get_leafs();
        let gates = chromosome.get_gates();
        let root = chromosome.root_mut();

        self.mutate_node(root, &leafs, &gates)
    }
}
