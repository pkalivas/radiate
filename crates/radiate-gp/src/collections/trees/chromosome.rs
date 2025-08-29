use crate::{NodeStore, TreeNode};
use radiate_core::{Chromosome, Valid};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Debug, sync::Arc};

type Constraint<N> = Arc<dyn Fn(&N) -> bool>;

#[derive(Clone, Default)]
pub struct TreeChromosome<T> {
    nodes: Vec<TreeNode<T>>,
    store: Option<NodeStore<T>>,
    constraint: Option<Constraint<TreeNode<T>>>,
}

impl<T> TreeChromosome<T> {
    pub fn new(
        nodes: Vec<TreeNode<T>>,
        store: Option<NodeStore<T>>,
        constraint: Option<Constraint<TreeNode<T>>>,
    ) -> Self {
        TreeChromosome {
            nodes,
            store,
            constraint,
        }
    }

    pub fn root(&self) -> &TreeNode<T> {
        &self.nodes[0]
    }

    pub fn root_mut(&mut self) -> &mut TreeNode<T> {
        &mut self.nodes[0]
    }

    pub fn get_store(&self) -> Option<NodeStore<T>> {
        self.store.clone()
    }
}

impl<T> Chromosome for TreeChromosome<T>
where
    T: Clone + PartialEq,
{
    type Gene = TreeNode<T>;

    fn genes(&self) -> &[Self::Gene] {
        &self.nodes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.nodes
    }
}

impl<T> Valid for TreeChromosome<T> {
    fn is_valid(&self) -> bool {
        for gene in &self.nodes {
            if let Some(constraint) = &self.constraint {
                if !constraint(gene) {
                    return false;
                }
            } else if !gene.is_valid() {
                return false;
            }
        }

        true
    }
}

impl<T> From<Vec<TreeNode<T>>> for TreeChromosome<T> {
    fn from(nodes: Vec<TreeNode<T>>) -> Self {
        TreeChromosome {
            nodes,
            store: None,
            constraint: None,
        }
    }
}

impl<T> FromIterator<TreeNode<T>> for TreeChromosome<T> {
    fn from_iter<I: IntoIterator<Item = TreeNode<T>>>(iter: I) -> Self {
        let nodes: Vec<TreeNode<T>> = iter.into_iter().collect();
        TreeChromosome {
            nodes,
            store: None,
            constraint: None,
        }
    }
}

impl<T> AsRef<[TreeNode<T>]> for TreeChromosome<T> {
    fn as_ref(&self) -> &[TreeNode<T>] {
        &self.nodes
    }
}

impl<T> AsMut<[TreeNode<T>]> for TreeChromosome<T> {
    fn as_mut(&mut self) -> &mut [TreeNode<T>] {
        &mut self.nodes
    }
}

impl<T: PartialEq> PartialEq for TreeChromosome<T> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<T> IntoIterator for TreeChromosome<T> {
    type Item = TreeNode<T>;
    type IntoIter = std::vec::IntoIter<TreeNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<T: Debug> Debug for TreeChromosome<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeChromosome")
            .field("nodes", &self.nodes)
            .field("store", &self.store)
            .field("constraint", &self.constraint.is_some())
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
        (&self.nodes, &self.store).serialize(serializer)
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
        let (nodes, store): (Vec<TreeNode<T>>, Option<NodeStore<T>>) =
            Deserialize::deserialize(deserializer)?;

        Ok(TreeChromosome {
            nodes,
            store,
            constraint: None, // There is no good way to serialize constraints directly
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, NodeType};

    fn create_test_chromosome() -> TreeChromosome<i32> {
        let store = NodeStore::new();
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

        TreeChromosome::new(vec![root], Some(store), None)
    }

    #[test]
    fn test_new_chromosome() {
        let chromosome = TreeChromosome::new(vec![TreeNode::new(42)], None, None);

        assert_eq!(chromosome.nodes.len(), 1);
        assert_eq!(chromosome.store, None);
        assert!(chromosome.constraint.is_none());
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
        let constraint = Arc::new(|node: &TreeNode<i32>| node.value() % 2 == 0);
        let chromosome = TreeChromosome::new(
            vec![TreeNode::with_children(
                2,
                vec![TreeNode::new(4), TreeNode::new(6)],
            )],
            None,
            Some(constraint.clone()),
        );

        assert!(chromosome.is_valid());

        let invalid_chromosome = TreeChromosome::new(
            vec![TreeNode::with_children(
                1,
                vec![TreeNode::new(4), TreeNode::new(6)],
            )],
            None,
            Some(constraint),
        );
        assert!(!invalid_chromosome.is_valid());
    }

    #[test]
    fn test_partial_eq() {
        let chromosome1 = create_test_chromosome();
        let chromosome2 = create_test_chromosome();
        let chromosome3 = TreeChromosome::new(vec![TreeNode::new(42)], None, None);

        assert_eq!(chromosome1, chromosome2);
        assert_ne!(chromosome1, chromosome3);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_basic() {
        let chromosome = create_test_chromosome();

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome.nodes, deserialized.nodes);
        assert!(deserialized.store.is_some());

        let store = deserialized.store.unwrap();

        assert!(store.contains_type(NodeType::Vertex));
        assert!(store.contains_type(NodeType::Leaf));
        assert!(deserialized.constraint.is_none());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_with_complex_type() {
        use crate::Op;
        let store = NodeStore::new();
        store.insert(NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]);
        store.insert(NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]);

        let root = TreeNode::with_children(
            Op::add(),
            vec![
                TreeNode::with_children(
                    Op::mul(),
                    vec![
                        TreeNode::new(Op::constant(1.0)),
                        TreeNode::new(Op::constant(2.0)),
                    ],
                ),
                TreeNode::new(Op::constant(3.0)),
            ],
        );

        let chromosome = TreeChromosome::new(vec![root], Some(store), None);

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<Op<f32>> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome.nodes, deserialized.nodes);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_empty() {
        let chromosome: TreeChromosome<i32> = TreeChromosome::new(vec![], None, None);

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_with_constraint() {
        let constraint = Arc::new(|node: &TreeNode<i32>| node.value() > &0);
        let chromosome = TreeChromosome::new(vec![TreeNode::new(42)], None, Some(constraint));

        let serialized = serde_json::to_string(&chromosome).unwrap();
        let deserialized: TreeChromosome<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chromosome.nodes, deserialized.nodes);
        assert!(deserialized.constraint.is_none());
    }
}
