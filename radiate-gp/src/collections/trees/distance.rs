use super::TreeChromosome;
use crate::{Node, TreeNode};
use radiate::{Distance, Genotype};
use std::{
    collections::{HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

pub struct SubtreeHashDistance {
    threshold: f32,
}

impl SubtreeHashDistance {
    pub fn new(threshold: f32) -> Self {
        SubtreeHashDistance { threshold }
    }

    pub fn subtree_hashes<T: Hash>(node: &TreeNode<T>, hashes: &mut HashSet<u64>) {
        let mut hasher = DefaultHasher::new();
        node.value().hash(&mut hasher);

        if let Some(children) = node.children() {
            for child in children {
                SubtreeHashDistance::subtree_hashes(child, hashes);
                child.value().hash(&mut hasher);
            }
        }

        hashes.insert(hasher.finish());
    }
}

impl<T> Distance<TreeChromosome<T>> for SubtreeHashDistance
where
    T: Clone + PartialEq + Default + Hash,
{
    fn threshold(&self) -> f32 {
        self.threshold
    }

    fn distance(
        &self,
        one: &Genotype<TreeChromosome<T>>,
        two: &Genotype<TreeChromosome<T>>,
    ) -> f32 {
        fn walk<T: PartialEq + Hash>(a: &TreeNode<T>, b: &TreeNode<T>) -> f32 {
            let mut hash_a = HashSet::new();
            let mut hash_b = HashSet::new();
            SubtreeHashDistance::subtree_hashes(a, &mut hash_a);
            SubtreeHashDistance::subtree_hashes(b, &mut hash_b);

            let common = hash_a.intersection(&hash_b).count() as f32;
            let total = hash_a.union(&hash_b).count() as f32;

            1.0 - (common / total) // Jaccard distance
        }

        let mut diff = 0_f32;

        for (a, b) in one.iter().zip(two.iter()) {
            let one_root = a.root();
            let two_root = b.root();

            diff += walk(one_root, two_root);
        }

        diff as f32 / one.len() as f32
    }
}
