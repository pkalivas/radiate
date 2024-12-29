use super::{Graph, GraphNode};
use crate::architects::node_collections::node_factory::NodeFactory;
use crate::NodeType;
use std::collections::BTreeMap;
use uuid::Uuid;

enum ConnectTypes {
    OneToOne,
    OneToMany,
    ManyToOne,
    AllToAll,
    AllToAllSelf,
}

struct Relationship<'a> {
    source_id: &'a Uuid,
    target_id: &'a Uuid,
}

#[derive(Default)]
pub struct GraphArchitect<'a, T>
where
    T: Clone,
{
    factory: Option<&'a NodeFactory<T>>,
    nodes: BTreeMap<&'a Uuid, &'a GraphNode<T>>,
    node_order: BTreeMap<usize, &'a Uuid>,
    relationships: Vec<Relationship<'a>>,
}

impl<'a, T> GraphArchitect<'a, T>
where
    T: Clone,
{
    pub fn new(factory: &'a NodeFactory<T>) -> Self {
        GraphArchitect {
            factory: Some(factory),
            nodes: BTreeMap::new(),
            node_order: BTreeMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn one_to_one<C: AsRef<[GraphNode<T>]>>(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::OneToOne, one, two);
        self
    }

    pub fn one_to_many<C: AsRef<[GraphNode<T>]>>(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::OneToMany, one, two);
        self
    }

    pub fn many_to_one<C: AsRef<[GraphNode<T>]>>(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::ManyToOne, one, two);
        self
    }

    pub fn all_to_all<C: AsRef<[GraphNode<T>]>>(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::AllToAll, one, two);
        self
    }

    pub fn one_to_one_self<C: AsRef<[GraphNode<T>]>>(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::AllToAllSelf, one, two);
        self
    }

    pub fn insert<C: AsRef<[GraphNode<T>]>>(mut self, collection: &'a C) -> Self {
        self.attach(collection.as_ref());
        self
    }

    pub fn build(self) -> Graph<T>
    where
        T: Default,
    {
        let mut new_nodes = Vec::new();
        let mut node_id_index_map = BTreeMap::new();

        for (index, (_, node_id)) in self.node_order.iter().enumerate() {
            let node = self.nodes.get(node_id).unwrap();
            let new_node = GraphNode::new(index, node.node_type, node.value.clone());

            new_nodes.push(new_node);
            node_id_index_map.insert(node_id, index);
        }

        let mut new_collection = Graph::new(new_nodes);
        for rel in self.relationships {
            let source_idx = node_id_index_map.get(&rel.source_id).unwrap();
            let target_idx = node_id_index_map.get(&rel.target_id).unwrap();

            new_collection.attach(*source_idx, *target_idx);
        }

        let mut collection = new_collection.clone().set_cycles(Vec::new());

        for node in collection.get_nodes_mut() {
            if let Some(factory) = self.factory {
                let temp_node = factory.new_node(node.index, NodeType::Aggregate);

                match node.node_type() {
                    NodeType::Input => {
                        if !node.incoming().is_empty() {
                            node.node_type = NodeType::Aggregate;
                            node.value = temp_node.value.clone();
                        }
                    }
                    NodeType::Output => {
                        if !node.outgoing().is_empty() {
                            node.node_type = NodeType::Aggregate;
                            node.value = temp_node.value.clone();
                        }
                    }
                    _ => {}
                }
            }
        }

        collection
    }

    pub fn layer<C: AsRef<[GraphNode<T>]>>(&self, collections: Vec<&'a C>) -> Self {
        let mut conn = GraphArchitect::new(self.factory.unwrap());
        let mut previous = collections[0];

        for collection in collections.iter() {
            conn.attach((*collection).as_ref());
        }

        for i in 1..collections.len() {
            conn = conn.one_to_one(previous, collections[i]);
            previous = collections[i];
        }

        conn
    }

    pub fn attach(&mut self, group: &'a [GraphNode<T>]) {
        for node in group.iter() {
            if !self.nodes.contains_key(&node.id) {
                let node_id = &node.id;

                self.nodes.insert(node_id, node);
                self.node_order.insert(self.node_order.len(), node_id);

                for outgoing in group
                    .iter()
                    .filter(|item| node.outgoing().contains(&item.index))
                {
                    self.relationships.push(Relationship {
                        source_id: &node.id,
                        target_id: &outgoing.id,
                    });
                }
            }
        }
    }

    fn connect<C: AsRef<[GraphNode<T>]>>(
        &mut self,
        connection: ConnectTypes,
        one: &'a C,
        two: &'a C,
    ) {
        self.attach(one.as_ref());
        self.attach(two.as_ref());

        match connection {
            ConnectTypes::OneToOne => self.one_to_one_connect(one, two),
            ConnectTypes::OneToMany => self.one_to_many_connect(one, two),
            ConnectTypes::ManyToOne => self.many_to_one_connect(one, two),
            ConnectTypes::AllToAll => self.all_to_all_connect(one, two),
            ConnectTypes::AllToAllSelf => self.all_to_all_self_connect(one, two),
        }
    }

    fn one_to_one_connect<C: AsRef<[GraphNode<T>]>>(&mut self, one: &'a C, two: &'a C) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("OneToOne - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: &one.id,
                target_id: &two.id,
            });
        }
    }

    fn one_to_many_connect<C: AsRef<[GraphNode<T>]>>(&mut self, one: &'a C, two: &'a C) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if two_inputs.len() % one_outputs.len() != 0 {
            panic!("OneToMany - TwoGroup inputs must be a multiple of OneGroup outputs.");
        }

        for targets in two_inputs.chunks(one_outputs.len()) {
            for (source, target) in one_outputs.iter().zip(targets.iter()) {
                self.relationships.push(Relationship {
                    source_id: &source.id,
                    target_id: &target.id,
                });
            }
        }
    }

    fn many_to_one_connect<C: AsRef<[GraphNode<T>]>>(&mut self, one: &'a C, two: &'a C) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() % two_inputs.len() != 0 {
            panic!("ManyToOne - OneGroup outputs must be a multiple of TwoGroup inputs.");
        }

        for sources in one_outputs.chunks(two_inputs.len()) {
            for (source, target) in sources.iter().zip(two_inputs.iter()) {
                self.relationships.push(Relationship {
                    source_id: &source.id,
                    target_id: &target.id,
                });
            }
        }
    }

    fn all_to_all_connect<C: AsRef<[GraphNode<T>]>>(&mut self, one: &'a C, two: &'a C) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        for source in one_outputs {
            for target in two_inputs.iter() {
                self.relationships.push(Relationship {
                    source_id: &source.id,
                    target_id: &target.id,
                });
            }
        }
    }

    fn all_to_all_self_connect<C: AsRef<[GraphNode<T>]>>(&mut self, one: &'a C, two: &'a C) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("Self - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: &one.id,
                target_id: &two.id,
            });
            self.relationships.push(Relationship {
                source_id: &two.id,
                target_id: &one.id,
            });
        }
    }

    fn get_outputs<C: AsRef<[GraphNode<T>]>>(&self, collection: &'a C) -> Vec<&'a GraphNode<T>> {
        let outputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .skip_while(|(_, node)| !node.outgoing().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>();

        if !outputs.is_empty() {
            return outputs;
        }

        let recurrent_outputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1
                    && node.is_recurrent()
                    && (node.node_type() == &NodeType::Gate
                        || node.node_type() == &NodeType::Aggregate)
            })
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_outputs.is_empty() {
            return recurrent_outputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>()
    }

    fn get_inputs<C: AsRef<[GraphNode<T>]>>(&self, collection: &'a C) -> Vec<&'a GraphNode<T>> {
        let inputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .take_while(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>();

        if !inputs.is_empty() {
            return inputs;
        }

        let recurrent_inputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1
                    && node.is_recurrent()
                    && node.node_type() == &NodeType::Gate
            })
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_inputs.is_empty() {
            return recurrent_inputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.outgoing().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>()
    }
}
//
// pub struct GraphBuilder<'a, T: Clone + Default> {
//     node_factory: &'a NodeFactory<T>,
// }
//
// impl<'a, T: Clone + Default> GraphBuilder<'a, T> {
//     pub fn new(node_factory: &'a NodeFactory<T>) -> Self {
//         GraphBuilder { node_factory }
//     }
//
//     pub fn build<F>(&self, build_fn: F) -> Graph<T>
//     where
//         F: FnOnce(&GraphBuilder<T>, GraphArchitect<T>) -> Graph<T>,
//     {
//         build_fn(self, GraphArchitect::new(self.node_factory))
//     }
//
//     pub fn input(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Input, size)
//     }
//
//     pub fn output(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Output, size)
//     }
//
//     pub fn gate(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Gate, size)
//     }
//
//     pub fn aggregate(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Aggregate, size)
//     }
//
//     pub fn weight(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Weight, size)
//     }
//
//     pub fn new_collection(&self, node_type: NodeType, size: usize) -> Graph<T> {
//         let nodes = self.new_nodes(node_type, size);
//         Graph::new(nodes)
//     }
//
//     pub fn new_nodes(&self, node_type: NodeType, size: usize) -> Vec<GraphNode<T>> {
//         (0..size)
//             .map(|i| self.node_factory.new_node(i, node_type))
//             .collect::<Vec<GraphNode<T>>>()
//     }
// }
//
// impl<T> GraphBuilder<'_, T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub fn acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             builder
//                 .all_to_all(&arc.input(input_size), &arc.output(output_size))
//                 .build()
//         })
//     }
//
//     pub fn cyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let aggregate = arc.aggregate(input_size);
//             let link = arc.gate(input_size);
//             let output = arc.output(output_size);
//
//             builder
//                 .one_to_one(&input, &aggregate)
//                 .one_to_one_self(&aggregate, &link)
//                 .all_to_all(&aggregate, &output)
//                 .build()
//         })
//     }
//
//     pub fn weighted_acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//             let weights = arc.weight(input_size * output_size);
//
//             builder
//                 .one_to_many(&input, &weights)
//                 .many_to_one(&weights, &output)
//                 .build()
//         })
//     }
//
//     pub fn weighted_cyclic(
//         &self,
//         input_size: usize,
//         output_size: usize,
//         memory_size: usize,
//     ) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//             let weights = arc.weight(input_size * memory_size);
//             let aggregate = arc.aggregate(memory_size);
//             let aggregate_weights = arc.weight(memory_size);
//
//             builder
//                 .one_to_many(&input, &weights)
//                 .many_to_one(&weights, &aggregate)
//                 .one_to_one_self(&aggregate, &aggregate_weights)
//                 .all_to_all(&aggregate, &output)
//                 .build()
//         })
//     }
//
//     pub fn attention_unit(
//         &self,
//         input_size: usize,
//         output_size: usize,
//         num_heads: usize,
//     ) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//
//             let query_weights = arc.weight(input_size * num_heads);
//             let key_weights = arc.weight(input_size * num_heads);
//             let value_weights = arc.weight(input_size * num_heads);
//
//             let attention_scores = arc.new_collection(NodeType::Aggregate, num_heads);
//             let attention_aggreg = arc.new_collection(NodeType::Aggregate, num_heads);
//
//             builder
//                 .one_to_many(&input, &query_weights)
//                 .one_to_many(&input, &key_weights)
//                 .one_to_many(&input, &value_weights)
//                 .many_to_one(&query_weights, &attention_scores)
//                 .many_to_one(&key_weights, &attention_scores)
//                 .one_to_many(&attention_scores, &attention_aggreg)
//                 .many_to_one(&value_weights, &attention_aggreg)
//                 .many_to_one(&attention_aggreg, &output)
//                 .build()
//         })
//     }
//
//     pub fn hopfield(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//             let aggregates = arc.aggregate(input_size);
//             let weights = arc.weight(input_size * output_size);
//
//             builder
//                 .one_to_many(&input, &aggregates)
//                 .one_to_many(&aggregates, &weights)
//                 .many_to_one(&weights, &aggregates)
//                 .many_to_one(&aggregates, &output)
//                 .build()
//         })
//     }
//
//     pub fn lstm(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//
//             let input_to_forget_weights = arc.weight(input_size * memory_size);
//             let hidden_to_forget_weights = arc.weight(memory_size * memory_size);
//
//             let input_to_input_weights = arc.weight(input_size * memory_size);
//             let hidden_to_input_weights = arc.weight(memory_size * memory_size);
//
//             let input_to_candidate_weights = arc.weight(input_size * memory_size);
//             let hidden_to_candidate_weights = arc.weight(memory_size * memory_size);
//
//             let input_to_output_weights = arc.weight(input_size * memory_size);
//             let hidden_to_output_weights = arc.weight(memory_size * memory_size);
//
//             let output_weights = arc.weight(memory_size * output_size);
//
//             let forget_gate = arc.aggregate(memory_size);
//             let input_gate = arc.aggregate(memory_size);
//             let candidate_gate = arc.aggregate(memory_size);
//             let output_gate = arc.aggregate(memory_size);
//
//             let input_candidate_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let forget_memory_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let memory_candidate_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let output_tahn_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let tanh_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//
//             builder
//                 .one_to_many(&input, &input_to_forget_weights)
//                 .one_to_many(&input, &input_to_input_weights)
//                 .one_to_many(&input, &input_to_candidate_weights)
//                 .one_to_many(&input, &input_to_output_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_forget_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_input_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_candidate_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_output_weights)
//                 .many_to_one(&input_to_forget_weights, &forget_gate)
//                 .many_to_one(&hidden_to_forget_weights, &forget_gate)
//                 .many_to_one(&input_to_input_weights, &input_gate)
//                 .many_to_one(&hidden_to_input_weights, &input_gate)
//                 .many_to_one(&input_to_candidate_weights, &candidate_gate)
//                 .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
//                 .many_to_one(&input_to_output_weights, &output_gate)
//                 .many_to_one(&hidden_to_output_weights, &output_gate)
//                 .one_to_one(&forget_gate, &forget_memory_mul_gate)
//                 .one_to_one(&memory_candidate_gate, &forget_memory_mul_gate)
//                 .one_to_one(&input_gate, &input_candidate_mul_gate)
//                 .one_to_one(&candidate_gate, &input_candidate_mul_gate)
//                 .one_to_one(&forget_memory_mul_gate, &memory_candidate_gate)
//                 .one_to_one(&input_candidate_mul_gate, &memory_candidate_gate)
//                 .one_to_one(&memory_candidate_gate, &tanh_gate)
//                 .one_to_one(&tanh_gate, &output_tahn_mul_gate)
//                 .one_to_one(&output_gate, &output_tahn_mul_gate)
//                 .one_to_many(&output_tahn_mul_gate, &output_weights)
//                 .many_to_one(&output_weights, &output)
//                 .build()
//         })
//     }
//
//     pub fn gru(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//
//             let output_weights = arc.weight(memory_size * output_size);
//
//             let reset_gate = arc.aggregate(memory_size);
//             let update_gate = arc.aggregate(memory_size);
//             let candidate_gate = arc.aggregate(memory_size);
//
//             let input_to_reset_weights = arc.weight(input_size * memory_size);
//             let input_to_update_weights = arc.weight(input_size * memory_size);
//             let input_to_candidate_weights = arc.weight(input_size * memory_size);
//
//             let hidden_to_reset_weights = arc.weight(memory_size * memory_size);
//             let hidden_to_update_weights = arc.weight(memory_size * memory_size);
//             let hidden_to_candidate_weights = arc.weight(memory_size * memory_size);
//
//             let hidden_reset_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let update_candidate_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let invert_update_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let hidden_invert_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//             let candidate_hidden_add_gate = arc.new_collection(NodeType::Aggregate, memory_size);
//
//             builder
//                 .one_to_many(&input, &input_to_reset_weights)
//                 .one_to_many(&input, &input_to_update_weights)
//                 .one_to_many(&input, &input_to_candidate_weights)
//                 .one_to_many(&candidate_hidden_add_gate, &hidden_to_reset_weights)
//                 .one_to_many(&candidate_hidden_add_gate, &hidden_to_update_weights)
//                 .many_to_one(&input_to_reset_weights, &reset_gate)
//                 .many_to_one(&hidden_to_reset_weights, &reset_gate)
//                 .many_to_one(&input_to_update_weights, &update_gate)
//                 .many_to_one(&hidden_to_update_weights, &update_gate)
//                 .one_to_one(&reset_gate, &hidden_reset_gate)
//                 .one_to_one(&candidate_hidden_add_gate, &hidden_reset_gate)
//                 .one_to_many(&hidden_reset_gate, &hidden_to_candidate_weights)
//                 .many_to_one(&input_to_candidate_weights, &candidate_gate)
//                 .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
//                 .one_to_one(&update_gate, &update_candidate_mul_gate)
//                 .one_to_one(&candidate_gate, &update_candidate_mul_gate)
//                 .one_to_one(&update_gate, &invert_update_gate)
//                 .one_to_one(&candidate_hidden_add_gate, &hidden_invert_mul_gate)
//                 .one_to_one(&invert_update_gate, &hidden_invert_mul_gate)
//                 .one_to_one(&hidden_invert_mul_gate, &candidate_hidden_add_gate)
//                 .one_to_one(&update_candidate_mul_gate, &candidate_hidden_add_gate)
//                 .one_to_many(&candidate_hidden_add_gate, &output_weights)
//                 .many_to_one(&output_weights, &output)
//                 .build()
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use crate::builder::TreeBuilder;
    use crate::operation;

    #[test]
    fn test_tree_archit() {
        let tree_archit = TreeBuilder::<f32>::new(3)
            .with_gates(vec![operation::add(), operation::sub()])
            .with_leafs(vec![operation::var(0), operation::var(1)]);
        let tree = tree_archit.build();
        let size = tree.root().map(|n| n.size()).unwrap_or(0);

        assert_eq!(size, 15);
    }
}
