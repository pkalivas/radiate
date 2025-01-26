use std::collections::HashSet;

use radiate::*;
use radiate_gp::*;

const MIN_SCORE: f32 = 0.001;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    random_provider::set_seed(1000);

    let (train, test) = load_iris_dataset().shuffle().standardize().split(0.75);
    let train_copy = train.clone();

    let graph_codex = GraphCodex::classification(4, 4);

    let engine = GeneticEngine::from_codex(graph_codex)
        .minimizing()
        .num_threads(10)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.02, 0.05),
            GraphMutator::new(vec![
                NodeMutate::Edge(0.03, false),
                NodeMutate::Vertex(0.03, false),
            ]),
        ))
        .fitness_fn(move |graph: Graph<Op<f32>>| {
            Loss::MSE.loss(&mut GraphEvaluator::new(&graph), &train_copy)
        })
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    display(&train, &test, &result);
}

fn display(
    train: &DataSet,
    test: &DataSet,
    result: &EngineContext<GraphChromosome<Op<f32>>, Graph<Op<f32>>>,
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
