use super::{Graph, GraphIterator, GraphNode, Tree};
use crate::expr::Expr;
use crate::node::Node;
use crate::{NodeCollection, NodeType, TreeNode};

pub trait Reduce<T> {
    type Input;
    type Output;

    fn reduce(&mut self, input: &Self::Input) -> Self::Output;
}

impl<T: Clone> Reduce<T> for Tree<T> {
    type Input = Vec<T>;
    type Output = T;

    fn reduce(&mut self, input: &Self::Input) -> Self::Output {
        let result = self.root_mut().map(|root| root.reduce(input));
        result.unwrap_or_else(|| panic!("Tree has no root node."))
    }
}

impl<T: Clone> Reduce<T> for TreeNode<T> {
    type Input = Vec<T>;
    type Output = T;

    fn reduce(&mut self, input: &Self::Input) -> Self::Output {
        fn eval<T: Clone>(node: &TreeNode<T>, curr_input: &Vec<T>) -> T {
            if node.is_leaf() {
                return node.cell.value.apply(&curr_input);
            } else {
                if let Some(children) = &node.children {
                    let mut inputs = Vec::with_capacity(children.len());
                    for child in children {
                        inputs.push(eval(child, &curr_input));
                    }

                    return node.cell.value.apply(&inputs);
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
pub struct GraphReducer<'a, T: Clone + Default> {
    graph: &'a Graph<T>,
    tracers: Vec<Tracer<T>>,
    order: Vec<usize>,
    outputs: Vec<T>,
}

impl<'a, T: Clone + Default> GraphReducer<'a, T> {
    pub fn new(graph: &'a Graph<T>) -> GraphReducer<'a, T> {
        let output_size = graph.nodes().iter().filter(|node| node.is_output()).count();

        GraphReducer {
            graph,
            tracers: graph
                .nodes()
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
                .iter_topological()
                .map(|node| node.index)
                .collect();
        }

        let mut output_index = 0;
        for index in &self.order {
            let node = self.graph.get(*index);
            if node.is_provider() {
                self.tracers[node.index].add_input(node.cell.value.apply(inputs));
            } else {
                for incoming in &node.incoming {
                    let arg = self.tracers[*incoming]
                        .result
                        .clone()
                        .unwrap_or_else(|| T::default());
                    self.tracers[node.index].add_input(arg);
                }
            }

            self.tracers[node.index].eval(node);

            if node.is_output() {
                self.outputs[output_index] = self.tracers[node.index].result.clone().unwrap();
                output_index += 1;
            }
        }

        self.outputs.clone()
    }
}

struct Tracer<T: Clone + Default> {
    pub input_size: usize,
    pub pending_idx: usize,
    pub args: Vec<T>,
    pub result: Option<T>,
    pub previous_result: Option<T>,
}

impl<T: Clone + Default> Tracer<T> {
    pub fn new(input_size: usize) -> Self {
        Tracer {
            input_size,
            pending_idx: 0,
            args: Vec::with_capacity(input_size),
            result: None,
            previous_result: None,
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
    pub fn eval(&mut self, node: &GraphNode<T>) {
        if self.pending_idx != self.input_size {
            panic!("Tracer is not ready to be evaluated.");
        }

        if !node.enabled {
            self.result = Some(T::default());
        }

        if node.is_provider() {
            self.result = Some(self.args[0].clone());
        } else {
            self.result = Some(node.cell.value.apply(&self.args));
        }

        // self.result = Some(node.cell.value.apply(&self.args));

        // self.result = match &node.value {
        //     Expr::Const(_, ref value) => Some(value.clone()),
        //     Expr::Fn(_, _, ref fn_ptr) => Some(fn_ptr(&self.args)),
        //     Expr::MutableConst(_, _, ref val, _, fn_ptr) => Some(fn_ptr(&self.args, val)),
        //     Expr::Var(_, _) => Some(self.args[0].clone()),
        // };

        self.pending_idx = 0;
        self.args.clear();
    }
}

fn input_size<T>(node: &GraphNode<T>) -> usize {
    if node.is_provider() {
        1
    } else {
        node.incoming.len()
    }

    // match node.node_type {
    //     NodeType::Input | NodeType::Link | NodeType::Leaf => 1,
    //     NodeType::Gate => *node.value.arity() as usize,
    //     _ => node.incoming.len(),
    // }
}

#[cfg(test)]
mod tests {
    use crate::{
        expr::{self},
        NodeCell,
    };

    use super::*;

    #[test]
    fn test_tree_reduce_simple() {
        let mut root = TreeNode::new(NodeCell::new(expr::add()));

        root.add_child(TreeNode::new(NodeCell::new(expr::value(1.0))));
        root.add_child(TreeNode::new(NodeCell::new(expr::value(2.0))));

        let result = root.reduce(&vec![]);

        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_tree_reduce_complex() {
        let mut root = TreeNode::new(NodeCell::new(expr::add()));

        let mut left = TreeNode::new(NodeCell::new(expr::mul()));
        left.add_child(TreeNode::new(NodeCell::new(expr::value(2.0))));
        left.add_child(TreeNode::new(NodeCell::new(expr::value(3.0))));

        let mut right = TreeNode::new(NodeCell::new(expr::add()));
        right.add_child(TreeNode::new(NodeCell::new(expr::value(2.0))));
        right.add_child(TreeNode::new(NodeCell::new(expr::var(0))));

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
