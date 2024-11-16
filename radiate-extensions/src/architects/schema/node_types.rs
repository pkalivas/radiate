#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Input,
    Output,
    Gate,
    Aggregate,
    Weight,
    Link,
}
