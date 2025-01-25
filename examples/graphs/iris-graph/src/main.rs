use std::collections::HashSet;

use radiate::*;
use radiate_gp::*;

const MIN_SCORE: f32 = 0.001;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    random_provider::set_seed(1000);

    let mut dataset = load_iris_dataset();

    dataset.shuffle();
    dataset.standardize();

    let (train, test) = dataset.split(0.75);

    let graph_codex = GraphCodex::classification(4, 4);

    let regression = Regression::new(train, Loss::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .num_threads(10)
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.02, 0.05),
            GraphMutator::new(vec![
                NodeMutate::Edge(0.03, false),
                NodeMutate::Vertex(0.03, false),
            ]),
        ))
        .fitness_fn(move |genotype: Graph<Op<f32>>| {
            let mut reducer = GraphReducer::new(&genotype);
            regression.error(|input| reducer.reduce(input))
        })
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    display(&test, &result);
}

fn display(test_data: &DataSet, result: &EngineContext<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut classification_accuracy = 0.0;
    let mut total = 0.0;
    let mut reducer = GraphReducer::new(&result.best);
    for sample in test_data.iter() {
        let output = reducer.reduce(&sample.1);

        let mut max_idx = 0;
        for i in 0..output.len() {
            if output[i] > output[max_idx] {
                max_idx = i;
            }
        }

        let target = sample.2.iter().position(|&x| x == 1.0).unwrap();

        total += 1.0;
        if max_idx == target {
            classification_accuracy += 1.0;
        }

        println!("Target: {:?} :: Pred: {:?}", target, max_idx);
    }

    classification_accuracy /= total;

    println!("Accuracy: {:.2?}", classification_accuracy);
    println!("{:?}", result)
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
