use radiate::prelude::*;

// --8<-- [start:graph_regression]
const MIN_SCORE: f32 = 0.001;

fn main() {
    // Set the random seed for reproducibility
    random_provider::seed(1000);

    // Define our node set for the graph - the operations that can be used by each
    // node type in the graph during evolution
    let set = node_set! {
        Input => Op::var(0),
        Edge => Op::weight(),
        Vertex => vec![Op::sub(), Op::mul(), Op::sum()],
        Output => Op::sum()
    };

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(1, 1, set).unwrap())
        .fitness_fn(Regression::new(dataset(), Loss::MSE))
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1)
        ))
        .build();

    engine
        .iter()
        .logging()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display)
        .expect("No result from engine run");
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

fn dataset() -> impl Into<DataSet<f32>> {
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
// --8<-- [end:graph_regression]

#[allow(dead_code)]
fn datasets() {
    // --8<-- [start:datasets]
    // Built-in datasets come ready to use - here Fisher's iris: 4 features, 3 one-hot labels
    // `split(0.75)` puts the first 75% of the rows in `train` and the rest in `test`
    let (mut train, mut test) = DataSet::<f32>::iris().split(0.75);

    // Scale each feature column to mean 0 and standard deviation 1
    train.standardize();
    test.standardize();

    let (samples, n_features, n_labels) = train.shape();

    let set = node_set! {
        Input => Op::vars(0..n_features),
        Edge => Op::weight(),
        Vertex => vec![Op::add(), Op::mul(), Op::sigmoid(), Op::relu()],
        Output => Op::sigmoid()
    };

    let engine = GeneticEngine::builder()
        .codec(GraphCodec::directed(n_features, n_labels, set).unwrap())
        .fitness_fn(Regression::new(train, Loss::MSE))
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1),
        ))
        .build();
    // --8<-- [end:datasets]
}
