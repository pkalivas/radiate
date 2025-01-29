#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Input,
    Output,
    Vertex,
    Edge,
    Leaf,
    Root,
}
