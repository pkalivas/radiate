use super::TreeChromosome;
use crate::{Factory, NodeStore, NodeType, TreeNode, node::Node};
use radiate::{AlterResult, Gene, Mutate, random_provider};

pub struct TreeMutator {
    rate: f32,
}

impl TreeMutator {
    pub fn new(rate: f32) -> Self {
        TreeMutator { rate }
    }

    fn mutate_node<T>(node: &mut TreeNode<T>, store: &NodeStore<T>, rate: f32) -> usize
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
                    *node = node.with_allele(new_gate.allele());
                    count += 1;
                }
            }

            for child in node.children_mut().unwrap() {
                count += TreeMutator::mutate_node(child, store, rate);
            }
        }

        count
    }
}

impl<T> Mutate<TreeChromosome<T>> for TreeMutator
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn mutate_chromosome(&self, chromosome: &mut TreeChromosome<T>, rate: f32) -> AlterResult {
        let store = chromosome.get_store();
        if let Some(store) = store {
            let mutations = TreeMutator::mutate_node(chromosome.root_mut(), &store, rate);
            mutations.into()
        } else {
            0.into()
        }
    }
}
