
# Nodes 

Nodes are not only the `gene` of the `graph` and `tree`, but the fundamental building blocks of them. Each `node` represents a connection, computation, or operation, and has explicit rules depending on its role in the structure. 

---

## Roles

Nodes in the `gp` system come in different types (roles) depending on whether you're working with trees or graphs:

**Tree Node Types:**

- **Root**: The starting point of a tree (can have any number of children)
- **Vertex**: Internal computation nodes (can have any number of children)
- **Leaf**: Terminal nodes with no children (arity is `Arity::Zero`)

**Graph Node Types:**

- **Input**: Entry points (no incoming connections, one or more outgoing)
- **Output**: Exit points (one or more incoming connections, no outgoing)
- **Vertex**: Internal computation nodes (both incoming and outgoing connections)
- **Edge**: Connection nodes (exactly one incoming and one outgoing connection)

Each node type is defined by the `NodeType` enum:

=== ":fontawesome-brands-python: Python"

    Node types aren't defined in python - we use strings or input variables instead.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    pub enum NodeType {
        Root,    // Tree-specific
        Vertex,  // Both trees and graphs
        Leaf,    // Tree-specific
        Input,   // Graph-specific
        Output,  // Graph-specific
        Edge,    // Graph-specific
    }
    ```

---

## Store

The `NodeStore<T>` manages available values for different node types, providing a centralized way to define what values can be used in each position of your genetic program. This makes it super easy to create `trees` or `graphs` from a specific template or with a specific structure.

**Usage Examples:**

=== ":fontawesome-brands-python: Python"

    There is no node store for python - it isn't nessesary for the api. Instead, the types of nodes are directly given the their codec or structure.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;
    
    // Create a store for tree operations
    // Each vertex node created will have a random value chosen from [1, 2, 3]
    // Each leaf node created will have a random value chosen from [4, 5, 6]
    let tree_store: NodeStore<i32> = vec![
        (NodeType::Vertex, vec![1, 2, 3]),
        (NodeType::Leaf, vec![4, 5, 6]),
    ].into();

    // -- or use the macro --

    let tree_store: NodeStore<i32> = node_store! {
        Root => [1, 2, 3],
        Vertex => [1, 2, 3],
        Leaf => [4, 5, 6],
    }

    // -- with ops --
    // for trees, the input nodes are always the leaf nodes, so we can use the `Op::var` to represent them
    let op_store: NodeStore<Op<f32>> = node_store! {
        Root => [Op::sigmoid()],
        Vertex => [Op::add(), Op::mul()],
        Leaf => (0..3).map(Op::var).collect::<Vec<_>>(),
    };

    // Create a new vertex tree node 
    let tree_node: TreeNode<i32> = tree_store.new_instance(NodeType::Vertex);

    // Create a new leaf tree node
    let leaf_node: TreeNode<Op<f32>> = op_store.new_instance(NodeType::Leaf);
    
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
    ].into();

    // -- or use the macro --

    let graph_store: NodeStore<i32> = node_store! {
        Input => [1, 2],
        Edge => [3, 4],
        Vertex => [5, 6, 7],
        Output => [8, 9, 10],
    };

    // -- with ops --
    let op_store: NodeStore<Op<f32>> = node_store! {
        Input => [Op::var(0), Op::var(1)],
        Edge => [Op::add(), Op::mul()],
        Vertex => [Op::sub(), Op::div(), Op::max()],
        Output => [Op::sigmoid(), Op::tanh(), Op::relu()],
    };

    // Create a new vertex graph node at index 0
    let graph_node: GraphNode<i32> = graph_store.new_instance((0, NodeType::Vertex));

    // Createa a new edge graph node at index 1
    let edge_node: GraphNode<Op<f32>> = op_store.new_instance((1, NodeType::Edge));
    ```

**Node Type Mapping:**

- **Tree**: `Root`, `Vertex`, `Leaf`
- **Graph**: `Input`, `Output`, `Vertex`, `Edge`

**Store Validation:**
The node store ensures that:

- Each node type has appropriate value
- Values have compatible arity for their node type
- Invalid combinations are prevented during evolution
