use radiate::prelude::*;

fn main() {
    // --8<-- [start:eval]
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
    // --8<-- [end:eval]

    // --8<-- [start:variants]
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
    let graph: Graph<Op<f32>> = Graph::directed(2, 2, values.clone());

    // create a recurrent graph with 2 input nodes and 2 output nodes
    let graph: Graph<Op<f32>> = Graph::recurrent(2, 2, values.clone());

    // create a weighted directed graph with 2 input nodes and 2 output nodes
    let graph: Graph<Op<f32>> = Graph::weighted_directed(2, 2, values.clone());

    // create a weighted recurrent graph with 2 input nodes and 2 output nodes
    let graph: Graph<Op<f32>> = Graph::weighted_recurrent(2, 2, values.clone());

    // create an LSTM graph with 2 input nodes and 2 output nodes
    let graph: Graph<Op<f32>> = Graph::lstm(2, 2, values.clone());

    // create a GRU graph with 2 input nodes and 2 output nodes
    let graph: Graph<Op<f32>> = Graph::gru(2, 2, values);

    // Op graphs can be evaluated much like trees, but with the added complexity of connections.
    let inputs = vec![vec![1.0, 2.0]];
    let outputs = graph.eval(&inputs);
    // --8<-- [end:variants]

    // --8<-- [start:graphnode]
    // Create a new input node with value 42
    let node = GraphNode::new(0, NodeType::Input, 42);

    // Create a node with specific arity
    // This node will be invalid if it has a number of incoming connections other than 2
    let node_with_arity = GraphNode::with_arity(1, NodeType::Vertex, 42, Arity::Exact(2));
    // --8<-- [end:graphnode]

    // --8<-- [start:encode_decode]
    // Create a store for graph operations
    let store = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Edge, vec![Op::weight(), Op::identity()]), // both of these ops have an arity of 1
        (NodeType::Vertex, vec![Op::sub(), Op::div()]),
        (NodeType::Output, vec![Op::sigmoid(), Op::tanh()]),
    ];

    // Create a directed graph codec with 2 input nodes and 2 output nodes
    let codec = GraphCodec::directed(2, 2, store.clone());
    let genotype: Genotype<GraphChromosome<Op<f32>>> = codec.encode();
    let graph: Graph<Op<f32>> = codec.decode(&genotype);

    // Create a recurrent graph codec with 2 input nodes and 2 output nodes
    let recurrent_codec = GraphCodec::recurrent(2, 2, store);
    let recurrent_genotype: Genotype<GraphChromosome<Op<f32>>> = recurrent_codec.encode();
    let recurrent_graph: Graph<Op<f32>> = recurrent_codec.decode(&recurrent_genotype);
    // --8<-- [end:encode_decode]

    // --8<-- [start:graph_mutator]
    // Create a mutator that adds vertices and edges with a 10% chance for either
    let mutator = GraphMutator::new(0.1, 0.1);

    let mutator = GraphMutator::new(0.1, 0.1).allow_recurrent(false); // Disallow recurrent connections
    // --8<-- [end:graph_mutator]

    // --8<-- [start:graph_crossover]
    // Crossover with a 10% rate; 50% chance the less-fit parent takes a node from the more-fit parent
    let crossover = GraphCrossover::new(0.1, 0.5);
    // --8<-- [end:graph_crossover]
}
