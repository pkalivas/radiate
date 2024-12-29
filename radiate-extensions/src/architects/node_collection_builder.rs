use std::collections::BTreeMap;

use super::{expr, Graph, GraphNode};
use crate::{NodeCell, NodeFactory, Role};
use radiate::random_provider;
use uuid::Uuid;

pub enum ConnectTypes {
    OneToOne,
    OneToMany,
    ManyToOne,
    AllToAll,
    AllToAllSelf,
}

pub struct Relationship<'a> {
    pub source_id: &'a Uuid,
    pub target_id: &'a Uuid,
}

pub struct GraphBuilder<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub factory: &'a NodeFactory<T>,
    pub nodes: BTreeMap<&'a Uuid, &'a GraphNode<T>>,
    pub node_order: BTreeMap<usize, &'a Uuid>,
    pub relationships: Vec<Relationship<'a>>,
}

impl<'a, T> GraphBuilder<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(factory: &'a NodeFactory<T>) -> Self {
        GraphBuilder {
            factory,
            nodes: BTreeMap::new(),
            node_order: BTreeMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        let builder = GraphBuilder::new(self.factory);
        let input = self.new_graph(input_size, Role::Provider);
        let output = self.new_graph(output_size, Role::Output);

        builder.all_to_all(&input, &output).build()
    }

    pub fn one_to_one(mut self, one: &'a Graph<T>, two: &'a Graph<T>) -> Self {
        self.connect(ConnectTypes::OneToOne, one, two);
        self
    }

    pub fn one_to_many(mut self, one: &'a Graph<T>, two: &'a Graph<T>) -> Self {
        self.connect(ConnectTypes::OneToMany, one, two);
        self
    }

    pub fn many_to_one(mut self, one: &'a Graph<T>, two: &'a Graph<T>) -> Self {
        self.connect(ConnectTypes::ManyToOne, one, two);
        self
    }

    pub fn all_to_all(mut self, one: &'a Graph<T>, two: &'a Graph<T>) -> Self {
        self.connect(ConnectTypes::AllToAll, one, two);
        self
    }

    pub fn one_to_one_self(mut self, one: &'a Graph<T>, two: &'a Graph<T>) -> Self {
        self.connect(ConnectTypes::AllToAllSelf, one, two);
        self
    }

    pub fn insert(mut self, collection: &'a Graph<T>) -> Self {
        self.attach(collection.nodes());
        self
    }

    pub fn build(self) -> Graph<T> {
        let mut index = 0;
        let mut new_nodes = Vec::new();
        let mut node_id_index_map = BTreeMap::new();

        for (_, node_id) in self.node_order.iter() {
            let node = self.nodes.get(node_id).unwrap();
            let new_node = GraphNode::new(index, node.cell.clone());

            new_nodes.push(new_node);
            node_id_index_map.insert(node_id, index);

            index += 1;
        }

        let mut new_collection = Graph::new(new_nodes);
        for rel in self.relationships {
            let source_idx = node_id_index_map.get(&rel.source_id).unwrap();
            let target_idx = node_id_index_map.get(&rel.target_id).unwrap();

            new_collection.attach(*source_idx, *target_idx);
        }

        new_collection
    }

    pub fn layer(&self, collections: Vec<&'a Graph<T>>) -> Self {
        let mut conn = GraphBuilder::new(&self.factory);
        let mut previous = collections[0];

        for collection in collections.iter() {
            conn.attach((*collection).nodes());
        }

        for i in 1..collections.len() {
            conn = conn.one_to_one(previous, collections[i]);
            previous = collections[i];
        }

        conn
    }

    pub fn connect(&mut self, connection: ConnectTypes, one: &'a Graph<T>, two: &'a Graph<T>) {
        self.attach(one.nodes());
        self.attach(two.nodes());

        match connection {
            ConnectTypes::OneToOne => self.one_to_one_connect(one, two),
            ConnectTypes::OneToMany => self.one_to_many_connect(one, two),
            ConnectTypes::ManyToOne => self.many_to_one_connect(one, two),
            ConnectTypes::AllToAll => self.all_to_all_connect(one, two),
            ConnectTypes::AllToAllSelf => self.all_to_all_self_connect(one, two),
        }
    }

    pub fn attach(&mut self, group: &'a [GraphNode<T>]) {
        for node in group.iter() {
            if !self.nodes.contains_key(&node.cell.id) {
                let node_id = &node.as_ref().id;

                self.nodes.insert(node_id, node);
                self.node_order.insert(self.node_order.len(), node_id);

                for outgoing in group
                    .iter()
                    .filter(|item| node.outgoing().contains(&item.index))
                {
                    self.relationships.push(Relationship {
                        source_id: &node.cell.id,
                        target_id: &outgoing.cell.id,
                    });
                }
            }
        }
    }

    fn new_graph(&self, count: usize, role: Role) -> Graph<T> {
        let mut nodes = Vec::with_capacity(count);

        for i in 0..count {
            let new_node = match role {
                Role::Provider => GraphNode::new(i, NodeCell::provider(expr::var(i))),
                Role::Output => {
                    let val = random_provider::choose(self.factory.get_outputs());
                    GraphNode::new(i, NodeCell::output(val.clone()))
                }
                Role::Internal => {
                    let val = random_provider::choose(self.factory.get_operations());
                    GraphNode::new(i, NodeCell::internal(val.clone()))
                }
            };

            nodes.push(new_node);
        }

        Graph::new(nodes)
    }

    fn one_to_one_connect(&mut self, one: &'a Graph<T>, two: &'a Graph<T>) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("OneToOne - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: &one.cell.id,
                target_id: &two.cell.id,
            });
        }
    }

    fn one_to_many_connect(&mut self, one: &'a Graph<T>, two: &'a Graph<T>) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if two_inputs.len() % one_outputs.len() != 0 {
            panic!("OneToMany - TwoGroup inputs must be a multiple of OneGroup outputs.");
        }

        for targets in two_inputs.chunks(one_outputs.len()) {
            for (source, target) in one_outputs.iter().zip(targets.iter()) {
                self.relationships.push(Relationship {
                    source_id: &source.cell.id,
                    target_id: &target.cell.id,
                });
            }
        }
    }

    fn many_to_one_connect(&mut self, one: &'a Graph<T>, two: &'a Graph<T>) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() % two_inputs.len() != 0 {
            panic!("ManyToOne - OneGroup outputs must be a multiple of TwoGroup inputs.");
        }

        for sources in one_outputs.chunks(two_inputs.len()) {
            for (source, target) in sources.iter().zip(two_inputs.iter()) {
                self.relationships.push(Relationship {
                    source_id: &source.cell.id,
                    target_id: &target.cell.id,
                });
            }
        }
    }

    fn all_to_all_connect(&mut self, one: &'a Graph<T>, two: &'a Graph<T>) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        for source in one_outputs {
            for target in two_inputs.iter() {
                self.relationships.push(Relationship {
                    source_id: &source.cell.id,
                    target_id: &target.cell.id,
                });
            }
        }
    }

    fn all_to_all_self_connect(&mut self, one: &'a Graph<T>, two: &'a Graph<T>) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("Self - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: &one.cell.id,
                target_id: &two.cell.id,
            });
            self.relationships.push(Relationship {
                source_id: &two.cell.id,
                target_id: &one.cell.id,
            });
        }
    }

    fn get_outputs(&self, collection: &'a Graph<T>) -> Vec<&'a GraphNode<T>> {
        let outputs = collection
            .nodes()
            .iter()
            .enumerate()
            .skip_while(|(_, node)| !node.outgoing().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !outputs.is_empty() {
            return outputs;
        }

        let recurrent_outputs = collection
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1 && node.is_recurrent() && (node.is_internal())
            })
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_outputs.is_empty() {
            return recurrent_outputs;
        }

        collection
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&GraphNode<T>>>()
    }

    fn get_inputs(&self, collection: &'a Graph<T>) -> Vec<&'a GraphNode<T>> {
        let inputs = collection
            .nodes()
            .iter()
            .enumerate()
            .take_while(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !inputs.is_empty() {
            return inputs;
        }

        let recurrent_inputs = collection
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1 && node.is_recurrent() && node.is_internal()
            })
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_inputs.is_empty() {
            return recurrent_inputs;
        }

        collection
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.outgoing().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&GraphNode<T>>>()
    }
}
