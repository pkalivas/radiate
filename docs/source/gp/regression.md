
# Regression

In machine learning its common to have a regression task. This is where you have a set of inputs and outputs, and you want to find a function that maps the inputs to the outputs. In Radiate, we can use genetic programming to evolve a `tree` or `graph` to do just that. The regression `problem` is a special type of `problem` that simplifies this process. It provides functionality to normalize/standarize/OHE the inputs and outputs, as well as calculate the fitness of a `genotype` based on how well it maps the inputs to the outputs.

The regression problem (fitness function) takes a set of inputs and outputs, and optionally a loss function. The default loss function is mean squared error (MSE), but other options include MAE (mean average error), CrossEntropy loss, and Diff (Difference - a simple difference between output and target).

Lets take a quick look at how we would put together a regression problem using a `tree` and a `graph`.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    rd.random.seed(518)

    def compute(x: float) -> float:
        return 4.0 * x**3 - 3.0 * x**2 + x


    inputs = []
    answers = []

    # Create a simple dataset of 20 points where the input is between -1 and 1
    # and the output is the result of the compute function above 
    input = -1.0
    for _ in range(-10, 10):
        input += 0.1
        inputs.append([input])
        answers.append([compute(input)])

    # Create a simple GraphCodec which takes one input and produces one output
    # The graph will have vertex nodes that can add, subtract, or multiply
    # The edges will have a weight operation and the output will be linear
    codec = rd.GraphCodec.directed(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    # All we have to do to create a regression problem is provide the features & targets for our 
    # dataset. Optionally, we can provide a loss function as well - the default is mean squared error (MSE).
    # The last argument is whether to use batch evaluation or not - the default is False. This has minimal impact on performance.
    fitness_func = rd.Regression(inputs, answers, loss="mse", batch=False)   

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_func,
        objective="min",   # Minimize the loss
        alters=[
            rd.GraphCrossover(0.5, 0.5),
            rd.OperationMutator(0.07, 0.05),
            rd.GraphMutator(0.1, 0.1, allow_recurrent=False), # True if evolving recurrent graphs is allowed
        ],
    )

    # Run the genetic engine with a score (error) limit of 0.001 or a maximum of 1000 generations
    result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(1000)], log=True)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    const MIN_SCORE: f32 = 0.001;

    fn main() {
        // Set the random seed for reproducibility
        random_provider::set_seed(1000);

        // Define our node store for the graph - the operations that can be used by each 
        // node type in the graph during evolution
        let store = vec![
            (NodeType::Input, vec![Op::var(0)]),
            (NodeType::Edge, vec![Op::weight()]),
            (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
            (NodeType::Output, vec![Op::linear()]),
        ];

        let engine = GeneticEngine::builder()
            .codec(GraphCodec::directed(1, 1, store))
            .fitness_fn(Regression::new(dataset(), Loss::MSE))
            .minimizing()
            .alter(alters!(
                GraphCrossover::new(0.5, 0.5),
                OperationMutator::new(0.07, 0.05),
                GraphMutator::new(0.1, 0.1).allow_recurrent(false)
            ))
            .build();

        engine
            .iter()
            .logging()
            .until_score(MIN_SCORE)
            .last()
            .inspect(display);
    }

    // A simple function to take the output of the engine and display the accuracy 
    // of the result on the dataset
    fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
        Accuracy::default()
            .named("Regression Graph")
            .on(&dataset().into())
            .loss(Loss::MSE)
            .eval(result.value())
            .inspect(|acc| {
                println!("{result:?}\n{acc:?}\n{}", result.metrics().dashboard());
            });
    }

    fn dataset() -> impl Into<DataSet> {
        let mut inputs = Vec::new();
        let mut answers = Vec::new();

        let mut input = -1.0;
        for _ in -10..10 {
            input += 0.1;
            inputs.push(vec![input]);
            answers.push(vec![compute(input)]);
        }

        (inputs, answers)
    }

    fn compute(x: f32) -> f32 {
        4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
    }
    ```


More robust examples can be found in the next section or in the [tree](https://github.com/pkalivas/radiate/tree/master/examples/trees) and [graph](https://github.com/pkalivas/radiate/tree/master/examples/graphs) examples in the git repository.

---
