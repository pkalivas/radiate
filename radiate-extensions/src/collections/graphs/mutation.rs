use radiate::Valid;

use super::transaction::GraphTransaction;
use super::{Graph, NodeType};
use crate::ops::Arity;
use crate::{CellStore, Factory, GraphMutator, NodeCell};

// updated GraphMutator implementation
impl GraphMutator {
    pub fn add_node<C: NodeCell + Clone + Default + PartialEq>(
        &self,
        graph: &mut Graph<C>,
        node_type: &NodeType,
        factory: &CellStore<C>,
        recurrent: bool,
    ) -> bool {
        let mut transaction = GraphTransaction::new(graph);

        if !self.try_add_node(&mut transaction, node_type, factory, recurrent) {
            transaction.rollback();
            return false;
        }

        true
    }

    fn try_add_node<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        node_type: &NodeType,
        factory: &CellStore<C>,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let source_node_index = transaction.as_ref().random_source_node().index();
        let target_node_index = transaction.as_ref().random_target_node().index();

        let source_node_type = transaction.as_ref()[source_node_index].node_type();

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            if is_recurrent {
                self.try_backward_edge_insertion(
                    transaction,
                    source_node_index,
                    target_node_index,
                    node_type,
                    factory,
                )
            } else {
                self.try_edge_insertion(
                    transaction,
                    source_node_index,
                    target_node_index,
                    node_type,
                    factory,
                )
            }
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

    fn try_edge_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let new_source_edge_index = transaction.as_ref().len();
        let new_node_index = transaction.as_ref().len() + 1;
        let new_target_edge_index = transaction.as_ref().len() + 2;

        let source_node = transaction.as_ref()[source_node].index();
        let target_node = transaction.as_ref()[target_node].index();

        if transaction.as_ref()[target_node].is_locked() {
            let edge = factory.new_instance((
                new_source_edge_index,
                transaction.as_ref()[source_node].node_type(),
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
                transaction.as_ref()[source_node].node_type(),
            ));
            let new_node = factory.new_instance((new_node_index, *node_type));
            let new_target_edge = factory.new_instance((
                new_target_edge_index,
                transaction.as_ref()[target_node].node_type(),
            ));

            transaction.add_node(new_source_edge);
            transaction.add_node(new_node);
            transaction.add_node(new_target_edge);

            transaction.attach(source_node, new_source_edge_index);
            transaction.attach(new_source_edge_index, new_node_index);
            transaction.attach(new_node_index, new_target_edge_index);
            transaction.attach(new_target_edge_index, target_node);
        }

        self.complete_node_arity(transaction, new_node_index, false)
    }

    fn try_backward_edge_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_idx: usize,
        target_idx: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let new_source_edge_index = transaction.as_ref().len();
        let new_node_index = transaction.as_ref().len() + 1;
        let new_target_edge_index = transaction.as_ref().len() + 2;
        let recurrent_edge_index = transaction.as_ref().len() + 3;

        // Get node info before mutations
        let source_node_type = transaction.as_ref()[source_idx].node_type();
        let source_is_recurrent = transaction.as_ref()[source_idx].is_recurrent();
        let source_incoming = transaction.as_ref()[source_idx].incoming().clone();
        let source_outgoing = transaction.as_ref()[source_idx].outgoing().clone();
        let target_is_locked = transaction.as_ref()[target_idx].is_locked();

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            let incoming_idx = *source_incoming.iter().next().unwrap();
            let outgoing_idx = *source_outgoing.iter().next().unwrap();

            if target_is_locked {
                let new_node = factory.new_instance((new_node_index, *node_type));

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
                let new_node = factory.new_instance((new_node_index, *node_type));

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
                let new_node = factory.new_instance((new_node_index, *node_type));

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

            self.complete_node_arity(transaction, new_node_index, true)
        } else {
            if !&transaction
                .as_ref()
                .can_connect(source_idx, target_idx, true)
            {
                return false;
            }

            let new_node = factory.new_instance((new_node_index, *node_type));

            transaction.add_node(new_node);

            transaction.attach(source_idx, new_node_index);
            transaction.attach(new_node_index, target_idx);
            transaction.detach(source_idx, target_idx);

            self.complete_node_arity(transaction, new_node_index, true)
        }
    }

    fn try_normal_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        if !&transaction
            .as_ref()
            .can_connect(source_node, target_node, is_recurrent)
        {
            return false;
        }

        let new_node = factory.new_instance((transaction.as_ref().len(), *node_type));

        let node_index = transaction.add_node(new_node);
        transaction.attach(source_node, node_index);
        transaction.attach(node_index, target_node);
        transaction.detach(source_node, target_node);

        self.complete_node_arity(transaction, node_index, is_recurrent)
    }

    fn complete_node_arity<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        node_index: usize,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let arity = transaction.as_ref()[node_index].value.arity();

        match arity {
            Arity::Any | Arity::Zero => {
                transaction.set_cycles();
                return transaction.as_ref().is_valid();
            }
            Arity::Exact(arity) => {
                for _ in 0..arity - 1 {
                    let other_source_node = transaction.as_ref().random_source_node();
                    if transaction.as_ref().can_connect(
                        other_source_node.index(),
                        node_index,
                        is_recurrent,
                    ) {
                        transaction.attach(other_source_node.index(), node_index);
                    }
                }
            }
        }

        transaction.set_cycles();
        transaction.as_ref().is_valid()
    }
}
