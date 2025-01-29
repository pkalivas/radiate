use super::TreeChromosome;
use crate::{node::Node, Factory, NodeStore, NodeType, TreeNode};
use radiate::{random_provider, Alter, AlterAction, EngineCompoment, Gene, Mutate};

pub struct TreeMutator {
    rate: f32,
}

impl TreeMutator {
    pub fn new(rate: f32) -> Self {
        TreeMutator { rate }
    }

    fn mutate_node<T>(&self, node: &mut TreeNode<T>, store: &NodeStore<T>) -> i32
    where
        T: Clone + PartialEq + Default,
    {
        let mut count = 0;

        if node.is_leaf() {
            if random_provider::random::<f32>() < self.rate {
                let leaf_value: TreeNode<T> = store.new_instance(NodeType::Leaf);
                node.with_allele(leaf_value.allele());
                count += 1;
            }
        } else {
            if random_provider::random::<f32>() < self.rate {
                let new_gate: TreeNode<T> = store.new_instance(node.node_type());

                if new_gate.arity() == node.arity() {
                    (*node) = node.with_allele(&new_gate.allele());
                    count += 1;
                }
            }

            for child in node.children_mut().unwrap() {
                count += self.mutate_node(child, store);
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
        let store = chromosome.get_store();
        if let Some(store) = store {
            self.mutate_node(chromosome.root_mut(), &store)
        } else {
            0
        }
    }
}
