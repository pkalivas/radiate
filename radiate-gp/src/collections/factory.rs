use radiate::random_provider;

use crate::Arity;

use super::{GraphNode, NodeStore, NodeValue, TreeNode};

/// A trait for types that can be created from a given input.
///
/// TODO: Document this trait.
pub trait Factory<I, O> {
    fn new_instance(&self, input: I) -> O;
}

impl<T> Factory<(), T> for NodeValue<T>
where
    T: Factory<(), T> + Default,
{
    fn new_instance(&self, _: ()) -> T {
        match self {
            NodeValue::Bounded(value, _) => value.new_instance(()),
            NodeValue::Unbound(value) => value.new_instance(()),
        }
    }
}

impl<T> Factory<Option<Arity>, T> for NodeStore<T>
where
    T: Factory<(), T> + Default,
{
    fn new_instance(&self, input: Option<Arity>) -> T {
        let new_node = self.map(input, |values| {
            if values.is_empty() {
                return T::default();
            }

            let node_value = random_provider::choose(&values);
            node_value.new_instance(())
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        T::default()
    }
}

impl<T> Factory<(usize, Arity, Arity), GraphNode<T>> for NodeStore<T>
where
    T: Default + Clone,
{
    fn new_instance(&self, input: (usize, Arity, Arity)) -> GraphNode<T> {
        let (index, one, two) = input;

        let new_node = self.map(None, |values| {
            let possible_values = values
                .into_iter()
                .filter(|value| value.arity() == Some(one) || value.arity() == Some(two))
                .collect::<Vec<&NodeValue<T>>>();

            let node_value = random_provider::choose(&possible_values);

            match node_value {
                NodeValue::Bounded(value, arity) => {
                    return GraphNode::with_arity(index, value.clone(), *arity);
                }
                NodeValue::Unbound(value) => {
                    return GraphNode::new(index, value.clone());
                }
            }
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        GraphNode::new(index, T::default())
    }
}

impl<T: Default + Clone> Factory<(usize, Arity), GraphNode<T>> for NodeStore<T> {
    fn new_instance(&self, input: (usize, Arity)) -> GraphNode<T> {
        let (index, arity) = input;

        let new_node = self.map(Some(arity), |values| {
            let node_value = match arity {
                Arity::Zero => values[index % values.len()],
                _ => random_provider::choose(&values),
            };

            match node_value {
                NodeValue::Bounded(value, arity) => {
                    return GraphNode::with_arity(index, value.clone(), *arity);
                }
                NodeValue::Unbound(value) => {
                    return GraphNode::new(index, value.clone());
                }
            }
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        GraphNode::new(index, T::default())
    }
}

impl<T, F> Factory<(usize, F), GraphNode<T>> for NodeStore<T>
where
    T: Default + Clone,
    F: Fn(Arity) -> bool,
{
    fn new_instance(&self, input: (usize, F)) -> GraphNode<T> {
        let (index, filter) = input;
        let new_node = self.map(None, |values| {
            let values = values
                .into_iter()
                .filter(|value| match value.arity() {
                    Some(arity) => filter(arity),
                    None => false,
                })
                .collect::<Vec<&NodeValue<T>>>();

            let node_value = random_provider::choose(&values);

            match node_value {
                NodeValue::Bounded(value, arity) => {
                    return GraphNode::with_arity(index, value.clone(), *arity);
                }
                NodeValue::Unbound(value) => {
                    return GraphNode::new(index, value.clone());
                }
            }
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        GraphNode::new(0, T::default())
    }
}

impl<T: Clone + Default> Factory<Arity, TreeNode<T>> for NodeStore<T> {
    fn new_instance(&self, input: Arity) -> TreeNode<T> {
        let new_node = self.map(Some(input), |values| {
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

impl<T, F> Factory<F, TreeNode<T>> for NodeStore<T>
where
    T: Clone + Default,
    F: Fn(Arity) -> bool,
{
    fn new_instance(&self, input: F) -> TreeNode<T> {
        let new_node = self.map(None, |values| {
            let values = values
                .into_iter()
                .filter(|value| match value.arity() {
                    Some(arity) => input(arity),
                    None => false,
                })
                .collect::<Vec<&NodeValue<T>>>();

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
