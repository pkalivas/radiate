use super::{Graph, GraphNode, NodeType, Tree, TreeNode};

use crate::ops::operation::Op;

pub trait Reduce<T> {
    type Input;
    type Output;

    fn reduce(&mut self, input: &Self::Input) -> Self::Output;
}

impl<T: Clone> Reduce<T> for Tree<Op<T>> {
    type Input = Vec<T>;
    type Output = T;

    fn reduce(&mut self, input: &Self::Input) -> Self::Output {
        let result = self.root_mut().map(|root| root.reduce(input));
        result.unwrap_or_else(|| panic!("Tree has no root node."))
    }
}

impl<T: Clone> Reduce<T> for TreeNode<Op<T>> {
    type Input = Vec<T>;
    type Output = T;

    fn reduce(&mut self, input: &Self::Input) -> Self::Output {
        fn eval<T: Clone>(node: &TreeNode<Op<T>>, curr_input: &Vec<T>) -> T {
            if node.is_leaf() {
                node.value().apply(curr_input)
            } else {
                if let Some(children) = &node.children() {
                    let mut inputs = Vec::with_capacity(children.len());

                    for child in *children {
                        inputs.push(eval(child, curr_input));
                    }

                    return node.value().apply(&inputs);
                }

                panic!("Node is not a leaf and has no children.");
            }
        }

        eval(self, input)
    }
}

/// `GraphReducer` is a struct that is used to evaluate a `Graph` of `Node`s. It uses the `GraphIterator`
/// to traverse the `Graph` in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the `Graph` and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
///
pub struct GraphReducer<'a, T>
where
    T: Clone + Default,
{
    graph: &'a Graph<Op<T>>,
    tracers: Vec<Tracer<T>>,
    order: Vec<usize>,
    outputs: Vec<T>,
}

impl<'a, T> GraphReducer<'a, T>
where
    T: Clone + Default,
{
    pub fn new(graph: &'a Graph<Op<T>>) -> GraphReducer<'a, T> {
        let output_size = graph
            .iter()
            .filter(|node| node.node_type() == NodeType::Output)
            .count();

        GraphReducer {
            graph,
            tracers: graph
                .iter()
                .map(|node| Tracer::new(input_size(node)))
                .collect::<Vec<Tracer<T>>>(),
            order: Vec::with_capacity(graph.len()),
            outputs: vec![T::default(); output_size],
        }
    }

    #[inline]
    pub fn reduce(&mut self, inputs: &[T]) -> Vec<T> {
        if self.order.is_empty() {
            self.order = self
                .graph
                .topological_iter()
                .map(|node| node.index())
                .collect();
        }

        let mut output_index = 0;
        for index in &self.order {
            let node = self.graph.get(*index);
            if node.node_type() == NodeType::Input {
                self.tracers[node.index()].add_input(inputs[node.index() % inputs.len()].clone());
            } else {
                for incoming in node.incoming() {
                    let arg = self.tracers[*incoming]
                        .result
                        .clone()
                        .unwrap_or_else(|| T::default());
                    self.tracers[node.index()].add_input(arg);
                }
            }

            self.tracers[node.index()].eval(node);

            if node.node_type() == NodeType::Output {
                self.outputs[output_index] = self.tracers[node.index()].result.clone().unwrap();
                output_index += 1;
            }
        }

        self.outputs.clone()
    }
}

struct Tracer<T>
where
    T: Clone,
{
    pub input_size: usize,
    pub pending_idx: usize,
    pub args: Vec<T>,
    pub result: Option<T>,
}

impl<T> Tracer<T>
where
    T: Clone + Default,
{
    pub fn new(input_size: usize) -> Self {
        Tracer {
            input_size,
            pending_idx: 0,
            args: Vec::with_capacity(input_size),
            result: None,
        }
    }

    pub fn add_input(&mut self, value: T) {
        if self.pending_idx == self.input_size {
            panic!("Tracer is not ready to accept more inputs.");
        }

        self.args.push(value);
        self.pending_idx += 1;
    }

    #[inline]
    pub fn eval(&mut self, node: &GraphNode<Op<T>>) {
        if self.pending_idx != self.input_size {
            panic!("Tracer is not ready to be evaluated.");
        }

        if !node.is_enabled() {
            self.result = Some(T::default());
        }

        self.result = match &node.value() {
            Op::Const(_, ref value) => Some(value.clone()),
            Op::Fn(_, _, ref fn_ptr) => Some(fn_ptr(&self.args)),
            Op::Var(_, _) => Some(self.args[0].clone()),
            Op::MutableConst {
                value, operation, ..
            } => Some(operation(&self.args, value)),
        };

        self.pending_idx = 0;
        self.args.clear();
    }
}

fn input_size<T>(node: &GraphNode<Op<T>>) -> usize
where
    T: Clone + Default,
{
    match node.node_type() {
        NodeType::Input => 1,
        _ => node.incoming().len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_reduce_simple() {
        let mut root = TreeNode::new(Op::add());

        root.add_child(TreeNode::new(Op::value(1.0)));
        root.add_child(TreeNode::new(Op::value(2.0)));

        let result = root.reduce(&vec![]);

        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_tree_reduce_complex() {
        let mut root = TreeNode::new(Op::add());

        let mut left = TreeNode::new(Op::mul());
        left.add_child(TreeNode::new(Op::value(2.0)));
        left.add_child(TreeNode::new(Op::value(3.0)));

        let mut right = TreeNode::new(Op::add());
        right.add_child(TreeNode::new(Op::value(2.0)));
        right.add_child(TreeNode::new(Op::var(0)));

        root.add_child(left);
        root.add_child(right);

        let result = root.reduce(&vec![1_f32]);
        assert_eq!(result, 9.0);

        let result = root.reduce(&vec![2_f32]);
        assert_eq!(result, 10.0);

        let result = root.reduce(&vec![3_f32]);
        assert_eq!(result, 11.0);
    }
}
