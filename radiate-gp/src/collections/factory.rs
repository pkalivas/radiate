use radiate::random_provider;

use crate::Arity;

use super::{GraphNode, NodeStore, NodeType, NodeValue, TreeNode};

/// A trait for types that can be created from a given input.
///
/// TODO: Document this trait.
pub trait Factory<I, O> {
    fn new_instance(&self, input: I) -> O;
}

impl<T> Factory<(), T> for NodeValue<T>
where
    T: Factory<(), T>,
{
    fn new_instance(&self, _: ()) -> T {
        match self {
            NodeValue::Bounded(value, _) => value.new_instance(()),
            NodeValue::Unbound(value) => value.new_instance(()),
        }
    }
}

impl<T> Factory<NodeType, T> for NodeStore<T>
where
    T: Factory<(), T> + Default,
{
    fn new_instance(&self, input: NodeType) -> T {
        let new_node = self.map_by_type(input, |values| {
            let node_value = random_provider::choose(&values);
            node_value.new_instance(())
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        T::default()
    }
}

impl<T: Default + Clone> Factory<(usize, NodeType), GraphNode<T>> for NodeStore<T> {
    fn new_instance(&self, input: (usize, NodeType)) -> GraphNode<T> {
        let (index, node_type) = input;

        let new_node = self.map_by_type(node_type, |values| {
            let node_value = match node_type {
                NodeType::Input => &values[index % values.len()],
                _ => random_provider::choose(&values),
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

impl<T, F> Factory<(usize, F), GraphNode<T>> for NodeStore<T>
where
    T: Default + Clone,
    F: Fn(Arity) -> bool,
{
    fn new_instance(&self, input: (usize, F)) -> GraphNode<T> {
        let (index, filter) = input;
        let new_node = self.map(|values| {
            let mapped_values = values
                .iter()
                .filter(|value| match value {
                    NodeValue::Bounded(_, arity) => filter(*arity),
                    _ => false,
                })
                .collect::<Vec<&NodeValue<T>>>();

            let node_value = random_provider::choose(&mapped_values);

            match node_value {
                NodeValue::Bounded(value, arity) => {
                    return GraphNode::new_wi(index, value.clone(), *arity);
                }
                NodeValue::Unbound(value) => {
                    return GraphNode::new_wi(index, value.clone(), Arity::Any);
                }
            }
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        GraphNode::new(index, NodeType::Vertex, T::default())

        // let new_node = self.filter_map

        // match node_value {
        //     NodeValue::Bounded(value, arity) => {
        //         return GraphNode::with_arity(index, value.clone(), *arity);
        //     }
        //     NodeValue::Unbound(value) => {
        //         return GraphNode::new(index, value.clone());
        //     }
        // }
        // });

        // if let Some(new_value) = new_node {
        //     return new_value;
        // }

        // GraphNode::new(0, T::default())
    }
}

impl<T: Clone + Default> Factory<NodeType, TreeNode<T>> for NodeStore<T> {
    fn new_instance(&self, input: NodeType) -> TreeNode<T> {
        let new_node = self.map_by_type(input, |values| {
            let node_value = random_provider::choose(&values);

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
