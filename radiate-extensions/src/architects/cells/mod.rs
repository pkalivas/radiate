pub mod expr;
pub mod graph_node;
pub mod tree_node;

use expr::Expr;
pub use graph_node::*;
pub use tree_node::*;

use super::NodeType;

#[derive(Clone, PartialEq)]
pub struct NodeCell<T> {
    pub value: Expr<T>,
    pub id: uuid::Uuid,
    pub node_type: NodeType,
}

impl<T> NodeCell<T> {
    pub fn new(value: Expr<T>, node_type: NodeType) -> Self {
        NodeCell {
            value,
            id: uuid::Uuid::new_v4(),
            node_type,
        }
    }
}
