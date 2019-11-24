
/// In a neural network a node can be either an input, hidden
/// or output node. These mark the neruons as such
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Layer {
    Input,
    Output,
    Hidden,
}
