use std::vec;

use radiate::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    random_provider::set_seed(4);

    let var_one = Domain::Discrete(2);
    let var_two = Domain::Discrete(2);
    let var_three = Domain::Discrete(3);

    let codec = PgmCodec::new(
        vec![
            ("VarOne".to_string(), var_one.clone()),
            ("VarTwo".to_string(), var_two.clone()),
            ("VarThree".to_string(), var_three.clone()),
        ],
        2,
        3,
    );

    let encoded = codec.encode();
    println!("Encoded Genotype: {:?}", encoded);
    let decoded = codec.decode(&encoded);
    // println!("Decoded Phenotype: {:?}", decoded);

    let prob_dataset = ProbDataset::new(vec![
        vec![Some(2), Some(0), Some(0)],
        vec![Some(1), Some(0), Some(0)],
        vec![Some(0), Some(1), Some(2)],
    ]);

    let temp = LogInfoEval;

    let ll = temp.log_likelihood(&decoded, &prob_dataset);
    println!("Log Likelihood of decoded PGM on dataset: {}", ll);

    // let seed_one = TreeNode::from(Op::sigmoid())
    //     .attach(Op::constant(1.0))
    //     .attach(Op::var(0));
    // let seed_two = TreeNode::from(Op::linear()).attach(Op::var(0));
    // let seeds = vec![seed_one, seed_two];

    // let pgm = Op::softmax_argmax(seeds);

    // let store = vec![
    //     (
    //         NodeType::Vertex,
    //         vec![Op::add(), Op::sub(), Op::mul(), Op::linear(), pgm],
    //     ),
    //     (NodeType::Leaf, vec![Op::var(0)]),
    // ];

    // let tree_codec = TreeCodec::single(3, store).constraint(|root| root.size() < 30);
    // let problem = Regression::new(get_dataset(), Loss::MSE);

    // let engine = GeneticEngine::builder()
    //     .codec(tree_codec)
    //     .fitness_fn(problem)
    //     .minimizing()
    //     .mutator(HoistMutator::new(0.01))
    //     .crossover(TreeCrossover::new(0.7))
    //     .build();

    // engine
    //     .iter()
    //     .logging()
    //     .until_score(MIN_SCORE)
    //     .last()
    //     .inspect(display);
}

fn display(result: &Generation<TreeChromosome<Op<f32>>, Tree<Op<f32>>>) {
    Accuracy::default()
        .named("Regression Tree")
        .on(&get_dataset())
        .loss(Loss::MSE)
        .eval(result.value())
        .inspect(|acc| {
            println!("{}", result.metrics().dashboard());
            println!("Best Tree: {}", result.value().format());
            println!("{:?}", acc);
        });
}

fn get_dataset() -> DataSet {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compute(input)]);
    }

    DataSet::new(inputs, answers)
}

fn compute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}
