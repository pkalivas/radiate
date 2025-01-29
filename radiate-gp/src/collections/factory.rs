use radiate::random_provider;

use super::{GraphNode, NodeStore, NodeType, NodeValue, TreeNode};

/// A trait for types that can be created from a given input.
///
/// TODO: Document this trait.
pub trait Factory<T> {
    type Input;
    fn new_instance(&self, input: Self::Input) -> T;
}

impl<T> Factory<T> for NodeValue<T>
where
    T: Factory<T, Input = ()>,
{
    type Input = ();

    fn new_instance(&self, _: Self::Input) -> T {
        match self {
            NodeValue::Bounded(value, _) => value.new_instance(()),
            NodeValue::Unbound(value) => value.new_instance(()),
        }
    }
}

impl<T> Factory<T> for NodeStore<T>
where
    T: Factory<T, Input = ()> + Default,
{
    type Input = NodeType;

    fn new_instance(&self, input: Self::Input) -> T {
        let new_node = self.map(input, |values| {
            let node_value = random_provider::choose(values);
            node_value.new_instance(())
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        T::default()
    }
}

impl<T: Default + Clone> Factory<GraphNode<T>> for NodeStore<T> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<T> {
        let (index, node_type) = input;

        let new_node = self.map(node_type, |values| {
            let node_value = match node_type {
                NodeType::Input => &values[index % values.len()],
                _ => random_provider::choose(values),
            };

            match node_value {
                NodeValue::Bounded(value, arity) => {
                    return GraphNode::with_arity(index, node_type, value.clone(), *arity);
                }
                NodeValue::Unbound(value) => {
                    return GraphNode::new(index, node_type, value.clone());
                }
            }
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        GraphNode::new(index, node_type, T::default())
    }
}

impl<T: Clone + Default> Factory<TreeNode<T>> for NodeStore<T> {
    type Input = NodeType;

    fn new_instance(&self, input: Self::Input) -> TreeNode<T> {
        let new_node = self.map(input, |values| {
            let node_value = random_provider::choose(values);

            match node_value {
                NodeValue::Bounded(value, arity) => {
                    return TreeNode::with_arity(value.clone(), *arity);
                }
                NodeValue::Unbound(value) => {
                    return TreeNode::new(value.clone());
                }
            }
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        TreeNode::new(T::default())
    }
}
