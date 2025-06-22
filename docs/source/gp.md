# Genetic Programming

!!! warning ":construction: Under Construction :construction:"

    These docs are a work in progress and may not be complete or accurate. Please check back later for updates.

___
Genetic Programming (GP) in Radiate enables the evolution of programs represented as **expression trees** and **computational graphs**. This powerful feature allows you to solve complex problems by evolving mathematical expressions, decision trees, neural network topologies, and more.

Radiate's GP implementation provides two core data structures: **Trees** for hierarchical expressions and **Graphs** for complex computational networks. Each offers unique capabilities for different problem domains. Both the `tree` and `graph` modules come with their own specific chromosomes, codecs and alters to evolve these structures effectively.

## Overview

| Structure | Best For | Complexity | Use Cases |
|-----------|----------|------------|-----------|
| **Trees** | Symbolic regression, mathematical expressions | Low-Medium | Formula discovery, decision trees |
| **Graphs** | Neural networks, complex computations | Medium-High | Neural evolution, complex programs |
| **Advanced Graphs** | Memory, recurrence, complex topologies | High | Recurrent networks, stateful programs |

## Core Data Structures

### Trees

#### Tree Structure
A `Tree<T>` represents a hierarchical structure where each node has exactly one parent (except the root) and zero or more children.

```rust
pub struct Tree<T> {
    root: Option<TreeNode<T>>,
}
```

**Key Properties:**

- **Rooted**: Always has a single root node
- **Acyclic**: No node is its own ancestor
- **Hierarchical**: Parent-child relationships

#### TreeNode
Each node in a tree contains a value and optional children & arity. The `TreeNode<T>` also implements the `gene` trait, making the node itself a `gene` and it's value the `allele`. 

```rust
pub struct TreeNode<T> {
    value: T,                    // The value
    arity: Option<Arity>,        // How many children this node can have
    children: Option<Vec<TreeNode<T>>>, // Child nodes
}
```

**Node Types:**

- **Root**: Starting point of the tree (can have any number of children)
- **Vertex**: Internal computation nodes (can have any number of children)
- **Leaf**: Terminal nodes with no children (arity is `Arity::Zero`)

**Tree Operations:**
```rust
// Create a tree
let tree = Tree::new(TreeNode::new(Op::add()));

// Add children
let node = TreeNode::new(Op::add())
    .attach(TreeNode::new(Op::constant(1.0)))
    .attach(TreeNode::new(Op::constant(2.0)));

// Tree properties
let size = tree.size();      // Total number of nodes
let height = tree.height();  // Maximum depth
```

#### TreeChromosome
A chromosome that represents a tree structure for genetic operations:

```rust
pub struct TreeChromosome<T> {
    nodes: Vec<TreeNode<T>>,                    // The tree nodes
    store: Option<NodeStore<T>>,                // Available operations
    constraint: Option<Constraint<TreeNode<T>>>, // Validation rules
}
```

**Features:**

- **Gene Collection**: Contains a single node in it's `nodes` vector - the root node
- **Node Store**: Manages available `alleles` for mutation
- **Constraints**: Enforces `tree` validity rules
- **Serialization**: Supports saving/loading evolved trees

### Graphs

#### Graph Structure
A `Graph<T>` represents a collection of interconnected nodes with flexible connections:

```rust
pub struct Graph<T> {
    nodes: Vec<GraphNode<T>>,
}
```

**Key Properties:**
- **Flexible Connections**: Nodes can have multiple inputs/outputs
- **Indexed Access**: Each node has a unique index in the vector
- **Connection Sets**: Each node maintains incoming/outgoing connections
- **Direction Support**: Can be directed acyclic (DAG) or cyclic

#### GraphNode
Each node in a graph contains a value and connection information:

```rust
pub struct GraphNode<T> {
    value: T,                           // The operation or value
    id: GraphNodeId,                    // Unique identifier
    index: usize,                       // Position in graph vector
    direction: Direction,               // Forward or Backward
    node_type: Option<NodeType>,        // Input, Output, Vertex, Edge
    arity: Option<Arity>,               // Expected number of inputs
    incoming: BTreeSet<usize>,          // Indices of input nodes
    outgoing: BTreeSet<usize>,          // Indices of output nodes
}
```

**Node Types:**
- **Input**: Entry points (no incoming, one or more outgoing)
- **Output**: Exit points (one or more incoming, no outgoing)
- **Vertex**: Internal computation (both incoming and outgoing)
- **Edge**: Connection nodes (exactly one incoming and one outgoing)

**Connection Direction:**
- **Forward**: Normal data flow (default)
- **Backward**: Recurrent connections for cycles

**Graph Operations:**
```rust
// Create a graph
let mut graph = Graph::default();

// Add nodes
let input_idx = graph.insert(NodeType::Input, Op::var(0));
let output_idx = graph.insert(NodeType::Output, Op::sigmoid());

// Connect nodes
graph.attach(input_idx, output_idx);

// Graph properties
let len = graph.len();           // Number of nodes
let inputs = graph.inputs();     // All input nodes
let outputs = graph.outputs();   // All output nodes
```

#### GraphChromosome
A chromosome that represents a graph structure for genetic operations:

```rust
pub struct GraphChromosome<T> {
    nodes: Vec<GraphNode<T>>,    // The graph nodes
    store: Option<NodeStore<T>>, // Available operations
}
```

**Features:**
- **Gene Collection**: Contains all graph nodes as genes
- **Node Store**: Manages available operations for mutation
- **Factory Pattern**: Can create new instances with different operations
- **Graph Conversion**: Easily convertible to/from Graph structure

## Codecs

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
```



<!-- # Genetic Programming

___
Genetic Programming (GP) in Radiate enables the evolution of programs represented as **expression trees** and **computational graphs**. This powerful feature allows you to solve complex problems by evolving mathematical expressions, decision trees, neural network topologies, and more.

Radiate's GP implementation provides two core data structures: **Trees** for hierarchical expressions and **Graphs** for complex computational networks. Each offers unique capabilities for different problem domains.

## Overview

| Structure | Best For | Complexity | Performance | Use Cases |
|-----------|----------|------------|-------------|-----------|
| **Trees** | Symbolic regression, mathematical expressions | Low-Medium | High | Formula discovery, decision trees |
| **Graphs** | Neural networks, complex computations | Medium-High | Medium | Neural evolution, complex programs |
| **Advanced Graphs** | Memory, recurrence, complex topologies | High | Lower | Recurrent networks, stateful programs |

## Quick Start

### Symbolic Regression with Trees

Evolve mathematical expressions to fit your data:

```rust
use radiate::*;

// Define available operations
let store = vec![
    (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
    (NodeType::Leaf, vec![Op::var(0)]),
];

// Create tree codec with size constraints
let tree_codec = TreeCodec::single(3, store)
    .constraint(|root| root.size() < 30);

// Set up regression problem
let problem = Regression::new(dataset, Loss::MSE, tree_codec);

// Build and run genetic engine
let engine = GeneticEngine::builder()
    .problem(problem)
    .minimizing()
    .mutator(HoistMutator::new(0.01))
    .crossover(TreeCrossover::new(0.7))
    .build();

let result = engine.iter()
    .until_score_below(0.01)
    .take(1)
    .last()
    .unwrap();

println!("Best expression: {}", result.value().format());
```

### Neural Network Evolution with Graphs

Evolve neural network topologies:

```rust
use radiate::*;

// Define graph structure
let values = vec![
    (NodeType::Input, vec![Op::var(0)]),
    (NodeType::Edge, vec![Op::weight()]),
    (NodeType::Vertex, vec![Op::sigmoid(), Op::tanh()]),
    (NodeType::Output, vec![Op::linear()]),
];

let graph_codec = GraphCodec::directed(1, 1, values);
let problem = Regression::new(dataset, Loss::MSE, graph_codec);

let engine = GeneticEngine::builder()
    .problem(problem)
    .minimizing()
    .alter(alters!(
        GraphCrossover::new(0.5, 0.5),
        OperationMutator::new(0.07, 0.05),
        GraphMutator::new(0.1, 0.1).allow_recurrent(false),
    ))
    .build();
```

## Core Concepts

### Operations (Ops)

Operations are the fundamental building blocks of genetic programs. Each node in a tree or graph contains an operation that defines its behavior and computation.

#### Operation Types

**Function Operations** (`Op::Fn`)
- Stateless functions that transform inputs
- Examples: `add`, `multiply`, `sigmoid`, `sin`
- Arity: Fixed number of inputs

**Variable Operations** (`Op::Var`)
- Input variables that read from external data
- Examples: `Op::var(0)` reads the first input
- Arity: Zero (no inputs, reads from context)

**Constant Operations** (`Op::Const`)
- Fixed values that don't change
- Examples: `Op::constant(3.14)`
- Arity: Zero

**Mutable Constants** (`Op::MutableConst`)
- Values that can change during evolution
- Examples: Learnable parameters, weights
- Arity: Variable

**Value Operations** (`Op::Value`)
- Stateful operations that maintain internal state
- Examples: Memory cells, accumulators
- Arity: Variable

#### Arity System

Arity defines how many inputs an operation expects:

```rust
pub enum Arity {
    Zero,           // No inputs (constants, variables)
    Exact(usize),   // Exactly N inputs
    Any,            // Any number of inputs
}
```

### Trees vs Graphs

#### Trees
- **Structure**: Hierarchical with single parent per node
- **Evaluation**: Top-down, deterministic
- **Best for**: Mathematical expressions, decision trees
- **Advantages**: Fast evaluation, easy to understand
- **Limitations**: Limited expressiveness, no cycles

#### Graphs
- **Structure**: Flexible connections between nodes
- **Evaluation**: Topological sort or iteration
- **Best for**: Neural networks, complex computations
- **Advantages**: High expressiveness, supports cycles
- **Limitations**: Slower evaluation, more complex

## Tree Programming

### Tree Structure

Trees represent hierarchical expressions where each node has exactly one parent (except the root) and zero or more children.

```rust
// Example tree: (x + y) * 2
//      *
//     / \
//    +   2
//   / \
//  x   y
```

### Tree Codecs

**Single Root Trees**
```rust
let codec = TreeCodec::single(max_depth, operations);
```

**Multi-Root Trees**
```rust
let codec = TreeCodec::multi_root(max_depth, num_roots, operations);
```

**Constrained Trees**
```rust
let codec = TreeCodec::single(3, operations)
    .constraint(|root| root.size() < 30)
    .constraint(|root| root.depth() < 10);
```

### Tree Operations

**Hoist Mutation**
- Moves a subtree to a new position
- Preserves tree structure
- Good for exploring different arrangements

**Tree Crossover**
- Swaps subtrees between parents
- Maintains valid tree structure
- Primary recombination operator

**Operation Mutation**
- Changes operation types at nodes
- Maintains arity compatibility
- Explores different functions

### Tree Examples

#### Symbolic Regression
```rust
let store = vec![
    (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul(), Op::div()]),
    (NodeType::Leaf, vec![Op::var(0), Op::var(1)]),
];

let codec = TreeCodec::single(5, store);
let problem = Regression::new(dataset, Loss::MSE, codec);
```

#### Classification
```rust
let store = vec![
    (NodeType::Root, vec![Op::sigmoid()]),
    (NodeType::Vertex, ops::math_ops()),
    (NodeType::Leaf, (0..4).map(Op::var).collect()),
];

let codec = TreeCodec::multi_root(3, 4, store);
```

## Graph Programming

### Graph Structure

Graphs represent computational networks with flexible connections between nodes. They can form complex topologies including cycles and shared computations.

### Node Types

**Input Nodes**
- Entry points for external data
- Zero incoming connections
- One or more outgoing connections

**Output Nodes**
- Final results of computation
- One or more incoming connections
- Zero outgoing connections

**Vertex Nodes**
- Internal computation nodes
- One or more incoming connections
- One or more outgoing connections

**Edge Nodes**
- Connection/weight nodes
- Exactly one incoming connection
- Exactly one outgoing connection

### Graph Codecs

**Directed Acyclic Graphs (DAGs)**
```rust
let codec = GraphCodec::directed(num_inputs, num_outputs, operations);
```

**Recurrent Graphs**
```rust
let codec = GraphCodec::recurrent(num_inputs, num_outputs, operations);
```

**Memory Graphs**
```rust
let codec = GraphCodec::memory(num_inputs, num_outputs, operations);
```

### Graph Operations

**Graph Crossover**
- Combines subgraphs from parents
- Maintains graph validity
- Preserves topological constraints

**Graph Mutation**
- Adds/removes nodes and connections
- Modifies graph topology
- Can enable/disable recurrence

**Operation Mutation**
- Changes node operations
- Maintains arity compatibility
- Explores different functions

### Graph Examples

#### XOR Problem
```rust
let values = vec![
    (NodeType::Input, vec![Op::var(0), Op::var(1)]),
    (NodeType::Edge, vec![Op::weight(), Op::identity()]),
    (NodeType::Vertex, ops::all_ops()),
    (NodeType::Output, vec![Op::sigmoid()]),
];

let codec = GraphCodec::directed(2, 1, values);
```

#### Neural Network Evolution
```rust
let values = vec![
    (NodeType::Input, vec![Op::var(0)]),
    (NodeType::Edge, vec![Op::weight()]),
    (NodeType::Vertex, vec![Op::sigmoid(), Op::tanh(), Op::relu()]),
    (NodeType::Output, vec![Op::linear()]),
];

let codec = GraphCodec::directed(1, 1, values);
```

## Advanced Features

### Constraints

Apply constraints to control program structure:

```rust
let codec = TreeCodec::single(5, operations)
    .constraint(|root| root.size() < 50)           // Limit tree size
    .constraint(|root| root.depth() < 8)           // Limit depth
    .constraint(|root| root.leaf_count() > 2);     // Ensure minimum leaves
```

### Custom Operations

Define your own operations:

```rust
let custom_op = Op::Fn("custom", Arity::Exact(2), Arc::new(|inputs| {
    let x = inputs[0];
    let y = inputs[1];
    x * x + y * y  // Custom function
}));
```

### Evaluation Strategies

**Tree Evaluation**
```rust
let result = tree.eval(&[1.0, 2.0, 3.0]);
```

**Graph Evaluation**
```rust
let mut evaluator = GraphEvaluator::new(&graph);
let result = evaluator.eval_mut(&[1.0, 2.0, 3.0]);
```

### Serialization

Save and load evolved programs:

```rust
// Save
let serialized = serde_json::to_string(&tree).unwrap();
std::fs::write("best_tree.json", serialized).unwrap();

// Load
let tree: Tree<Op<f32>> = serde_json::from_str(&content).unwrap();
```

## Best Practices

### Problem Selection

**Use Trees When:**
- You need interpretable expressions
- The problem has clear mathematical structure
- Performance is critical
- You want to understand the solution

**Use Graphs When:**
- You need complex computational patterns
- The problem requires memory or state
- You want maximum expressiveness
- Neural network evolution is the goal

### Configuration Tips

**Population Size**
- Start with 100-500 individuals
- Increase for complex problems
- Balance between exploration and computation cost

**Mutation Rates**
- Tree operations: 0.01-0.1
- Graph operations: 0.05-0.2
- Operation changes: 0.02-0.1

**Constraints**
- Always limit program size to prevent bloat
- Use depth constraints for trees
- Consider computational complexity

**Operations**
- Include only relevant operations
- Balance between expressiveness and search space
- Consider domain-specific operations

### Common Pitfalls

**Overfitting**
- Use validation sets
- Limit program complexity
- Apply regularization constraints

**Premature Convergence**
- Increase population diversity
- Adjust selection pressure
- Use multiple mutation operators

**Computational Bloat**
- Apply size constraints
- Use parsimony pressure
- Monitor program complexity

## Examples

### Complete Symbolic Regression
```rust
use radiate::*;

fn main() {
    // Generate dataset: y = 2x^2 + 3x + 1
    let dataset = generate_dataset();
    
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
    println!("Final score: {:?}", result.score());
}
```

### Neural Network Evolution
```rust
use radiate::*;

fn main() {
    // XOR dataset
    let dataset = DataSet::new(
        vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![1.0, 0.0], vec![0.0, 1.0]],
        vec![vec![0.0], vec![0.0], vec![1.0], vec![1.0]]
    );
    
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
```

## Performance Considerations

### Evaluation Speed
- **Trees**: Very fast, O(n) where n is number of nodes
- **Graphs**: Slower, depends on topology and evaluation strategy
- **Memory graphs**: Slowest due to state management

### Memory Usage
- **Trees**: Minimal memory overhead
- **Graphs**: Higher memory usage for complex topologies
- **Large populations**: Consider memory constraints

### Parallelization
- Use `Executor::worker_pool()` for parallel evaluation
- GP operations are naturally parallelizable
- Balance between cores and memory usage

## Integration with Radiate

### Engine Integration
GP programs integrate seamlessly with Radiate's genetic engine:

```rust
let engine = GeneticEngine::builder()
    .problem(gp_problem)
    .minimizing()
    .executor(Executor::worker_pool(8))
    .diversity(NeatDistance::new(1.0, 1.0, 3.0))
    .species_threshold(1.8)
    .build();
```

### Event System
Monitor GP evolution with events:

```rust
let engine = GeneticEngine::builder()
    .problem(problem)
    .subscribe(EventLogger::default())
    .subscribe(MetricsAggregator::new())
    .build();
```

### Diversity and Speciation
Apply diversity measures to GP populations:

```rust
let engine = GeneticEngine::builder()
    .problem(problem)
    .diversity(TreeDistance::new())
    .species_threshold(2.0)
    .build();
```

This comprehensive documentation provides a complete guide to using Genetic Programming in Radiate, from basic concepts to advanced techniques and best practices. -->

<!-- 
!!! warning ":construction: Under Construction :construction:"

    These docs are a work in progress and may not be complete or accurate. Please check back later for updates.

# Genetic Programming 

The `gp` feature provides the fundamental building blocks for [Genetic Programming](https://en.wikipedia.org/wiki/Genetic_programming) (GP). It includes **data structures and algorithms** for building and evolving **`Tree`s** and **`Graph`s**.

Genetic Programming (GP) is an evolutionary algorithm that **evolves programs** to solve problems. Programs are represented as **expression trees, decision trees, random forests, or neural networks** and evolved over generations.

**Typical Use Cases**

1. Symbolic Regression (finding equations that fit data)
2. Evolving Decision Trees (classification, optimization)
3. Neural Network Topology Evolution (similar to NEAT)
4. Evolving Graph-Based Programs

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
