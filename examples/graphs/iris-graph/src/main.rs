use std::collections::HashSet;

use radiate::*;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    random_provider::set_seed(1000);

    let (train, test) = load_iris_dataset().shuffle().standardize().split(0.75);

    let ops = ops::all_ops();
    let edges = vec![Op::identity(), Op::weight()];
    let outputs = vec![Op::sigmoid()];

    let store = vec![
        (NodeType::Input, (0..4).map(Op::var).collect()),
        (NodeType::Edge, edges.clone()),
        (NodeType::Vertex, ops.clone()),
        (NodeType::Output, outputs.clone()),
    ];

    let codex = GraphCodex::directed(4, 4, store);
    let regression = Regression::new(train.clone(), Loss::MSE, codex);

    let engine = GeneticEngine::builder()
        .problem(regression)
        .minimizing()
        .num_threads(10)
        .offspring_fraction(0.92)
        .replace_strategy(GraphReplacement)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.02, 0.05),
            GraphMutator::new(0.008, 0.002)
        ))
        .build();

    engine
        .iter()
        .take_while(|epoch| epoch.score.as_f32() > MIN_SCORE && epoch.seconds() < MAX_SECONDS)
        .inspect(|ctx| log_ctx!(ctx))
        .last()
        .inspect(|ctx| display(&train, &test, ctx));
}

fn display(
    train: &DataSet,
    test: &DataSet,
    result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>,
) {
    let mut reducer = GraphEvaluator::new(&result.best);

    let train_acc = Accuracy::new("train", &train, Loss::MSE);
    let test_acc = Accuracy::new("test", &test, Loss::MSE);

    let train_acc_result = train_acc.calc(|input| reducer.eval_mut(input));
    let test_acc_result = test_acc.calc(|input| reducer.eval_mut(input));

    println!("{:?}", result);
    println!("{:?}", train_acc_result);
    println!("{:?}", test_acc_result);
}

fn load_iris_dataset() -> DataSet {
    let file = include_str!("../iris.csv")
        .split("\n")
        .collect::<Vec<&str>>();

    let mut features = Vec::new();
    let mut labels = Vec::new();

    for line in file.iter() {
        let row = line
            .split(",")
            .map(|value| value.to_string())
            .collect::<Vec<String>>();

        let row_features = row
            .iter()
            .take(4)
            .map(|value| value.parse::<f32>().expect("Failed to parse value"))
            .collect::<Vec<f32>>();

        let row_labels = row.iter().last().unwrap().clone();

        features.push(row_features);
        labels.push(row_labels);
    }

    let unique_labels = labels.iter().collect::<HashSet<&String>>();

    let mut new_labels = Vec::new();
    for label in labels.iter() {
        let mut new_label = vec![0.0; unique_labels.len()];
        let index = unique_labels.iter().position(|x| *x == label).unwrap();
        new_label[index] = 1.0;
        new_labels.push(new_label);
    }

    DataSet::new(features, new_labels)
}
