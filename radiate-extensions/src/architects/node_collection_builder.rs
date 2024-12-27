use std::collections::{BTreeMap, HashSet};

use crate::architects::node_collections::node::Node;
use crate::architects::node_collections::node_factory::NodeFactory;
use crate::architects::node_collections::NodeCollection;
use crate::architects::schema::node_types::NodeType;

use uuid::Uuid;

use super::NodeRepairs;

pub enum ConnectTypes {
    OneToOne,
    OneToMany,
    ManyToOne,
    AllToAll,
    AllToAllSelf,
    ParentToChild,
    Replace,
}

pub struct Relationship<'a> {
    pub source_id: &'a Uuid,
    pub target_id: &'a Uuid,
}

#[derive(Default)]
pub struct NodeCollectionBuilder<'a, C, T>
where
    C: NodeCollection<T> + NodeRepairs<T>,
    T: Clone + PartialEq + Default,
{
    pub factory: Option<&'a NodeFactory<T>>,
    pub nodes: BTreeMap<&'a Uuid, &'a Node<T>>,
    pub node_order: BTreeMap<usize, &'a Uuid>,
    pub relationships: Vec<Relationship<'a>>,
    pub removed: HashSet<&'a Uuid>,
    _phantom_c: std::marker::PhantomData<C>,
}

impl<'a, C, T> NodeCollectionBuilder<'a, C, T>
where
    C: NodeCollection<T> + NodeRepairs<T>,
    T: Clone + PartialEq + Default,
{
    pub fn new(factory: &'a NodeFactory<T>) -> Self {
        NodeCollectionBuilder {
            factory: Some(factory),
            nodes: BTreeMap::new(),
            node_order: BTreeMap::new(),
            relationships: Vec::new(),
            removed: HashSet::new(),
            _phantom_c: std::marker::PhantomData,
        }
    }

    pub fn one_to_one(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::OneToOne, one, two);
        self
    }

    pub fn one_to_many(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::OneToMany, one, two);
        self
    }

    pub fn many_to_one(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::ManyToOne, one, two);
        self
    }

    pub fn all_to_all(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::AllToAll, one, two);
        self
    }

    pub fn one_to_one_self(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::AllToAllSelf, one, two);
        self
    }

    pub fn parent_to_child(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::ParentToChild, one, two);
        self
    }

    pub fn replace(mut self, one: &'a C, two: &'a C) -> Self {
        self.connect(ConnectTypes::Replace, one, two);
        self
    }

    pub fn insert(mut self, collection: &'a C) -> Self {
        self.attach(collection.get_nodes());
        self
    }

    pub fn build(self) -> C {
        let mut index = 0;
        let mut new_nodes = Vec::new();
        let mut node_id_index_map = BTreeMap::new();

        for (_, node_id) in self.node_order.iter() {
            if self.removed.contains(node_id) {
                continue;
            }

            let node = self.nodes.get(node_id).unwrap();
            let new_node = Node::new(index, node.node_type, node.value.clone());

            new_nodes.push(new_node);
            node_id_index_map.insert(node_id, index);

            index += 1;
        }

        let mut new_collection = C::from_nodes(new_nodes);
        for rel in self.relationships {
            if self.removed.contains(rel.source_id) || self.removed.contains(rel.target_id) {
                continue;
            }

            let source_idx = node_id_index_map.get(&rel.source_id).unwrap();
            let target_idx = node_id_index_map.get(&rel.target_id).unwrap();

            new_collection.attach(*source_idx, *target_idx);
        }

        new_collection.repair(self.factory)
    }

    pub fn layer(&self, collections: Vec<&'a C>) -> Self {
        let mut conn = NodeCollectionBuilder::new(self.factory.unwrap());
        let mut previous = collections[0];

        for collection in collections.iter() {
            conn.attach((*collection).get_nodes());
        }

        for i in 1..collections.len() {
            conn = conn.one_to_one(previous, collections[i]);
            previous = collections[i];
        }

        conn
    }

    pub fn connect(&mut self, connection: ConnectTypes, one: &'a C, two: &'a C) {
        self.attach(one.get_nodes());
        self.attach(two.get_nodes());

        match connection {
            ConnectTypes::OneToOne => self.one_to_one_connect(one, two),
            ConnectTypes::OneToMany => self.one_to_many_connect(one, two),
            ConnectTypes::ManyToOne => self.many_to_one_connect(one, two),
            ConnectTypes::AllToAll => self.all_to_all_connect(one, two),
            ConnectTypes::AllToAllSelf => self.all_to_all_self_connect(one, two),
            ConnectTypes::ParentToChild => self.parent_to_child_connect(one, two),
            ConnectTypes::Replace => self.replace_connect(one, two),
        }
    }

    pub fn attach(&mut self, group: &'a [Node<T>]) {
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

    fn replace_connect(&mut self, one: &'a C, two: &'a C) {
        let two_inputs = self.get_inputs(two);
        let one_inputs = self.get_inputs(one);

        for node in one.iter() {
            self.removed.insert(&node.id);
        }

        let source_to_removed = self
            .relationships
            .iter()
            .filter(|rel| one_inputs.iter().any(|node| node.id == *rel.target_id))
            .map(|rel| (rel.source_id, rel.target_id))
            .collect::<Vec<(&Uuid, &Uuid)>>();

        if source_to_removed.len() != two_inputs.len() {
            panic!("Replace - OneGroup outputs must be the same length as TwoGroup inputs.");
        }

        for (source, target) in source_to_removed.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: source.0,
                target_id: &target.id,
            });
        }
    }

    fn one_to_one_connect(&mut self, one: &'a C, two: &'a C) {
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

    fn one_to_many_connect(&mut self, one: &'a C, two: &'a C) {
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

    fn many_to_one_connect(&mut self, one: &'a C, two: &'a C) {
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

    fn all_to_all_connect(&mut self, one: &'a C, two: &'a C) {
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

    fn all_to_all_self_connect(&mut self, one: &'a C, two: &'a C) {
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

    fn parent_to_child_connect(&mut self, one: &'a C, two: &'a C) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != 1 {
            panic!("ParentToChild - oneGroup outputs must be a single node.");
        }

        let parent_node = one_outputs[0];
        for child_node in two_inputs {
            self.relationships.push(Relationship {
                source_id: &parent_node.id,
                target_id: &child_node.id,
            });
        }
    }

    fn get_outputs(&self, collection: &'a C) -> Vec<&'a Node<T>> {
        let outputs = collection
            .iter()
            .enumerate()
            .skip_while(|(_, node)| !node.outgoing().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&Node<T>>>();

        if !outputs.is_empty() {
            return outputs;
        }

        let recurrent_outputs = collection
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1
                    && node.is_recurrent()
                    && (node.node_type() == &NodeType::Gate
                        || node.node_type() == &NodeType::Aggregate)
            })
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&Node<T>>>();

        if !recurrent_outputs.is_empty() {
            return recurrent_outputs;
        }

        collection
            .iter()
            .enumerate()
            .filter(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&Node<T>>>()
    }

    fn get_inputs(&self, collection: &'a C) -> Vec<&'a Node<T>> {
        let inputs = collection
            .iter()
            .enumerate()
            .take_while(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&Node<T>>>();

        if !inputs.is_empty() {
            return inputs;
        }

        let recurrent_inputs = collection
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1
                    && node.is_recurrent()
                    && node.node_type() == &NodeType::Gate
            })
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&Node<T>>>();

        if !recurrent_inputs.is_empty() {
            return recurrent_inputs;
        }

        collection
            .iter()
            .enumerate()
            .filter(|(_, node)| node.outgoing().is_empty())
            .map(|(idx, _)| collection.get(idx))
            .collect::<Vec<&Node<T>>>()
    }
}
