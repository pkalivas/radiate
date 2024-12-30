use super::{Direction, Graph, GraphNode};
use crate::ops::Arity;
use crate::{graphs, Factory, GraphMutator, NodeFactory, NodeType};
use radiate::Valid;

/// Represents a reversible change to the graph
#[derive(Debug)]
enum MutationStep {
    AddNode,
    AddEdge(usize, usize),
    RemoveEdge(usize, usize),
}

/// Tracks changes and provides rollback capability
pub struct GraphTransaction<'a, T>
where
    T: Clone + Default + PartialEq,
{
    graph: &'a mut Graph<T>,
    steps: Vec<MutationStep>,
    starting_len: usize,
}

impl<'a, T: Clone + Default + PartialEq> GraphTransaction<'a, T> {
    pub fn new(graph: &'a mut Graph<T>) -> Self {
        let starting_len = graph.nodes.len();
        Self {
            graph,
            steps: Vec::new(),
            starting_len,
        }
    }

    pub fn add_node(&mut self, node: GraphNode<T>) -> usize {
        let index = self.graph.nodes.len();
        self.steps.push(MutationStep::AddNode);
        self.graph.nodes.push(node);
        index
    }

    pub fn attach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::AddEdge(from, to));
        self.graph.attach(from, to);
    }

    pub fn detach(&mut self, from: usize, to: usize) {
        self.steps.push(MutationStep::RemoveEdge(from, to));
        self.graph.detach(from, to);
    }

    pub fn commit(self) -> bool {
        true // Transaction succeeded
    }

    pub fn rollback(self) {
        // Reverse all changes in reverse order
        for step in self.steps.into_iter().rev() {
            match step {
                MutationStep::AddNode => {
                    self.graph.nodes.pop();
                }
                MutationStep::AddEdge(from, to) => {
                    self.graph.detach(from, to);
                }
                MutationStep::RemoveEdge(from, to) => {
                    self.graph.attach(from, to);
                }
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.graph.is_valid()
    }
}

// updated GraphMutator implementation
impl<T: Clone + PartialEq + Default> GraphMutator<T> {
    pub fn add_forward_node(
        &self,
        graph: &mut Graph<T>,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
    ) -> bool {
        let mut transaction = GraphTransaction::new(graph);

        if !self.try_add_node(&mut transaction, node_type, factory, false) {
            transaction.rollback();
            return false;
        }

        transaction.commit()
    }

    pub fn add_backward_node(
        &self,
        graph: &mut Graph<T>,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
    ) -> bool {
        let mut transaction = GraphTransaction::new(graph);

        if !self.try_add_backward_node(&mut transaction, node_type, factory, true) {
            transaction.rollback();
            return false;
        }

        transaction.commit()
    }

    fn try_add_backward_node(
        &self,
        transaction: &mut GraphTransaction<T>,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
        is_recurrent: bool,
    ) -> bool {
        let source_node_index = graphs::random_source_node(&transaction.graph.nodes).index;
        let target_node_index = graphs::random_target_node(&transaction.graph.nodes).index;

        let source_node_type = transaction.graph.nodes[source_node_index].node_type;

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            self.try_backward_edge_insertion(
                transaction,
                source_node_index,
                target_node_index,
                node_type,
                factory,
            )
        } else {
            self.handle_normal_node_backward(
                transaction,
                source_node_index,
                target_node_index,
                node_type,
                factory,
            )
        }
    }

    fn try_add_node(
        &self,
        transaction: &mut GraphTransaction<T>,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
        is_recurrent: bool,
    ) -> bool {
        let source_node_index = graphs::random_source_node(&transaction.graph.nodes).index;
        let target_node_index = graphs::random_target_node(&transaction.graph.nodes).index;

        let source_node_type = transaction.graph.nodes[source_node_index].node_type;

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            self.try_edge_insertion(
                transaction,
                source_node_index,
                target_node_index,
                node_type,
                factory,
                is_recurrent,
            )
        } else {
            self.try_normal_insertion(
                transaction,
                source_node_index,
                target_node_index,
                node_type,
                factory,
                is_recurrent,
            )
        }
    }

    fn try_edge_insertion(
        &self,
        transaction: &mut GraphTransaction<T>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
        is_recurrent: bool,
    ) -> bool {
        let new_source_edge_index = transaction.graph.nodes.len();
        let new_node_index = transaction.graph.nodes.len() + 1;
        let new_target_edge_index = transaction.graph.nodes.len() + 2;

        let source_node = transaction.graph.nodes[source_node].index;
        let target_node = transaction.graph.nodes[target_node].index;

        if graphs::is_locked(&transaction.graph.nodes[target_node]) {
            let edge = factory.new_instance((
                new_source_edge_index,
                transaction.graph.nodes[source_node].node_type,
            ));
            let new_node = factory.new_instance((new_node_index, *node_type));

            let edge_index = transaction.add_node(edge);
            let node_index = transaction.add_node(new_node);

            transaction.attach(source_node, node_index);
            transaction.attach(node_index, edge_index);
            transaction.attach(edge_index, target_node);
            transaction.detach(source_node, target_node);
        } else {
            let new_source_edge = factory.new_instance((
                new_source_edge_index,
                transaction.graph.nodes[source_node].node_type,
            ));
            let new_node = factory.new_instance((new_node_index, *node_type));
            let new_target_edge = factory.new_instance((
                new_target_edge_index,
                transaction.graph.nodes[target_node].node_type,
            ));

            transaction.add_node(new_source_edge);
            transaction.add_node(new_node);
            transaction.add_node(new_target_edge);

            transaction.attach(source_node, new_source_edge_index);
            transaction.attach(new_source_edge_index, new_node_index);
            transaction.attach(new_node_index, new_target_edge_index);
            transaction.attach(new_target_edge_index, target_node);
        }

        self.complete_node_arity(transaction, new_node_index, factory)
    }

    fn try_normal_insertion(
        &self,
        transaction: &mut GraphTransaction<T>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
        is_recurrent: bool,
    ) -> bool {
        let new_node_index = transaction.graph.nodes.len();
        let new_node = factory.new_instance((new_node_index, *node_type));

        let node_index = transaction.add_node(new_node);
        transaction.attach(source_node, node_index);
        transaction.attach(node_index, target_node);
        transaction.detach(source_node, target_node);

        self.complete_node_arity(transaction, node_index, factory)
    }

    fn try_backward_edge_insertion(
        &self,
        transaction: &mut GraphTransaction<T>,
        source_idx: usize,
        target_idx: usize,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
    ) -> bool {
        let new_source_edge_index = transaction.graph.nodes.len();
        let new_node_index = transaction.graph.nodes.len() + 1;
        let new_target_edge_index = transaction.graph.nodes.len() + 2;
        let recurrent_edge_index = transaction.graph.nodes.len() + 3;

        // Get node info before mutations
        let source_node_type = transaction.graph.nodes[source_idx].node_type;
        let source_is_recurrent = transaction.graph.nodes[source_idx].is_recurrent();
        let source_incoming = transaction.graph.nodes[source_idx].incoming.clone();
        let source_outgoing = transaction.graph.nodes[source_idx].outgoing.clone();
        let target_is_locked = graphs::is_locked(&transaction.graph.nodes[target_idx]);

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            let incoming_idx = *source_incoming.iter().next().unwrap();
            let outgoing_idx = *source_outgoing.iter().next().unwrap();

            if target_is_locked {
                let mut new_node = factory.new_instance((new_node_index, *node_type));
                new_node.direction = Direction::Backward;
                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);

                transaction.attach(incoming_idx, new_node_index);
                transaction.attach(new_node_index, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
                transaction.detach(incoming_idx, outgoing_idx);
            } else if !source_is_recurrent {
                let mut new_node = factory.new_instance((new_node_index, *node_type));
                new_node.direction = Direction::Backward;
                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));
                let recurrent_edge = factory.new_instance((recurrent_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);
                transaction.add_node(recurrent_edge);

                transaction.attach(incoming_idx, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
                transaction.attach(recurrent_edge_index, new_node_index);
                transaction.attach(new_node_index, recurrent_edge_index);
            } else {
                let mut new_node = factory.new_instance((new_node_index, *node_type));
                new_node.direction = Direction::Backward;
                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);

                transaction.attach(incoming_idx, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
            }

            self.complete_node_arity(transaction, new_node_index, factory)
        } else {
            if !graphs::can_connect(&transaction.graph.nodes, source_idx, target_idx, true) {
                return false;
            }

            let mut new_node = factory.new_instance((new_node_index, *node_type));
            new_node.direction = Direction::Backward;
            transaction.add_node(new_node);

            transaction.attach(source_idx, new_node_index);
            transaction.attach(new_node_index, target_idx);
            transaction.detach(source_idx, target_idx);

            self.complete_node_arity(transaction, new_node_index, factory)
        }
    }

    fn handle_normal_node_backward(
        &self,
        transaction: &mut GraphTransaction<T>,
        source_idx: usize,
        target_idx: usize,
        node_type: &NodeType,
        factory: &NodeFactory<T>,
    ) -> bool {
        if !graphs::can_connect(&transaction.graph.nodes, source_idx, target_idx, true) {
            return false;
        }

        let mut new_node = factory.new_instance((transaction.graph.nodes.len(), *node_type));
        new_node.direction = Direction::Backward;
        let new_idx = transaction.add_node(new_node);

        transaction.attach(source_idx, new_idx);
        transaction.attach(new_idx, target_idx);
        transaction.detach(source_idx, target_idx);

        self.complete_node_arity(transaction, new_idx, factory)
    }

    fn complete_node_arity(
        &self,
        transaction: &mut GraphTransaction<T>,
        node_index: usize,
        factory: &NodeFactory<T>,
    ) -> bool {
        let arity = transaction.graph.nodes[node_index].value.arity();

        match arity {
            Arity::Any | Arity::Zero => {
                return true;
            }
            Arity::Exact(arity) => {
                for _ in 0..arity - 1 {
                    let other_source_node = graphs::random_source_node(&transaction.graph.nodes);
                    if graphs::can_connect(
                        &transaction.graph.nodes,
                        other_source_node.index,
                        node_index,
                        false,
                    ) {
                        transaction.attach(other_source_node.index, node_index);
                    }
                }
            }
        }

        transaction.is_valid()

        // let node = collection.get(new_node_index);
        // match node.value.arity() {
        //     Arity::Any | Arity::Zero => {
        //         return Some(collection.into_iter().collect::<Vec<GraphNode<T>>>());
        //     }
        //     Arity::Exact(arity) => {
        //         for _ in 0..arity - 1 {
        //             let other_source_node = graphs::random_source_node(collection.as_ref());
        //             if graphs::can_connect(
        //                 collection.as_ref(),
        //                 other_source_node.index,
        //                 new_node_index,
        //                 recurrent,
        //             ) {
        //                 collection.attach(other_source_node.index, new_node_index);
        //             }
        //         }
        //     }
        // }
        //
        // if !collection.is_valid() {
        //     return None;
        // }
    }
}
