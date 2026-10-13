use crate::{Factory, NodeSet, Tree, TreeIterator, TreeNode};
use radiate_core::{Chromosome, Valid};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Debug, hash::Hash};

pub(crate) const DEFAULT_MAX_SIZE: u32 = 200;

#[derive(Clone, Default)]
pub struct TreeChromosome<T> {
    root: TreeNode<T>,
    store: Option<NodeSet<T>>,
    max_size: Option<u32>,
}

impl<T> TreeChromosome<T> {
    pub fn new(root: TreeNode<T>, store: Option<NodeSet<T>>, max_size: Option<u32>) -> Self {
        TreeChromosome {
            root,
            store,
            max_size,
        }
    }

    pub fn root(&self) -> &TreeNode<T> {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut TreeNode<T> {
        &mut self.root
    }

    pub fn get_store(&self) -> Option<NodeSet<T>> {
        self.store.clone()
    }
}

impl<T> Chromosome for TreeChromosome<T>
where
    T: Clone + PartialEq,
{
    type Gene = TreeNode<T>;

    fn get(&self, index: usize) -> Option<&Self::Gene> {
        if index != 0 {
            return None;
        }

        Some(&self.root)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Gene> {
        if index != 0 {
            return None;
        }

        Some(&mut self.root)
    }

    fn set(&mut self, index: usize, gene: Self::Gene) {
        if index != 0 {
            return;
        }

        self.root = gene;
    }

    fn iter(&self) -> impl Iterator<Item = &Self::Gene> {
        std::iter::once(&self.root)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Gene> {
        std::iter::once(&mut self.root)
    }

    fn len(&self) -> usize {
        1
    }
}

impl<T> Valid for TreeChromosome<T> {
    fn is_valid(&self) -> bool {
        let max_size = self.max_size.unwrap_or(DEFAULT_MAX_SIZE);
        let mut valid = true;
        let mut count = 0_u32;

        for node in self.root.iter_breadth_first() {
            count += 1;
            valid &= node.arity_matches_children();

            if !valid || count > max_size {
                return false;
            }
        }

        true
    }
}

impl<T> Factory<Option<NodeSet<T>>, TreeChromosome<T>> for TreeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn new_instance(&self, input: Option<NodeSet<T>>) -> TreeChromosome<T> {
        input
            .or_else(|| self.store.clone())
            .map(|store| {
                TreeChromosome::new(
                    Tree::with_depth(self.root.height(), &store)
                        .take_root()
                        .unwrap_or_else(|| self.root.clone()),
                    Some(store.clone()),
                    self.max_size,
                )
            })
            .map(|chrom| {
                if chrom.len() != self.len() {
                    self.clone()
                } else {
                    chrom
                }
            })
            .unwrap_or_else(|| self.clone())
    }
}

impl<T> From<TreeNode<T>> for TreeChromosome<T> {
    fn from(node: TreeNode<T>) -> Self {
        TreeChromosome {
            root: node,
            store: None,
            max_size: Some(DEFAULT_MAX_SIZE),
        }
    }
}

impl<T> AsRef<TreeNode<T>> for TreeChromosome<T> {
    fn as_ref(&self) -> &TreeNode<T> {
        &self.root
    }
}

impl<T> AsMut<TreeNode<T>> for TreeChromosome<T> {
    fn as_mut(&mut self) -> &mut TreeNode<T> {
        &mut self.root
    }
}

impl<T: PartialEq> PartialEq for TreeChromosome<T> {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl<T> IntoIterator for TreeChromosome<T> {
    type Item = TreeNode<T>;
    type IntoIter = std::vec::IntoIter<TreeNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.root].into_iter()
    }
}

impl<T: Hash> Hash for TreeChromosome<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root.hash(state);
    }
}

impl<T: Debug> Debug for TreeChromosome<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeChromosome")
            .field("nodes", &self.root)
            .field("store", &self.store)
            .field("max_size", &self.max_size)
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for TreeChromosome<T>
where
    T: Serialize + Clone + PartialEq,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&self.root, &self.store).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for TreeChromosome<T>
where
    T: Deserialize<'de> + Clone + PartialEq,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (node, store): (TreeNode<T>, Option<NodeSet<T>>) =
            Deserialize::deserialize(deserializer)?;

        Ok(TreeChromosome {
            root: node,
            store,
            max_size: Some(DEFAULT_MAX_SIZE),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, NodeType};

    fn create_test_chromosome() -> TreeChromosome<i32> {
        let mut store = NodeSet::new();
        store.insert(NodeType::Vertex, vec![1, 2, 3]);
        store.insert(NodeType::Leaf, vec![4, 5]);

        // Create a simple tree: 1 -> (2 -> (4, 5), 3)
        let root = TreeNode::with_children(
            1,
            vec![
                TreeNode::with_children(2, vec![TreeNode::new(4), TreeNode::new(5)]),
                TreeNode::new(3),
            ],
        );

        TreeChromosome::new(root, Some(store), None)
    }

    #[test]
    fn test_new_chromosome() {
        let chromosome = TreeChromosome::new(TreeNode::new(42), None, None);

        assert_eq!(chromosome.store, None);
        assert!(chromosome.max_size.is_none());
        assert_eq!(chromosome.root().value(), &42);
    }

    #[test]
    fn test_root_access() {
        let chromosome = create_test_chromosome();

        assert_eq!(chromosome.root().value(), &1);

        let mut chromosome = chromosome;
        let root_mut = chromosome.root_mut();

        assert_eq!(root_mut.value(), &1);

        *root_mut.value_mut() = 10;

        assert_eq!(chromosome.root().value(), &10);
    }

    #[test]
    fn test_store_access() {
        let chromosome = create_test_chromosome();
        let store = chromosome.get_store();

        assert!(store.is_some());

        let store = store.unwrap();
        assert!(store.contains_type(NodeType::Vertex));
        assert!(store.contains_type(NodeType::Leaf));
    }

    #[test]
    fn test_constraint_validation() {
        let chromosome = TreeChromosome::new(
            TreeNode::with_children(2, vec![TreeNode::new(4), TreeNode::new(6)]),
            None,
            Some(DEFAULT_MAX_SIZE),
        );

        assert!(chromosome.is_valid());

        let invalid_chromosome = TreeChromosome::new(
            TreeNode::with_children(1, vec![TreeNode::new(4), TreeNode::new(6)]),
            None,
            Some(1),
        );
        assert!(!invalid_chromosome.is_valid());
    }

    #[test]
    fn test_partial_eq() {
        let chromosome1 = create_test_chromosome();
        let chromosome2 = create_test_chromosome();
        let chromosome3 = TreeChromosome::new(TreeNode::new(42), None, None);

        assert_eq!(chromosome1, chromosome2);
        assert_ne!(chromosome1, chromosome3);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_basic() {
        let chromosome = create_test_chromosome();

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome.root, deserialized.root);
        assert!(deserialized.store.is_some());

        let store = deserialized.store.unwrap();

        assert!(store.contains_type(NodeType::Vertex));
        assert!(store.contains_type(NodeType::Leaf));
        assert!(deserialized.max_size.is_some());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_with_complex_type() {
        use crate::Op;
        let mut store = NodeSet::new();
        store.insert(NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]);
        store.insert(NodeType::Leaf, vec![Op::one(), Op::two()]);

        let root = TreeNode::with_children(
            Op::add(),
            vec![
                TreeNode::with_children(
                    Op::mul(),
                    vec![TreeNode::new(Op::one()), TreeNode::new(Op::two())],
                ),
                TreeNode::new(Op::three()),
            ],
        );

        let chromosome = TreeChromosome::new(root, Some(store), None);

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<Op<f32>> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome.root, deserialized.root);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_empty() {
        let chromosome: TreeChromosome<i32> = TreeChromosome::new(TreeNode::new(0), None, None);

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_with_constraint() {
        let chromosome = TreeChromosome::new(TreeNode::new(42), None, Some(DEFAULT_MAX_SIZE));

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome.root, deserialized.root);
        assert!(deserialized.max_size.is_some());
    }
}
