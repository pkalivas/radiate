use crate::{NodeType, Op, Reduce};

use super::{Graph, GraphNode};

/// `GraphReducer` is a struct that is used to evaluate a `Graph` of `Node`s. It uses the `GraphIterator`
/// to traverse the `Graph` in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the `Graph` and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
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
    /// Creates a new `GraphReducer` with the given `Graph`. Will cache the order of nodes in
    /// the `Graph` on the first iteration. On initialization the `GraphReducer` will cache the
    /// output size of the `Graph` to be used in the `reduce` method and create a vec of `Tracer`
    /// which will be used to evaluate the `Graph` in the `reduce` method.
    ///
    /// # Arguments
    /// * `graph` - The `Graph` to reduce.
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
}

/// Implements the `Reduce` trait for `GraphReducer`.
impl<'a, T: Clone + Default> Reduce for GraphReducer<'a, T> {
    type Input = Vec<T>;
    type Output = Vec<T>;

    /// Evaluates the `Graph` with the given input. Returns the output of the `Graph`.
    /// The `reduce` method will cache the order of nodes in the `Graph` on the first iteration.
    /// On subsequent iterations it will use the cached order to evaluate the `Graph` in the correct order.
    ///
    /// # Arguments
    /// * `input` - A `Vec` of `T` to evaluate the `Graph` with.
    ///
    ///  # Returns
    /// * A `Vec` of `T` which is the output of the `Graph`.
    #[inline]
    fn reduce(&mut self, input: &Self::Input) -> Self::Output {
        if self.order.is_empty() {
            self.order = self
                .graph
                .iter_topological()
                .map(|node| node.index())
                .collect();
        }

        let mut output_index = 0;
        for index in &self.order {
            let node = self.graph.get(*index);
            if node.node_type() == NodeType::Input {
                self.tracers[node.index()].add_input(input[node.index() % input.len()].clone());
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

/// `Tracer` is a struct that is used to evaluate a `GraphNode` in the correct order.
/// It uses the `args` field to store the inputs to the `GraphNode` and the `result`
/// field to store the result of the evaluation. The `GraphNode` itself is not stateful
/// and because different nodes of a `Graph` can be evaulated at different times, we need to keep
/// track of state some how. The `Tracer` is the solution to that.
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
