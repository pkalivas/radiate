use super::{Direction, Graph, GraphNode};
use radiate::Valid;
use std::{
    collections::HashSet,
    ops::{Deref, Index},
};

/// Represents a reversible change to the graph
#[derive(Debug)]
enum MutationStep {
    AddNode,
    AddEdge(usize, usize),
    RemoveEdge(usize, usize),
    DirectionChange {
        index: usize,
        previous_direction: Direction,
    },
}

/// Tracks changes and provides rollback capability
pub struct GraphTransaction<'a, T> {
    graph: &'a mut Graph<T>,
    steps: Vec<MutationStep>,
    effects: HashSet<usize>,
}

impl<'a, T> GraphTransaction<'a, T> {
    pub fn new(graph: &'a mut Graph<T>) -> Self {
        GraphTransaction {
            graph,
            steps: Vec::new(),
            effects: HashSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn add_node(&mut self, node: GraphNode<T>) -> usize {
        let index = self.graph.len();
        self.steps.push(MutationStep::AddNode);
        self.graph.push(node);
        self.effects.insert(index);
        index
    }

    pub fn attach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::AddEdge(from, to));
        self.graph.attach(from, to);
        self.effects.insert(from);
        self.effects.insert(to);
    }

    pub fn detach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::RemoveEdge(from, to));
        self.graph.detach(from, to);
        self.effects.insert(from);
        self.effects.insert(to);
    }

    pub fn change_direction(&mut self, index: usize, direction: Direction) {
        let previous_direction = self.graph[index].direction();
        self.steps.push(MutationStep::DirectionChange {
            index,
            previous_direction,
        });
        self.graph[index].set_direction(direction);
    }

    pub fn rollback(self) {
        // Reverse all changes in reverse order
        for step in self.steps.into_iter().rev() {
            match step {
                MutationStep::AddNode => {
                    self.graph.pop();
                }
                MutationStep::AddEdge(from, to) => {
                    self.graph.detach(from, to);
                }
                MutationStep::RemoveEdge(from, to) => {
                    self.graph.attach(from, to);
                }
                MutationStep::DirectionChange {
                    index,
                    previous_direction,
                    ..
                } => {
                    self.graph[index].set_direction(previous_direction);
                }
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.graph.is_valid()
    }

    pub fn set_cycles(&mut self) {
        let effects = self.effects.clone();

        for idx in effects {
            let node_cycles = self.graph.get_cycles(idx);

            if node_cycles.is_empty() {
                self.change_direction(idx, Direction::Forward);
            } else {
                for cycle_idx in node_cycles {
                    self.change_direction(cycle_idx, Direction::Backward);
                }
            }
        }
    }
}

impl<'a, T> AsRef<Graph<T>> for GraphTransaction<'a, T> {
    fn as_ref(&self) -> &Graph<T> {
        self.graph
    }
}

impl<'a, T> Index<usize> for GraphTransaction<'a, T> {
    type Output = GraphNode<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.graph[index]
    }
}

impl<'a, T> Deref for GraphTransaction<'a, T> {
    type Target = Graph<T>;

    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
