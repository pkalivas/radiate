use super::{Graph, Tree};
use crate::expr::Expr;
use crate::node::Node;
use crate::{NodeCollection, NodeType};

/// `GraphReducer` is a struct that is used to evaluate a `Graph` of `Node`s. It uses the `GraphIterator`
/// to traverse the `Graph` in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the `Graph` and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
///
pub struct GraphReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    graph: &'a Graph<T>,
    tracers: Vec<Tracer<T>>,
    order: Vec<usize>,
    outputs: Vec<T>,
}

impl<'a, T> GraphReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(graph: &'a Graph<T>) -> GraphReducer<'a, T> {
        let output_size = graph
            .iter()
            .filter(|node| node.node_type == NodeType::Output)
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
                .map(|node| node.index)
                .collect();
        }

        let mut output_index = 0;
        for index in &self.order {
            let node = self.graph.get(*index);
            if node.node_type == NodeType::Input {
                self.tracers[node.index].add_input(inputs[node.index].clone());
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

            if node.node_type == NodeType::Output {
                self.outputs[output_index] = self.tracers[node.index].result.clone().unwrap();
                output_index += 1;
            }
        }

        self.outputs.clone()
    }
}

// pub struct TreeReducer<'a, T>
// where
//     T: Clone + PartialEq + Default,
// {
//     nodes: &'a Tree<T>,
//     tracers: Vec<Tracer<T>>,
// }

// impl<'a, T> TreeReducer<'a, T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub fn new(nodes: &'a Tree<T>) -> TreeReducer<'a, T> {
//         TreeReducer {
//             nodes,
//             tracers: nodes
//                 .iter()
//                 .map(|node| Tracer::new(input_size(node)))
//                 .collect::<Vec<Tracer<T>>>(),
//         }
//     }

//     #[inline]
//     pub fn reduce(&mut self, inputs: &[T]) -> Vec<T> {
//         self.eval_recurrent(0, inputs, &self.nodes.nodes)
//     }

//     fn eval_recurrent(&mut self, index: usize, input: &[T], nodes: &[Node<T>]) -> Vec<T> {
//         let node = &nodes[index];

//         if node.node_type == NodeType::Input || node.node_type == NodeType::Leaf {
//             self.tracers[node.index].add_input(input[0].clone());
//             self.tracers[node.index].eval(node);
//             vec![self.tracers[node.index].result.clone().unwrap()]
//         } else {
//             for incoming in &node.outgoing {
//                 let arg = self.eval_recurrent(*incoming, input, nodes);
//                 self.tracers[node.index].add_input(arg[0].clone());
//             }

//             self.tracers[node.index].eval(node);
//             vec![self.tracers[node.index].result.clone().unwrap()]
//         }
//     }
// }

struct Tracer<T>
where
    T: Clone,
{
    pub input_size: usize,
    pub pending_idx: usize,
    pub args: Vec<T>,
    pub result: Option<T>,
    pub previous_result: Option<T>,
}

impl<T> Tracer<T>
where
    T: Clone + PartialEq + Default,
{
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
    pub fn eval(&mut self, node: &Node<T>) {
        if self.pending_idx != self.input_size {
            panic!("Tracer is not ready to be evaluated.");
        }

        if !node.enabled {
            self.result = Some(T::default());
        }

        self.previous_result = self.result.clone();
        self.result = match &node.value {
            Expr::Value(ref value) => Some(value.clone()),
            Expr::Const(_, ref value) => Some(value.clone()),
            Expr::Fn(_, _, ref fn_ptr) => Some(fn_ptr(&self.args)),
            Expr::MutableConst(_, _, ref val, _, fn_ptr) => Some(fn_ptr(&self.args, val)),
            Expr::Var(_, _) => Some(self.args[0].clone()),
        };

        self.pending_idx = 0;
        self.args.clear();
    }
}

fn input_size<T>(node: &Node<T>) -> usize
where
    T: Clone + PartialEq + Default,
{
    match node.node_type {
        NodeType::Input | NodeType::Link | NodeType::Leaf => 1,
        NodeType::Gate => node.value.arity() as usize,
        _ => node.incoming.len(),
    }
}
