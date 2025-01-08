use crate::{
    collections::{Builder, Graph, GraphNode, NodeType},
    NodeCell,
};
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
pub struct GraphArchitect<'a, C: NodeCell + Clone> {
    nodes: BTreeMap<&'a Uuid, &'a GraphNode<C>>,
    node_order: BTreeMap<usize, &'a Uuid>,
    relationships: Vec<Relationship<'a>>,
}

impl<'a, C: NodeCell + Clone> GraphArchitect<'a, C> {
    pub fn new() -> Self {
        GraphArchitect {
            nodes: BTreeMap::new(),
            node_order: BTreeMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn one_to_one<G: AsRef<[GraphNode<C>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToOne, one, two);
        self
    }

    pub fn one_to_many<G: AsRef<[GraphNode<C>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToMany, one, two);
        self
    }

    pub fn many_to_one<G: AsRef<[GraphNode<C>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::ManyToOne, one, two);
        self
    }

    pub fn all_to_all<G: AsRef<[GraphNode<C>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::AllToAll, one, two);
        self
    }

    pub fn one_to_one_self<G: AsRef<[GraphNode<C>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::AllToAllSelf, one, two);
        self
    }

    pub fn insert<G: AsRef<[GraphNode<C>]>>(mut self, collection: &'a G) -> Self {
        self.attach(collection.as_ref());
        self
    }
}

impl<'a, C: NodeCell + Clone> GraphArchitect<'a, C> {
    pub fn layer<G: AsRef<[GraphNode<C>]>>(&self, collections: Vec<&'a G>) -> Self {
        let mut conn = GraphArchitect::new();
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

    pub fn attach(&mut self, group: &'a [GraphNode<C>]) {
        for node in group.iter() {
            if !self.nodes.contains_key(&node.id()) {
                let node_id = &node.id();

                self.nodes.insert(node_id, node);
                self.node_order.insert(self.node_order.len(), node_id);

                for outgoing in group
                    .iter()
                    .filter(|item| node.outgoing().contains(&item.index()))
                {
                    self.relationships.push(Relationship {
                        source_id: &node.id(),
                        target_id: &outgoing.id(),
                    });
                }
            }
        }
    }
}

impl<'a, C: NodeCell + Clone> GraphArchitect<'a, C> {
    fn connect<G: AsRef<[GraphNode<C>]>>(
        &mut self,
        connection: ConnectTypes,
        one: &'a G,
        two: &'a G,
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

    fn one_to_one_connect<G: AsRef<[GraphNode<C>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("OneToOne - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: &one.id(),
                target_id: &two.id(),
            });
        }
    }

    fn one_to_many_connect<G: AsRef<[GraphNode<C>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if two_inputs.len() % one_outputs.len() != 0 {
            panic!("OneToMany - TwoGroup inputs must be a multiple of OneGroup outputs.");
        }

        for targets in two_inputs.chunks(one_outputs.len()) {
            for (source, target) in one_outputs.iter().zip(targets.iter()) {
                self.relationships.push(Relationship {
                    source_id: &source.id(),
                    target_id: &target.id(),
                });
            }
        }
    }

    fn many_to_one_connect<G: AsRef<[GraphNode<C>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() % two_inputs.len() != 0 {
            panic!("ManyToOne - OneGroup outputs must be a multiple of TwoGroup inputs.");
        }

        for sources in one_outputs.chunks(two_inputs.len()) {
            for (source, target) in sources.iter().zip(two_inputs.iter()) {
                self.relationships.push(Relationship {
                    source_id: &source.id(),
                    target_id: &target.id(),
                });
            }
        }
    }

    fn all_to_all_connect<G: AsRef<[GraphNode<C>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        for source in one_outputs {
            for target in two_inputs.iter() {
                self.relationships.push(Relationship {
                    source_id: &source.id(),
                    target_id: &target.id(),
                });
            }
        }
    }

    fn all_to_all_self_connect<G: AsRef<[GraphNode<C>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("Self - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: &one.id(),
                target_id: &two.id(),
            });
            self.relationships.push(Relationship {
                source_id: &two.id(),
                target_id: &one.id(),
            });
        }
    }

    fn get_outputs<G: AsRef<[GraphNode<C>]>>(&self, collection: &'a G) -> Vec<&'a GraphNode<C>> {
        let outputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .skip_while(|(_, node)| !node.outgoing().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<C>>>();

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
                    && (node.node_type() == NodeType::Vertex)
            })
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<C>>>();

        if !recurrent_outputs.is_empty() {
            return recurrent_outputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<C>>>()
    }

    fn get_inputs<G: AsRef<[GraphNode<C>]>>(&self, collection: &'a G) -> Vec<&'a GraphNode<C>> {
        let inputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .take_while(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<C>>>();

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
                    && node.node_type() == NodeType::Vertex
            })
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<C>>>();

        if !recurrent_inputs.is_empty() {
            return recurrent_inputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.outgoing().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<C>>>()
    }
}

impl<C: NodeCell + Clone> Builder for GraphArchitect<'_, C> {
    type Output = Graph<C>;

    fn build(&self) -> Self::Output {
        let mut new_nodes = Vec::new();
        let mut node_id_index_map = BTreeMap::new();

        for (index, (_, node_id)) in self.node_order.iter().enumerate() {
            let node = self.nodes.get(node_id).unwrap();
            let new_node = GraphNode::new(index, node.node_type(), node.value().clone());

            new_nodes.push(new_node);
            node_id_index_map.insert(node_id, index);
        }

        let mut new_collection = Graph::new(new_nodes);
        for rel in self.relationships.iter() {
            let source_idx = node_id_index_map.get(&rel.source_id).unwrap();
            let target_idx = node_id_index_map.get(&rel.target_id).unwrap();

            new_collection.attach(*source_idx, *target_idx);
        }

        new_collection.set_cycles(vec![]);
        new_collection
    }
}

// pub fn build(self) -> Graph<T>
// where
//     T: Default,
// {
//     let mut new_nodes = Vec::new();
//     let mut node_id_index_map = BTreeMap::new();

//     for (index, (_, node_id)) in self.node_order.iter().enumerate() {
//         let node = self.nodes.get(node_id).unwrap();
//         let new_node = GraphNode::new(index, node.node_type, node.value.clone());

//         new_nodes.push(new_node);
//         node_id_index_map.insert(node_id, index);
//     }

//     let mut new_collection = Graph::new(new_nodes);
//     for rel in self.relationships {
//         let source_idx = node_id_index_map.get(&rel.source_id).unwrap();
//         let target_idx = node_id_index_map.get(&rel.target_id).unwrap();

//         new_collection.attach(*source_idx, *target_idx);
//     }

//     let mut collection = new_collection.clone().set_cycles(Vec::new());

//     for node in collection.as_mut() {
//         if let Some(factory) = self.factory {
//             let temp_node = factory.new_node(node.index, NodeType::Vertex);

//             match node.node_type() {
//                 NodeType::Input => {
//                     if !node.incoming().is_empty() {
//                         node.node_type = NodeType::Vertex;
//                         node.value = temp_node.value.clone();
//                     }
//                 }
//                 NodeType::Output => {
//                     if !node.outgoing().is_empty() {
//                         node.node_type = NodeType::Vertex;
//                         node.value = temp_node.value.clone();
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }

//     collection
// }
