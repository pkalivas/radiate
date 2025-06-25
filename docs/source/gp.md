# Genetic Programming

!!! warning ":construction: Under Construction :construction:"

    As of `6/22/2025`: These docs are a work in progress and may not be complete or fully accurate. Please check back later for updates.

___

Genetic Programming (GP) in Radiate enables the evolution of programs represented as **expression trees** and **computational graphs**. This powerful feature allows you to solve complex problems by evolving mathematical expressions, decision trees, neural network topologies, and more.

Radiate's GP implementation provides two core data structures: **Trees** for hierarchical expressions and **Graphs** for complex computational networks. Each offers unique capabilities for different problem domains. Both the `tree` and `graph` modules come with their own specific chromosomes, codecs and alters to evolve these structures effectively.

---

## Installation

To use Radiate's Genetic Programming features, you need to install the library with the appropriate feature flags.

=== ":fontawesome-brands-python: Python"

    ```bash
    pip install radiate
    ```

=== ":fontawesome-brands-rust: Rust"
    ```shell
    cargo add radiate -F gp

    # Or Cargo.toml
    [dependencies]
    radiate = { version = "x", features = ["gp", ...] }
    ```

---

## Overview

| Structure | Best For | Complexity | Use Cases |
|-----------|----------|------------|-----------|
| **[Arity](#arity)** | Number of inputs for nodes and ops | Low | Function arguments, input counts |
| **[Ops](#ops)** | Operations and functions for nodes | Low | Mathematical, logical, activation functions |
| **[Nodes](#nodes)** | Building blocks for trees and graphs | Low | Node types, arity, operations |
| **[Trees](#trees)** | Symbolic regression, mathematical expressions | Low-Medium | Formula discovery, decision trees |
| **[Graphs](#graphs)** | Neural networks, complex computations | Medium-High | Neural evolution, complex programs |

---

## Arity

[Arity](https://en.wikipedia.org/wiki/Arity) is a term used to describe the number of arguments or inputs a function takes. In the context of genetic programming, arity is crucial for defining how many inputs a `node` or an `op` can accept in both `trees` and `graphs`. For `graphs` `arity` is used to determine how many incoming connections a `GraphNode` can have, while for `trees` it determines how many children a `TreeNode` can have. Radiate uses an enum to express `arity` defined in three variants:

1. **Zero**: The operation takes no inputs (e.g., constants).
2. **Exact(usize)**: The operation takes a specific number of inputs.
3. **Any**: The operation can take any number of inputs (e.g., functions like sum or product).

In most cases, the `tree` or `graph` will try it's best ensure that their node's `arity` is not violated, but it will ultimately be up to the user to ensure that the `arity` is correct.

---

## Ops

The `ops` module provides sets of operations and formats for building and evolve genetic programs including `graphs` and `trees`. In the language of radiate, when using an `op`, it is the `Allele` of the `GraphNode` or `TreeNode`.
An `op` is a function that takes a number of inputs and returns a single output. The `op` can be a constant value, a variable, or a function that operates on the inputs.   

The `op` comes in five flavors:

1. **Function Operations**: Stateless functions that take inputs and return a value.
2. **Variable Operations**: Read from an input index, returning the value at that index.
3. **Constant Operations**: Fixed values that do not change - returning the value when called.
4. **Mutable Constant Operations**: Constants that can change over time, allowing for learnable parameters.
5. **Value Operations**: Stateful operations that maintain internal state and can take inputs to produce a value.

Each `op` has an `arity`, definingt the number of inputs it accepts. For example, the `Add` operation has an `arity` of 2 because it takes two inputs and returns their sum. The `Const` operation has an arity of 0 because it does not take any inputs, it just returns it's value. The `Var` operation has an arity of 0 because it takes an index as a parameter, and returns the value of the input at that index. 

Provided `Ops` include:

??? info "Basic ops"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `const` | 0 | x | `Op::constant()` | Const |
    | `named_const` | 0 | x | `Op::named_constant(name)` | Const |
    | `var` | 0 | input[i] - return the value of the input at index `i` | `Op::var(i)` | Var |
    | `identity` |1| return the input value | `Op::identity()` | Fn |


??? info "Basic math operations"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `Add` | 2 | x + y | `Op::add()` | Fn |
    | `Sub` | 2 | x - y | `Op::sub()` | Fn |
    | `Mul` | 2 | x * y | `Op::mul()` | Fn |
    | `Div` | 2 | x / y | `Op::div()` | Fn |
    | `Sum` | Any | Sum of n values | `Op::sum()` | Fn |
    | `Product` | Any | Product of n values | `Op::prod()` | Fn |
    | `Difference` | Any | Difference of n values | `Op::diff()` | Fn |
    | `Neg` | 1 | -x | `Op::neg()` | Fn |
    | `Abs` | 1 | abs(x) | `Op::abs()` | Fn |
    | `pow` | 2 | x^y | `Op::pow()` | Fn |
    | `Sqrt` | 1 | sqrt(x) | `Op::sqrt()` | Fn |
    | `Abs` | 1 | abs(x) | `Op::abs()` | Fn |
    | `Exp` | 1 | e^x | `Op::exp()` | Fn |
    | `Log` | 1 | log(x) | `Op::log()` | Fn |
    | `Sin` | 1 | sin(x) | `Op::sin()` | Fn |
    | `Cos` | 1 | cos(x) | `Op::cos()` | Fn |
    | `Tan` | 1 | tan(x) | `Op::tan()` | Fn |
    | `Max` | Any | Max of n values | `Op::max()` | Fn |
    | `Min` | Any | Min of n values | `Op::min()` | Fn |
    | `Ceil` | 1 | ceil(x) | `Op::ceil()` | Fn |
    | `Floor` | 1 | floor(x) | `Op::floor()` | Fn |
    | `Weight` | 1 | Weighted sum of n values | `Op::weight()` | MutableConst |

??? info "Activation Ops"

    These are the most common activation functions used in Neural Networks.

    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `Sigmoid` | Any | 1 / (1 + e^-x) | `Op::sigmoid()` | Fn |
    | `Tanh` | Any | tanh(x) | `Op::tanh()` | Fn |
    | `ReLU` | Any | max(0, x) | `Op::relu()` | Fn |
    | `LeakyReLU` | Any | x if x > 0 else 0.01x | `Op::leaky_relu()` | Fn |
    | `ELU` | Any | x if x > 0 else a(e^x - 1) | `Op::elu()` | Fn |
    | `Linear` | Any | Linear combination of n values | `Op::linear()` | Fn |
    | `Softmax` | Any | Softmax of n values | `Op::softmax()` | Fn |
    | `Softplus` | Any | log(1 + e^x) | `Op::softplus()` | Fn |
    | `SELU` | Any | x if x > 0 else a(e^x - 1) | `Op::selu()` | Fn |
    | `Swish` | Any | x / (1 + e^-x) | `Op::swish()` | Fn |
    | `Mish` | Any | x * tanh(ln(1 + e^x)) | `Op::mish()` | Fn |
    
??? info "bool Ops"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `And` | 2 | x && y | `Op::and()` | Fn |
    | `Or` | 2 | `x || y` | `Op::or()` | Fn |
    | `Not` | 1 | !x | `Op::not()` | Fn |
    | `Xor` | 2 | x ^ y | `Op::xor()` | Fn |
    | `Nand` | 2 | !(x && y) | `Op::nand()` | Fn |
    | `Nor` | 2 | `!(x || y)` | `Op::nor()` | Fn |
    | `Xnor` | 2 | !(x ^ y) | `Op::xnor()` | Fn |
    | `Equal` | 2 | x == y | `Op::eq()` | Fn |
    | `NotEqual` | 2 | x != y | `Op::ne()` | Fn |
    | `Greater` | 2 | x > y | `Op::gt()` | Fn |
    | `Less` | 2 | x < y | `Op::lt()` | Fn |
    | `GreaterEqual` | 2 | x >= y | `Op::ge()` | Fn |
    | `LessEqual` | 2 | x <= y | `Op::le()` | Fn |
    | `IfElse` | 3 | if x then y else z | `Op::if_else()` | Fn |

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Example usage of an Op
    let fn_op = Op::add();
    let result = fn_op.eval(&[1.0, 2.0]); // result is 3.0

    // Example usage of a constant Op
    let const_op = Op::constant(42.0);
    let result = const_op.eval(&[]); // result is 42.0

    // Example usage of a variable Op
    let var_op = Op::var(0); // Read from input at index 0
    let inputs = var_op.eval(&[5.0, 10.0]); // result is 5.0 when evaluated with inputs
    ```

---

## Nodes 

Nodes are not only the `gene` of the `graph` and `tree`, but the fundamental building blocks of them. Each `node` represents a connection, computation, or operation, and has explicit rules depending on its role in the structure. 

### Roles

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

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

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

### Store

The `NodeStore<T>` manages available values for different node types, providing a centralized way to define what values can be used in each position of your genetic program. This makes it super easy to create `trees` or `graphs` from a specific template or with a specific structure.

**Usage Examples:**

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

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

---

## Trees

A `Tree<T>` represents a hierarchical structure where each node has exactly one parent (except the root) and zero or more children.

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // create a simple tree:
    //              42
    //           /  |   \
    //          1   2    3
    //             / \    
    //            3   4    
    let tree: Tree<i32> = Tree::new(TreeNode::new(42)
        .attach(TreeNode::new(1))
        .attach(TreeNode::new(2)
            .attach(TreeNode::new(3))
            .attach(TreeNode::new(4))
        .attach(TreeNode::new(3))));
    ```

**Key Properties:**

- **Rooted**: Always has a single root node
- **Acyclic**: No node is its own ancestor
- **Hierarchical**: Parent-child relationships

### Node

Each node in a tree contains a value and optional children & arity. The `TreeNode` also implements the `gene` trait, making the node itself a `gene` and it's value the `allele`. 

**Node Types:**

- **Root**: Starting point of the tree (can have any number of children)
- **Vertex**: Internal computation nodes (can have any number of children)
- **Leaf**: Terminal nodes with no children (arity is `Arity::Zero`)

### Codec

The `TreeCodec` is simply a `codec` that encodes a `TreeChromosome` and decodes it back into a `Tree`. The `TreeCodec` can be configured to create a single `tree` or a multi-root `tree` structure. 

**Codec Types:**

- **Single Root**: Creates one tree per `genotype`
- **Multi-Root**: Creates multiple trees per `genotype`

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let store = vec![
        (NodeType::Root, vec![Op::add(), Op::sub()]),
        (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
        (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
    ];

    // Create a single rooted tree codec with a starting (minimum) depth of 3
    let codec = TreeCodec::single(3, store);
    let genotype: Genotype<TreeChromosome<Op<f32>>> = single_root_codec.encode();
    let tree: Tree<Op<f32>> = codec.decode(&genotype);

    // Create a multi-rooted tree codec with a starting (minimum) depth of 3 and 2 trees
    let codec = TreeCodec::multi_root(3, 2, store);
    let genotype: Genotype<TreeChromosome<Op<f32>>> = codec.encode();
    // multi-rooted codec decodes to a Vec of Trees
    // one for each root in the genotype
    let trees: Vec<Tree<Op<f32>>> = codec.decode(&genotype); 
    ```

### Alters

#### HoistMutator

> Inputs
> 
>   * `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**:  Randomly hoists subtrees from one part of the tree to another.
- **Best for**: Evolving trees with complex structures.
- **Example**: Symbolic regression or decision trees.
- **Compatible with**: `TreeNode`, `TreeChromosome`

The `HoistMutator` is a mutation operator that randomly selects a subtree from the tree and moves it to a different location in the tree. This can create new structures and relationships between nodes, allowing for more complex solutions to emerge.

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mutator = HoistMutator::new(0.1);
    ```

#### TreeCrossover

> Inputs
> 
>   * `rate`: f32 - Mutation rate (0.0 to 1.0)

- **Purpose**: Swaps two subtrees between two trees.
- **Best for**: Combining structures from two parent trees.
- **Example**: Evolving decision trees or symbolic expressions.
- **Compatible with**: `TreeNode`, `TreeChromosome`

The `TreeCrossover` is a crossover operator that randomly selects a subtree from one parent tree and swaps it with a subtree from another parent tree.

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let mutator = TreeCrossover::new(0.1);
    ```

---

## Graphs

Graphs are a powerful way to represent problems. They are used in many fields, such as Neural Networks, and can be used to solve complex problems. Radiate thinks of graphs in a more general way than most implementations. Instead of being a collection of inputs, nodes, edges, and outputs, radiate thinks of a graph as simply a bag of nodes that can be connected in any way. Why? Well, because it allows for more flexibility within the graph and it lends itself well to the evolutionary nature of genetic programming. However, this representation is not without it's drawbacks. It can be difficult to reason about the graph and it can be difficult to ensure that the graph is valid. Radiate tries to mitigate these issues by sticking to a few simple rules that govern the graph.

1. Each input node must have 0 incoming connections and at least 1 outgoing connection.
2. Each output node must have at least 1 incoming connection and 0 outgoing connections.
3. Each edge node must have exactly 1 incoming connection and 1 outgoing connection.
4. Each vertex node must have at least 1 incoming connection and at least 1 outgoing connection.

With these rules in mind, we can begin to build and evolve graphs. The graph typically relies on an underlying `GraphArchitect` to construct a valid graph. This architect is a builder pattern that keeps an aggregate of nodes added and their relationships to other nodes. Because of the architect's decoupled nature, we can easily create complex graphs, however it is up to the user to ensure that the desired end graph is valid. 

Radiate provides a few basic graph architectures, but it is also possible to construct your own graph through either the built in graph functions or by using the architect. In most cases building a graph requires a vec of tuples (or a `NodeStore`) where the first element is the `NodeType` and the second element is a vec of values that the `GraphNode` can take. The `NodeType` is either `Input`, `Output`, `Vertex`, or `Edge`. The value of the `GraphNode` is picked at random from the vec of it's `NodeType`.

**Key Properties:**

- **Flexible Connections**: Nodes can have multiple inputs/outputs
- **Indexed Access**: Each node has a unique index in the vector
- **Connection Sets**: Each node maintains incoming/outgoing connections
- **Direction Support**: Can be directed acyclic (DAG) or cyclic

Manually create a simple graph:

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // create a simple graph:
    // 0 -> 1 -> 2
    let mut graph = Graph::<i32>::default();

    let idx_one = graph.insert(NodeType::Input, 0);
    let idx_two = graph.insert(NodeType::Vertex, 1);
    let idx_three = graph.insert(NodeType::Output, 2);

    graph.attach(idx_one, idx_two).attach(idx_two, idx_three);

    // Set cycles in a cyclic graph:
    let mut graph = Graph::<i32>::default();

    let idx_one = graph.insert(NodeType::Input, 0);
    let idx_two = graph.insert(NodeType::Vertex, 1);
    let idx_three = graph.insert(NodeType::Vertex, 2);
    let idx_four = graph.insert(NodeType::Output, 3);

    graph
        .attach(idx_one, idx_two)
        .attach(idx_two, idx_three)
        .attach(idx_three, idx_two)
        .attach(idx_three, idx_four)
        .attach(idx_four, idx_two);

    graph.set_cycles(vec![]);
    ```

Now, the above works just fine, but can become cumbersome quickly. To ease the process of creating a `graph`, we can use the default `graph` types to create graphs in a better way. All we need to do is define a `NodeStore` that contains the possible values for each node given a `NodeType`. 

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Input nodes are picked in order while the rest of the node's values
    // are picked at random.

    // Take note that the NodeType::Input has two variables, [0, 1] 
    // and we create a graph with two input nodes.
    let values = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    // create a directed graph with 2 input nodes and 2 output nodes
    let dag = Graph::directed(2, 2, values);

    // create a recurrent graph with 2 input nodes and 2 output nodes
    let recurrent = Graph::recurrent(2, 2, values);

    // create a weighted directed graph with 2 input nodes and 2 output nodes
    let weighted_dag = Graph::weighted_directed(2, 2, values);

    // create a weighted recurrent graph with 2 input nodes and 2 output nodes
    let weighted_recurrent = Graph::weighted_recurrent(2, 2, values);
    ```


### Node

The `GraphNode` struct is a fundamental building block for graph-based genetic programming in Radiate. It represents a node in a directed graph that can have both incoming and outgoing connections to other nodes. Each node has a unique identifier, an index in the graph, a value of type T, and maintains sets of incoming and outgoing connections. The `GraphNode` can be of different types, such as `Input`, `Output`, `Vertex`, or `Edge`, each serving a specific role in the graph structure. To ensure the integrity of the graph, the `GraphNode` enforces rules based on its type, such as the number of incoming and outgoing connections it can have. In order to facilitate genetic programming, the `GraphNode` implements the `Gene` trait, where it's `allele` is the value of the node, and its `gene` is the node itself. 

**Node Types:**

- **Input**: Entry points (no incoming, one or more outgoing)
- **Output**: Exit points (one or more incoming, no outgoing)
- **Vertex**: Internal computation (both incoming and outgoing)
- **Edge**: Connection nodes (exactly one incoming and one outgoing)

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create a new input node with value 42
    let node = GraphNode::new(0, NodeType::Input, 42);

    // Create a node with specific arity
    // This node will be invalid if it has a number of incoming connections other than 2
    let node_with_arity = GraphNode::with_arity(1, NodeType::Vertex, 42, Arity::Exact(2));
    ```

### Codec

The `GraphCodec` is a codec that encodes a `GraphChromosome` and decodes it back into a `Graph`. The `GraphCodec` can be configured to create directed or recurrent graphs.

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create a store for graph operations
    let store = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Edge, vec![Op::add(), Op::mul()]),
        (NodeType::Vertex, vec![Op::sub(), Op::div()]),
        (NodeType::Output, vec![Op::sigmoid(), Op::tanh()]),
    ];

    // Create a directed graph codec with 2 input nodes and 2 output nodes
    let codec = GraphCodec::directed(2, 2, store);
    let genotype: Genotype<GraphChromosome<Op<f32>>> = codec.encode();
    let graph: Graph<Op<f32>> = codec.decode(&genotype);

    // Create a recurrent graph codec with 2 input nodes and 2 output nodes
    let recurrent_codec = GraphCodec::recurrent(2, 2, store);
    let recurrent_genotype: Genotype<GraphChromosome<Op<f32>>> = recurrent_codec.encode();
    let recurrent_graph: Graph<Op<f32>> = recurrent_codec.decode(&recurrent_genotype);
    ```

### Alters

#### GraphMutator

> Inputs
> 
>   * `vertex_rate`: f32 - Probabilty of adding a vertex to the graph (0.0 to 1.0)
>   * `edge_rate`: f32 - Probabilty of adding an edge to the graph (0.0 to 1.0)
>   * `allow_recurrent`: bool - Whether to allow recurrent connections in the graph. The default is `false`, meaning the graph will be a directed acyclic graph (DAG).

- **Purpose**: Randomly adds vertices and edges to the graph.

This mutator is used to add new nodes and connections to the graph. It can be used to evolve the graph structure over time, allowing for more complex solutions to emerge.

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create a mutator that adds vertices and edges with a 10% chance for either
    let mutator = GraphMutator::new(0.1, 0.1);

    let mutator = GraphMutator::new(0.1, 0.1).allow_recurrent(true); // Allow recurrent connections
    ```

#### GraphCrossover

> Inputs
> 
>   * `rate`: f32 - Crossover rate (0.0 to 1.0)
>   * `cross_parent_node_rate`: f32 - Probability of the less fit parent taking a node from the more fit parent (0.0 to 1.0)

- **Purpose**: Swaps node value's (`alleles`) between two graphs.

This crossover operator is used to combine two parent graphs by swapping the values of their nodes. It can be used to create new graphs that inherit the structure and values of their parents. Given that a more fit parent's node's `arity` matches the less fit parent's node's `arity`, the less fit parent will take (inherit) the more fit parent's node's value. This means the child is guaranteed to have the same structure as the less fit parent, but with some of the more fit parent's values (`alleles`). This process is extremely similar to how the [NEAT](https://en.wikipedia.org/wiki/NeuroEvolution_of_Augmenting_Topologies) algorithm works.

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        Python's GP is still under development and will be available in a future release.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create a mutator that adds vertices and edges with a 10% chance for either
    let crossover = GraphCrossover::new(0.1, 0.5);
    ```

---

<!-- 
### TreeCodec
Encodes and decodes tree structures for genetic algorithms:

```rust
pub struct TreeCodec<T: Clone, D = Vec<Tree<T>>> {
    depth: usize,                                    // Maximum tree depth
    num_trees: usize,                                // Number of trees to generate
    store: Option<NodeStore<T>>,                     // Available operations
    constraint: Option<Constraint<TreeNode<T>>>,     // Validation rules
    template: Option<Tree<T>>,                       // Template tree
}
```

**Codec Types:**
- **Single Root**: Creates one tree per chromosome
- **Multi-Root**: Creates multiple trees per chromosome

**Usage:**
```rust
// Single tree codec
let codec = TreeCodec::single(5, operations)
    .constraint(|node| node.size() < 30);

// Multi-tree codec
let codec = TreeCodec::multi_root(3, 4, operations)
    .constraint(|node| node.depth() < 10);
```

### GraphCodec
Encodes and decodes graph structures for genetic algorithms:

```rust
pub struct GraphCodec<T> {
    store: NodeStore<T>,              // Available operations
    template: GraphChromosome<T>,     // Template graph structure
}
```

**Codec Types:**
- **Directed**: Creates directed acyclic graphs (DAGs)
- **Recurrent**: Creates graphs with cyclic connections

**Usage:**
```rust
// Directed graph codec
let codec = GraphCodec::directed(2, 1, operations);

// Recurrent graph codec
let codec = GraphCodec::recurrent(2, 1, operations);
```

## Genes and Alleles

### Operations (Ops) as Alleles
In GP, the **allele** of each node is an `Op<T>` that defines the node's behavior:

```rust
pub enum Op<T> {
    // Function operations (stateless)
    Fn(&'static str, Arity, Arc<dyn Fn(&[T]) -> T>),
    
    // Variable operations (read from input)
    Var(&'static str, usize),
    
    // Constant operations (fixed values)
    Const(&'static str, T),
    
    // Mutable constants (learnable parameters)
    MutableConst {
        name: &'static str,
        arity: Arity,
        value: T,
        supplier: Arc<dyn Fn() -> T>,
        modifier: Arc<dyn Fn(&T) -> T>,
        operation: Arc<dyn Fn(&[T], &T) -> T>,
    },
    
    // Value operations (stateful)
    Value(&'static str, Arity, T, Arc<dyn Fn(&[T], &T) -> T>),
}
```

**Operation Types:**
- **Function**: Stateless operations like `add`, `multiply`, `sigmoid`
- **Variable**: Input variables like `Op::var(0)` for first input
- **Constant**: Fixed values like `Op::constant(3.14)`
- **Mutable Constant**: Learnable parameters that can change during evolution
- **Value**: Stateful operations that maintain internal state

### Arity System
Arity defines how many inputs an operation expects:

```rust
pub enum Arity {
    Zero,           // No inputs (constants, variables)
    Exact(usize),   // Exactly N inputs
    Any,            // Any number of inputs
}
```

**Arity Rules:**
- **Tree Nodes**: Arity determines number of children
- **Graph Nodes**: Arity determines number of incoming connections
- **Validation**: Nodes are invalid if connections don't match arity

## Node Stores

### NodeStore
Manages available operations for different node types:

```rust
pub struct NodeStore<T> {
    store: HashMap<NodeType, Vec<T>>,
}
```

**Usage:**
```rust
let store = vec![
    (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
    (NodeType::Leaf, vec![Op::var(0), Op::var(1)]),
    (NodeType::Input, vec![Op::var(0)]),
    (NodeType::Output, vec![Op::sigmoid()]),
];
```

**Node Type Mapping:**
- **Tree**: `Root`, `Vertex`, `Leaf`
- **Graph**: `Input`, `Output`, `Vertex`, `Edge`

## Genetic Operations

### Tree Operations

#### Tree Crossover
Swaps subtrees between parent trees:

```rust
let crossover = TreeCrossover::new(0.7);
```

**Process:**
1. Select crossover points in both parents
2. Swap subtrees at those points
3. Ensure resulting trees are valid

#### Hoist Mutation
Moves a subtree to a new position:

```rust
let mutator = HoistMutator::new(0.05);
```

**Process:**
1. Select a random subtree
2. Move it to a new valid position
3. Maintain tree structure integrity

#### Operation Mutation
Changes operation types at nodes:

```rust
let mutator = OperationMutator::new(0.03, 0.02);
```

**Process:**
1. Select random nodes
2. Replace operations with compatible alternatives
3. Maintain arity compatibility

### Graph Operations

#### Graph Crossover
Combines subgraphs from parents:

```rust
let crossover = GraphCrossover::new(0.5, 0.5);
```

**Process:**
1. Select subgraphs from both parents
2. Combine them into offspring
3. Maintain graph connectivity

#### Graph Mutation
Modifies graph topology:

```rust
let mutator = GraphMutator::new(0.1, 0.1)
    .allow_recurrent(false);
```

**Process:**
1. Add/remove nodes and connections
2. Modify graph structure
3. Ensure graph validity

#### Operation Mutation
Changes node operations:

```rust
let mutator = OperationMutator::new(0.05, 0.05);
```

**Process:**
1. Select random nodes
2. Replace operations
3. Maintain arity compatibility

## Evaluation

### Tree Evaluation
Trees are evaluated in a top-down, deterministic manner:

```rust
// Direct evaluation
let result = tree.eval(&[1.0, 2.0, 3.0]);

// Tree evaluation follows the structure:
// 1. Start at root node
// 2. Evaluate children recursively
// 3. Apply operation to child results
// 4. Return final result
```

### Graph Evaluation
Graphs require more complex evaluation strategies:

```rust
// Create evaluator
let mut evaluator = GraphEvaluator::new(&graph);

// Evaluate with input
let result = evaluator.eval_mut(&[1.0, 2.0, 3.0]);

// Graph evaluation process:
// 1. Topological sort (for DAGs)
// 2. Iterative evaluation (for cyclic graphs)
// 3. State management (for recurrent graphs)
```

## Constraints and Validation

### Tree Constraints
Enforce tree structure rules:

```rust
let codec = TreeCodec::single(5, operations)
    .constraint(|node| node.size() < 30)           // Size limit
    .constraint(|node| node.depth() < 8)           // Depth limit
    .constraint(|node| node.leaf_count() > 2);     // Minimum leaves
```

### Graph Validation
Ensure graph connectivity and validity:

```rust
// Graph validity rules:
// - Input nodes: no incoming, at least one outgoing
// - Output nodes: at least one incoming, no outgoing
// - Vertex nodes: both incoming and outgoing
// - Edge nodes: exactly one incoming and one outgoing
```

## Advanced Features

### Recurrent Graphs
Support cyclic connections for memory and state:

```rust
let codec = GraphCodec::recurrent(2, 1, operations);
let mutator = GraphMutator::new(0.1, 0.1)
    .allow_recurrent(true);
```

### Memory Graphs
Graphs with special memory nodes:

```rust
// Memory nodes maintain state across evaluations
let memory_op = Op::MutableConst {
    name: "memory",
    arity: Arity::Exact(1),
    value: 0.0,
    supplier: Arc::new(|| 0.0),
    modifier: Arc::new(|old| old + 1.0),
    operation: Arc::new(|inputs, state| inputs[0] + state),
};
```

### Custom Operations
Define domain-specific operations:

```rust
let custom_op = Op::Fn("custom", Arity::Exact(2), Arc::new(|inputs| {
    let x = inputs[0];
    let y = inputs[1];
    x * x + y * y  // Custom function
}));
```

## Performance Considerations

### Tree Performance
- **Evaluation**: O(n) where n is number of nodes
- **Memory**: Minimal overhead
- **Mutation**: Fast subtree operations
- **Crossover**: Efficient subtree swapping

### Graph Performance
- **Evaluation**: O(V + E) where V is nodes, E is edges
- **Memory**: Higher overhead for connection tracking
- **Mutation**: More complex topology changes
- **Crossover**: Subgraph extraction and combination

### Optimization Tips
- **Limit Tree Size**: Prevent bloat with size constraints
- **Use Appropriate Arity**: Match operations to expected inputs
- **Cache Evaluations**: Reuse results when possible
- **Parallel Evaluation**: Use worker pools for large populations

## Complete Examples

### Symbolic Regression with Trees
```rust
use radiate::*;

fn main() {
    // Define operations
    let store = vec![
        (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul(), Op::pow()]),
        (NodeType::Leaf, vec![Op::var(0), Op::constant(1.0), Op::constant(2.0)]),
    ];
    
    // Create codec with constraints
    let codec = TreeCodec::single(5, store)
        .constraint(|root| root.size() < 20);
    
    // Set up problem
    let problem = Regression::new(dataset, Loss::MSE, codec);
    
    // Build engine
    let engine = GeneticEngine::builder()
        .problem(problem)
        .minimizing()
        .mutator(HoistMutator::new(0.05))
        .crossover(TreeCrossover::new(0.7))
        .build();
    
    // Run evolution
    let result = engine.iter()
        .until_score_below(0.001)
        .take(1)
        .last()
        .unwrap();
    
    println!("Best expression: {}", result.value().format());
}
```

### Neural Network Evolution with Graphs
```rust
use radiate::*;

fn main() {
    // Define graph operations
    let values = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sigmoid(), Op::tanh()]),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];
    
    // Create graph codec
    let codec = GraphCodec::directed(2, 1, values);
    let problem = Regression::new(dataset, Loss::MSE, codec);
    
    // Build engine with graph-specific operators
    let engine = GeneticEngine::builder()
        .problem(problem)
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.05, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false),
        ))
        .build();
    
    // Run evolution
    let result = engine.iter()
        .until_score_below(0.01)
        .take(1)
        .last()
        .unwrap();
    
    // Test the evolved network
    let mut evaluator = GraphEvaluator::new(result.value());
    for input in &[[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]] {
        let output = evaluator.eval_mut(input)[0];
        println!("{:?} -> {:.3}", input, output);
    }
}
``` -->

<--

####### HERE

!!! warning 

    As of 3/5/2025:

    This crate is still being finalized and is not yet documented to the extent it should be. If you are interested in using it, please refer to the [examples](https://github.com/pkalivas/radiate/tree/master/examples) for now - they are current. The documentation will be added as functionality is finalized. 

    Just for reassurance, this crate is pretty much done and ready for production use, I'm just not totally happy with a few very small details. This is just a personal preference and I want to make sure I'm happy with the general flow of the code before I fully document it (this stuff takes a lot of time to write).

## Ops

The `Ops` module provides sets of operations and formats for building your own that can be used to build and evolve genetic programs including Graphs and Trees. In the language of `radiate`, when using an `Op`, it is the `Allele` of the `GraphNode` or `TreeNode`.
An `Op` is a function that takes a number of inputs and returns a single output. The `Op` can be a constant value, a variable, or a function that operates on the inputs. 

In `radiate` an `Op` is an enum defined as:

```rust
pub enum Op<T> {
    /// 1) A stateless function operation:
    ///
    /// # Arguments
    ///    - A `&'static str` name (e.g., "Add", "Sigmoid")
    ///    - Arity (how many inputs it takes)
    ///    - Arc<dyn Fn(&[T]) -> T> for the actual function logic
    Fn(&'static str, Arity, Arc<dyn Fn(&[T]) -> T>),
    /// 2) A variable-like operation:
    ///
    /// # Arguments
    ///    - `String` = a name or identifier
    ///    - `usize` = perhaps an index to retrieve from some external context
    Var(&'static str, usize),
    /// 3) A compile-time constant: e.g., 1, 2, 3, etc.
    ///
    /// # Arguments
    ///    - `&'static str` name
    ///    - `T` the actual constant value
    Const(&'static str, T),
    /// 4) A `mutable const` is a constant that can change over time:
    ///
    ///  # Arguments
    /// - `&'static str` name
    /// - `Arity` of how many inputs it might read
    /// - Current value of type `T`
    /// - An `Arc<dyn Fn() -> T>` for retrieving (or resetting) the value
    /// - An `Arc<dyn Fn(&[T], &T) -> T>` for updating or combining inputs & old value -> new value
    ///
    ///    This suggests a node that can mutate its internal state over time, or
    ///    one that needs a special function to incorporate the inputs into the next state.
    MutableConst {
        name: &'static str,
        arity: Arity,
        value: T,
        supplier: Arc<dyn Fn() -> T>,
        modifier: Arc<dyn Fn(&T) -> T>,
        operation: Arc<dyn Fn(&[T], &T) -> T>,
    },
    /// 5) A 'Value' operation that can be used as a 'stateful' constant:
    ///
    /// # Arguments
    /// - `&'static str` name
    /// - `Arity` of how many inputs it might read
    /// - Current value of type `T`
    /// - An `Arc<dyn Fn(&[T], &T) -> T>` for updating or combining inputs & old value -> new value
    Value(&'static str, Arity, T, Arc<dyn Fn(&[T], &T) -> T>),
}
```

You might have noticed that each `Op` contains a field called `Arity`. This is the number of inputs the `Op` takes. For example, the `Add` operation has an arity of 2 because it takes two inputs and returns their sum. The `Const` operation has an arity of 0 because it does not take any inputs. The `Var` operation has an arity of 0 because it takes an index and returns the value of the input at that index. In traditional mathmatics an `Arity` is simply the number of arguments a function takes - for example, `f(x) = x^2` has an `Arity` of 1. `radiate` treats this much the same way, but allows for more complex operations that can take any number of inputs. Because of this, `radiate` abscracts this concept into an `Arity` type defined as:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Arity {
    Zero,
    Exact(usize),
    Any,
}
```

In GP, each node in either a `Graph` or a `Tree` relies heavily on the `Arity` provided by it's allele (`Op`). For example, a `GraphNode` can only have as many inputs as it's `Op`'s `Arity` allows and a `TreeNode` can only have as many children as it's `Op`'s `Arity` allows. If this is violated, the node is considered invalid and will result in the entire `Graph` or `Tree` being invalid. In most cases, the `Tree` or `Graph` will try it's best ensure that the `Arity` is not violated, but it will ultimately be up to the user to ensure that the `Arity` is correct.

Provided `Ops` include:

??? info "Basic ops"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `const` | 0 | x | `Op::constant()` | Const |
    | `named_const` | 0 | x | `Op::named_constant(name)` | Const |
    | `var` | 0 | input[i] - return the value of the input at index `i` | `Op::var(i)` | Var |
    | `identity` |1| return the input value | `Op::identity()` | Fn |


??? info "Basic math operations"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `Add` | 2 | x + y | `Op::add()` | Fn |
    | `Sub` | 2 | x - y | `Op::sub()` | Fn |
    | `Mul` | 2 | x * y | `Op::mul()` | Fn |
    | `Div` | 2 | x / y | `Op::div()` | Fn |
    | `Sum` | Any | Sum of n values | `Op::sum()` | Fn |
    | `Product` | Any | Product of n values | `Op::prod()` | Fn |
    | `Difference` | Any | Difference of n values | `Op::diff()` | Fn |
    | `Neg` | 1 | -x | `Op::neg()` | Fn |
    | `Abs` | 1 | abs(x) | `Op::abs()` | Fn |
    | `pow` | 2 | x^y | `Op::pow()` | Fn |
    | `Sqrt` | 1 | sqrt(x) | `Op::sqrt()` | Fn |
    | `Abs` | 1 | abs(x) | `Op::abs()` | Fn |
    | `Exp` | 1 | e^x | `Op::exp()` | Fn |
    | `Log` | 1 | log(x) | `Op::log()` | Fn |
    | `Sin` | 1 | sin(x) | `Op::sin()` | Fn |
    | `Cos` | 1 | cos(x) | `Op::cos()` | Fn |
    | `Tan` | 1 | tan(x) | `Op::tan()` | Fn |
    | `Max` | Any | Max of n values | `Op::max()` | Fn |
    | `Min` | Any | Min of n values | `Op::min()` | Fn |
    | `Ceil` | 1 | ceil(x) | `Op::ceil()` | Fn |
    | `Floor` | 1 | floor(x) | `Op::floor()` | Fn |
    | `Weight` | 1 | Weighted sum of n values | `Op::weight()` | MutableConst |

??? info "Activation Ops"

    These are the most common activation functions used in Neural Networks.

    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `Sigmoid` | Any | 1 / (1 + e^-x) | `Op::sigmoid()` | Fn |
    | `Tanh` | Any | tanh(x) | `Op::tanh()` | Fn |
    | `ReLU` | Any | max(0, x) | `Op::relu()` | Fn |
    | `LeakyReLU` | Any | x if x > 0 else 0.01x | `Op::leaky_relu()` | Fn |
    | `ELU` | Any | x if x > 0 else a(e^x - 1) | `Op::elu()` | Fn |
    | `Linear` | Any | Linear combination of n values | `Op::linear()` | Fn |
    | `Softmax` | Any | Softmax of n values | `Op::softmax()` | Fn |
    | `Softplus` | Any | log(1 + e^x) | `Op::softplus()` | Fn |
    | `SELU` | Any | x if x > 0 else a(e^x - 1) | `Op::selu()` | Fn |
    | `Swish` | Any | x / (1 + e^-x) | `Op::swish()` | Fn |
    | `Mish` | Any | x * tanh(ln(1 + e^x)) | `Op::mish()` | Fn |
    
??? info "bool Ops"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `And` | 2 | x && y | `Op::and()` | Fn |
    | `Or` | 2 | `x || y` | `Op::or()` | Fn |
    | `Not` | 1 | !x | `Op::not()` | Fn |
    | `Xor` | 2 | x ^ y | `Op::xor()` | Fn |
    | `Nand` | 2 | !(x && y) | `Op::nand()` | Fn |
    | `Nor` | 2 | `!(x || y)` | `Op::nor()` | Fn |
    | `Xnor` | 2 | !(x ^ y) | `Op::xnor()` | Fn |
    | `Equal` | 2 | x == y | `Op::eq()` | Fn |
    | `NotEqual` | 2 | x != y | `Op::ne()` | Fn |
    | `Greater` | 2 | x > y | `Op::gt()` | Fn |
    | `Less` | 2 | x < y | `Op::lt()` | Fn |
    | `GreaterEqual` | 2 | x >= y | `Op::ge()` | Fn |
    | `LessEqual` | 2 | x <= y | `Op::le()` | Fn |
    | `IfElse` | 3 | if x then y else z | `Op::if_else()` | Fn |
    

## Graphs

Graphs are a powerful way to represent problems. They are used in many fields, such as Neural Networks, and can be used to solve complex problems. Radiate-gp thinks of graphs in a more general way than most implementations. Instead of being a collection of inputs, nodes, edges, and outputs, radiate-gp thinks of a graph as simply a bag of nodes that can be connected in any way. Why? Well, because it allows for more flexibility within the graph and it lends itself well to the evolutionary nature of genetic programming. However, this representation is not without it's drawbacks. It can be difficult to reason about the graph and it can be difficult to ensure that the graph is valid. Radiate-gp tries to mitigate these issues by sticking to a few simple rules that govern the graph.

1. Each input node must have 0 incoming connections and at least 1 outgoing connection.
2. Each output node must have at least 1 incoming connection and 0 outgoing connections.
3. Each edge node must have exactly 1 incoming connection and 1 outgoing connection.
4. Each vertex node must have at least 1 incoming connection and at least 1 outgoing connection.

With these rules in mind, we can begin to build and evolve graphs. The graph typically relies on an underlying `GraphArchitect` to construct a valid graph. This architect is a builder pattern that keeps an aggregate of nodes added and their relationships to other nodes. Because of the architect's decoupled nature, we can easily create complex graphs, however it is up to the user to ensure that the desired end graph is valid. 

Radiate-gp provides a few basic graph architectures, but it is also possible to construct your own graph through either the built in graph functions or by using the architect. In most cases building a graph requires a vec of tuples where the first element is the `NodeType` and the second element is a vec of values that the `GraphNode` can take. The `NodeType` is either `Input`, `Output`, `Vertex`, or `Edge`. The value of the `GraphNode` is picked at random from the vec of it's `NodeType`.

??? info "Directed Acyclic Graphs (DAGs)"

    ``` rust
    let values = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let graph = Graph::directed(2, 1, values);
    ```
    This will create a `Graph` with 2 inputs and 1 output with a sigmoid activation function and will look like this:

    ```plaintext
    Input0 ---
                \
                  Sigmoid
                /
    Input1 ---
    ```

    Its also possible to create the same graph manually like so

    ``` rust
    let mut graph = Graph::new(vec![
        GraphNode::new(0, NodeType::Input, Op::var(0)),
        GraphNode::new(1, NodeType::Input, Op::var(1)),
        GraphNode::new(2, NodeType::Output, Op::sigmoid()),
    ])

    graph.attach(0, 2).attach(1, 2);
    ```

??? info "Directed Cyclic Graphs (DCGs)"

    ``` rust
    let values = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight(), Op::identity()]),
        (NodeType::Vertex, ops::all_ops()),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let graph = Graph::recurrent(1, 1, values);
    ```

    This will create a `Graph` with 1 input, 1 output, 1 vertex with a cycle back to itself, and 1 output with a sigmoid activation function.
    It will look like this:

    ```plaintext
                     ___________________
                    /                   \
    Input0 --- Vertex --- Vertex    Sigmoid
            \_____________/
    ```

### Codec

The `GraphCodec` is much the same as other codeces gone over previously. It's encode function will produce a Genotype with a single `GraphChromosome` representing one graph, while the decode function will take a Genotype and produce a single `Graph`. The graph codec can be created similar to how a graph is created. It requires a set of values that a `GraphNode` can take the type of graph that is desired. 

A codec for a directed graph with 2 inputs and 1 output might look like this:
```rust
let values = vec![
    (NodeType::Input, vec![Op::var(0), Op::var(1)]),
    (NodeType::Output, vec![Op::sigmoid()]),
];

let codec = GraphCodec::directed(2, 1, values);
```

while a codec for a directed cyclic graph with 2 inputs and 2 outputs might look like this:
```rust
let values = vec![
    (NodeType::Input, vec![Op::var(0), Op::var(1)]),
    (NodeType::Edge, vec![Op::weight(), Op::identity()]),
    (NodeType::Vertex, ops::all_ops()),
    (NodeType::Output, vec![Op::sigmoid(), Op::tanh()]),
];

let codec = GraphCodec::recurrent(2, 2, values);
``` 

### Alters

=== "GraphCrossover"
    
    > Inputs
	>
	> * `rate`: f32 - Crossover rate.
	> * `crossover_parent_rate`: f32 - The rate at which the nodes of the fittest parent are copied.

	Borrowing from the popular [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) (NeuroEvoltuion of Augmenting Topologies).
	The `GraphCrossover` operator is used to crossover two `Graph`s. It works by 
	first copying the nodes from the fittest parent, then iterating over the edges of the other parent and adding them to the child if a random value is less than the `crossover_parent_rate`. 

	Create a new `GraphCrossover` with a crossover rate of `0.7` and a parent rate of `0.5`
	```rust
	let crossover = GraphCrossover::new(0.7, 0.5);
	```

=== "GraphMutator"

	> Inputs
	>
	> * `edge_rate`: f32 - Mutation rate.
	> * `node_rate`: f32 - Mutation rate.
	> * `allow_recurrent`: bool - Allow recurrent connections to be made (default: true).

	The `GraphMutator` adds edges or nodes to a `Graph` with the given probabilities. It uses a transaction to add these nodes to the graph so invalid mutations can be rolled back allowing for extremely efficeint graph mutation. If `allow_recurrent` is set to `true`, the mutator will allow recurrent connections to be made. The `GraphMutator` will return a metric of invalid mutations created so the user can adjust the mutation rates accordingly.

	Create a new `GraphMutator` with an edge mutation rate of `0.1`, a node mutation rate of `0.1`, and disallow recurrent connections.
	```rust
	let mutator = GraphMutator::new(0.1, 0.1).allow_recurrent(false);
	```

=== "OperationMutator"

	> Inputs
	>
	> * `rate`: f32 - Mutation rate.
	> * `replace_rate`: f32 - Rate at which the `Op` is replaced with a new `Op`.

	The `OperationMutator` mutates the `Op` of a `GraphNode` in a `Graph`. It works by iterating over the nodes of the graph and determining if the `Op` should be mutated or replaced. If the `Op` is to be mutated, the `Op` is mutated by changing the `Op`'s internal properties. If the `Op` is to be replaced, the `Op` is replaced with a new `Op`. It is gaurenteed that the `Op`'s `Arity` will not change which ensures that that the graph remains valid.

	Create a new `OperationMutator` with a mutation rate of `0.1` and a replace rate of `0.1`.
	```rust
	let mutator = OpMutator::new(0.1, 0.1);
	```

## Trees

Trees use a very similar pattern to the `Graph` but are more simple in nature. 
The `Architect` for trees, grows a tree given a desired starting minimum depth.

`Tree`s can be evolved using the:

* `TreeCrossover` - Crossover two subtrees of a `Tree`.
* `NodeMutator` - Mutate a `Node` by editing its internal (its `Allele`) properties.
* `NodeCrossover` - Crossover two `Node`s by swapping their internal properties. -->


-->