
# Nodes 

Nodes are not only the `gene` of the `graph` and `tree`, but the fundamental building blocks of them. Each `node` represents a connection, computation, or operation, and has explicit rules depending on its role in the structure. 

---

## Roles

Nodes in the `gp` system come in different types (roles) depending on whether you're working with trees or graphs:

**Tree Node Types:**

- **Root**: The starting point of a tree (number of children is determined by its op's arity)
- **Vertex**: Internal computation nodes (number of children is determined by the op's arity)
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
        Input,   // Graph-specific
        Output,  // Graph-specific
        Vertex,  // Both trees and graphs
        Edge,    // Graph-specific
        Leaf,    // Tree-specific
        Root,    // Tree-specific
    }
    ```

---

## Store

The `NodeStore<T>` manages available values for different node types, providing a centralized way to define what values can be used in each position of your genetic program. This makes it super easy to create `trees` or `graphs` from a specific template or with a specific structure.

**Usage Examples:**

=== ":fontawesome-brands-python: Python"

    There is no node store for python - it isn't necessary for the api. Instead, the types of nodes are directly given by their codec or structure.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/gp/node.rs:store"
    ```

**Node Type Mapping:**

- **Tree**: `Root`, `Vertex`, `Leaf`
- **Graph**: `Input`, `Output`, `Vertex`, `Edge`

**Store Validation:**
The node store ensures that:

- Each node type has appropriate values
- Values have compatible arity for their node type
- Invalid combinations are prevented during evolution
