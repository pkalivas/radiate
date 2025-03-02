use super::TreeChromosome;
use crate::{Factory, NodeStore, NodeType, TreeNode, node::Node};
use radiate::{AlterAction, AlterResult, Alterer, Gene, IntoAlter, Mutate, random_provider};

pub struct TreeMutator {
    rate: f32,
}

impl TreeMutator {
    pub fn new(rate: f32) -> Self {
        TreeMutator { rate }
    }

    fn mutate_node<T>(&self, node: &mut TreeNode<T>, store: &NodeStore<T>, rate: f32) -> i32
    where
        T: Clone + PartialEq + Default,
    {
        let mut count = 0;

        if node.is_leaf() {
            if random_provider::random::<f32>() < rate {
                let leaf_value: TreeNode<T> = store.new_instance(NodeType::Leaf);
                node.with_allele(leaf_value.allele());
                count += 1;
            }
        } else {
            if random_provider::random::<f32>() < rate {
                let new_gate: TreeNode<T> = store.new_instance(node.node_type());

                if new_gate.arity() == node.arity() {
                    (*node) = node.with_allele(&new_gate.allele());
                    count += 1;
                }
            }

            for child in node.children_mut().unwrap() {
                count += self.mutate_node(child, store, rate);
            }
        }

        count
    }
}

impl<T> Mutate<TreeChromosome<T>> for TreeMutator
where
    T: Clone + PartialEq + Default,
{
    fn mutate_chromosome(&self, chromosome: &mut TreeChromosome<T>, rate: f32) -> AlterResult {
        let store = chromosome.get_store();
        if let Some(store) = store {
            let mutations = self.mutate_node(chromosome.root_mut(), &store, rate);
            return mutations.into();
        } else {
            return 0.into();
        }
    }
}

impl<T> IntoAlter<TreeChromosome<T>> for TreeMutator
where
    T: Clone + PartialEq + Default,
{
    fn into_alter(self) -> Alterer<TreeChromosome<T>> {
        Alterer::new(
            "TreeMutator",
            self.rate,
            AlterAction::Mutate(Box::new(self)),
        )
    }
}
