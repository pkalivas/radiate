use super::{GraphNode, NodeStore, NodeType, NodeValue, TreeNode};
use crate::Arity;
use radiate_core::random_provider;

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
        self.map_by_type(input, |values| {
            random_provider::choose(&values).new_instance(())
        })
        .unwrap_or_default()
    }
}

impl<T: Default + Clone> Factory<(usize, NodeType), GraphNode<T>> for NodeStore<T> {
    fn new_instance(&self, (index, node_type): (usize, NodeType)) -> GraphNode<T> {
        self.map_by_type(node_type, |values| {
            let node_value = match node_type {
                NodeType::Input => &values[index % values.len()],
                _ => random_provider::choose(&values),
            };

            match node_value {
                NodeValue::Bounded(value, arity) => {
                    (index, node_type, value.clone(), *arity).into()
                }
                NodeValue::Unbound(value) => (index, node_type, value.clone()).into(),
            }
        })
        .unwrap_or(GraphNode::new(index, node_type, T::default()))
    }
}

impl<T, F> Factory<(usize, NodeType, F), GraphNode<T>> for NodeStore<T>
where
    T: Default + Clone,
    F: Fn(Arity) -> bool,
{
    fn new_instance(&self, (index, node_type, filter): (usize, NodeType, F)) -> GraphNode<T> {
        self.map(|values| {
            let mapped_values = values
                .into_iter()
                .filter(|value| match value {
                    NodeValue::Bounded(_, arity) => filter(*arity),
                    _ => false,
                })
                .collect::<Vec<&NodeValue<T>>>();

            if mapped_values.is_empty() {
                self.new_instance((index, node_type))
            } else {
                let node_value = random_provider::choose(&mapped_values);

                match node_value {
                    NodeValue::Bounded(value, arity) => {
                        GraphNode::with_arity(index, node_type, value.clone(), *arity)
                    }
                    NodeValue::Unbound(value) => GraphNode::new(index, node_type, value.clone()),
                }
            }
        })
        .unwrap_or(GraphNode::new(index, node_type, T::default()))
    }
}

impl<T: Clone + Default> Factory<NodeType, TreeNode<T>> for NodeStore<T> {
    fn new_instance(&self, input: NodeType) -> TreeNode<T> {
        self.map_by_type(input, |values| {
            let node_value = random_provider::choose(&values);

            match node_value {
                NodeValue::Bounded(value, arity) => TreeNode::with_arity(value.clone(), *arity),
                NodeValue::Unbound(value) => TreeNode::new(value.clone()),
            }
        })
        .unwrap_or(TreeNode::new(T::default()))
    }
}
