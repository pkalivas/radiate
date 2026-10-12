use radiate::*;
use std::collections::BTreeSet;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 5.0;
const MIN_TREE_DEPTH: usize = 3;

fn main() {
    random_provider::seed(1000);

    let (train, test) = load_iris_dataset().shuffle().standardize().split(0.75);

    let store = vec![
        (NodeType::Root, vec![Op::sigmoid()]),
        (NodeType::Vertex, ops::all_ops()),
        (NodeType::Leaf, (0..4).map(Op::var).collect()),
    ];

    let num_trees = train.shape().1;

    let engine = GeneticEngine::builder()
        .codec(
            TreeCodec::multi_root(MIN_TREE_DEPTH, num_trees, store)
                .constraint(|node| node.size() < 40),
        )
        .fitness_fn(Regression::new(train.clone(), Loss::MSE))
        .minimizing()
        .crossover(TreeCrossover::new(0.5))
        .mutator(OperationMutator::new(0.03, 0.02))
        .build();

    engine
        .iter()
        .logging()
        .until(|epoch| epoch.score().as_f32() < MIN_SCORE || epoch.seconds() > MAX_SECONDS)
        .last()
        .inspect(|ctx| display(&train, &test, ctx))
        .unwrap();
}

fn display(
    train: &DataSet<f32>,
    test: &DataSet<f32>,
    result: &Generation<TreeChromosome<Op<f32>>, Vec<Tree<Op<f32>>>>,
) {
    println!("{}", result.metrics().dashboard());
    let mut best = result.value().clone();

    let train_acc = Accuracy::default()
        .named("train")
        .on(train)
        .loss(Loss::MSE)
        .calc(&mut best);
    let test_acc = Accuracy::default()
        .named("test")
        .on(test)
        .loss(Loss::MSE)
        .calc(&mut best);

    println!("{:?}", result);
    println!("{:?}", train_acc);
    println!("{:?}", test_acc);
}

fn load_iris_dataset() -> DataSet<f32> {
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

    let unique_labels = labels.iter().collect::<BTreeSet<&String>>();

    let mut new_labels = Vec::new();
    for label in labels.iter() {
        let mut new_label = vec![0.0; unique_labels.len()];
        let index = unique_labels.iter().position(|x| *x == label).unwrap();
        new_label[index] = 1.0;
        new_labels.push(new_label);
    }

    DataSet::new(features, new_labels)
}
