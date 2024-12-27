use crate::NodeType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub type NodeValueMap<T> = Arc<RefCell<HashMap<NodeType, Vec<T>>>>;

#[derive(Clone, PartialEq)]
pub struct NodeSchema<T> {
    pub id: Uuid,
    pub name: Option<String>,
    pub node_type: Option<NodeType>,
    pub max_incoming: Option<usize>,
    pub max_outgoing: Option<usize>,
    pub min_incoming: Option<usize>,
    pub min_outgoing: Option<usize>,
    pub values: Option<NodeValueMap<T>>,
}

impl<T> NodeSchema<T> {
    pub fn new(id: Uuid, node_type: NodeType) -> Self {
        Self {
            id,
            name: None,
            node_type: Some(node_type),
            max_incoming: None,
            max_outgoing: None,
            min_incoming: None,
            min_outgoing: None,
            values: None,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type.unwrap()
    }

    pub fn max_incoming(&self) -> Option<usize> {
        self.max_incoming
    }

    pub fn max_outgoing(&self) -> Option<usize> {
        self.max_outgoing
    }

    pub fn min_incoming(&self) -> Option<usize> {
        self.min_incoming
    }

    pub fn min_outgoing(&self) -> Option<usize> {
        self.min_outgoing
    }

    pub fn values(&self) -> Option<&NodeValueMap<T>> {
        self.values.as_ref()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn set_node_type(&mut self, node_type: NodeType) {
        self.node_type = Some(node_type);
    }

    pub fn set_max_incoming(&mut self, max_incoming: usize) {
        self.max_incoming = Some(max_incoming);
    }

    pub fn set_max_outgoing(&mut self, max_outgoing: usize) {
        self.max_outgoing = Some(max_outgoing);
    }

    pub fn set_min_incoming(&mut self, min_incoming: usize) {
        self.min_incoming = Some(min_incoming);
    }

    pub fn set_min_outgoing(&mut self, min_outgoing: usize) {
        self.min_outgoing = Some(min_outgoing);
    }

    pub fn set_values(&mut self, values: NodeValueMap<T>) {
        self.values = Some(values);
    }
}

impl<T> Default for NodeSchema<T> {
    fn default() -> Self {
        Self::new(Uuid::new_v4(), NodeType::Unknown)
    }
}
