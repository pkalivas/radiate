use radiate::*;
use std::collections::HashSet;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    random_provider::set_seed(1000);

    let (train, test) = load_iris_dataset().shuffle().standardize().split(0.75);

    let store = vec![
        (NodeType::Root, vec![Op::sigmoid()]),
        (NodeType::Vertex, ops::math_ops()),
        (NodeType::Leaf, (0..4).map(Op::var).collect()),
    ];

    let codec = TreeCodec::multi_root(3, 4, store).constraint(|node| node.size() < 40);
    let regression = Regression::new(train.clone(), Loss::MSE);

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(regression)
        .minimizing()
        .parallel()
        .crossover(TreeCrossover::new(0.5))
        .mutator(OperationMutator::new(0.03, 0.02))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index(), ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    display(&train, &test, &result);
}

fn display(
    train: &DataSet,
    test: &DataSet,
    result: &Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>,
) {
    let mut best = result.value().clone();

    let train_acc = Accuracy::new("train")
        .on(train)
        .loss(Loss::MSE)
        .calc(&mut best);
    let test_acc = Accuracy::new("test")
        .on(test)
        .loss(Loss::MSE)
        .calc(&mut best);

    println!("{:?}", result);
    println!("{:?}", train_acc);
    println!("{:?}", test_acc);
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
