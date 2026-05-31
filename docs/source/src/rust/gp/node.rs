use radiate::prelude::*;

fn main() {
    // --8<-- [start:store]
    // Create a store for tree operations
    // Each vertex node created will have a random value chosen from [1, 2, 3]
    // Each leaf node created will have a random value chosen from [4, 5, 6]
    let tree_store: NodeStore<i32> = vec![
        (NodeType::Vertex, vec![1, 2, 3]),
        (NodeType::Leaf, vec![4, 5, 6]),
    ]
    .into();

    // -- or use the macro --

    let tree_store: NodeStore<i32> = node_store! {
        Root => vec![1, 2, 3],
        Vertex => vec![1, 2, 3],
        Leaf => vec![4, 5, 6]
    };

    // -- with ops --
    // for trees, the input nodes are always the leaf nodes, so we can use the `Op::var` to represent them
    let op_store: NodeStore<Op<f32>> = node_store! {
        Root => vec![Op::sigmoid()],
        Vertex => vec![Op::add(), Op::mul()],
        Leaf => (0..3).map(Op::var).collect::<Vec<_>>()
    };

    // Create a new vertex tree node
    let tree_node: Option<TreeNode<i32>> = tree_store.new_instance(NodeType::Vertex);

    // Create a new leaf tree node
    let leaf_node: Option<TreeNode<Op<f32>>> = op_store.new_instance(NodeType::Leaf);

    // Create a store for graph operations
    // Each input node created will have a random value chosen from [1, 2]
    // Each edge node created will have a random value chosen from [3, 4]
    // Each vertex node created will have a random value chosen from [5, 6, 7]
    // Each output node created will have a random value chosen from [8, 9, 10]
    let graph_store: NodeStore<i32> = vec![
        (NodeType::Input, vec![1, 2]),
        (NodeType::Edge, vec![3, 4]),
        (NodeType::Vertex, vec![5, 6, 7]),
        (NodeType::Output, vec![8, 9, 10]),
    ]
    .into();

    // -- or use the macro --

    let graph_store: NodeStore<i32> = node_store! {
        Input => vec![1, 2],
        Edge => vec![3, 4],
        Vertex => vec![5, 6, 7],
        Output => vec![8, 9, 10]
    };

    // -- with ops --
    let op_store: NodeStore<Op<f32>> = node_store! {
        Input => vec![Op::var(0), Op::var(1)],
        Edge => vec![Op::add(), Op::mul()],
        Vertex => vec![Op::sub(), Op::div(), Op::max()],
        Output => vec![Op::sigmoid(), Op::tanh(), Op::relu()]
    };

    // Create a new vertex graph node at index 0
    let graph_node: Option<GraphNode<i32>> = graph_store.new_instance((0, NodeType::Vertex));

    // Create a new edge graph node at index 1
    let edge_node: Option<GraphNode<Op<f32>>> = op_store.new_instance((1, NodeType::Edge));
    // --8<-- [end:store]
}
