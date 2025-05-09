
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

### Codex

The `GraphCodex` is much the same as other codexes gone over previously. It's encode function will produce a Genotype with a single `GraphChromosome` representing one graph, while the decode function will take a Genotype and produce a single `Graph`. The graph codex can be created similar to how a graph is created. It requires a set of values that a `GraphNode` can take the type of graph that is desired. 

A codex for a directed graph with 2 inputs and 1 output might look like this:
```rust
let values = vec![
    (NodeType::Input, vec![Op::var(0), Op::var(1)]),
    (NodeType::Output, vec![Op::sigmoid()]),
];

let codex = GraphCodex::directed(2, 1, values);
```

while a codex for a directed cyclic graph with 2 inputs and 2 outputs might look like this:
```rust
let values = vec![
    (NodeType::Input, vec![Op::var(0), Op::var(1)]),
    (NodeType::Edge, vec![Op::weight(), Op::identity()]),
    (NodeType::Vertex, ops::all_ops()),
    (NodeType::Output, vec![Op::sigmoid(), Op::tanh()]),
];

let codex = GraphCodex::recurrent(2, 2, values);
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
* `NodeCrossover` - Crossover two `Node`s by swapping their internal properties.
