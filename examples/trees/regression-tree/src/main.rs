use std::vec;

use radiate::{
    pgm::{PgmDataSet, PgmLogLik},
    *,
};

const MIN_SCORE: f32 = 0.001;

pub fn pgm_store_both(num_vars: usize, num_factors: usize) -> NodeStore<PgmOp<f32>> {
    let vars = (0..num_vars)
        .map(|i| {
            PgmOp::Variable(
                "var_input",
                i,
                Some(random_provider::range(2..num_factors + 2)),
            )
        })
        .collect::<Vec<PgmOp<f32>>>();

    let st = vec![
        (NodeType::Input, vars.clone()),
        (
            NodeType::Vertex,
            vec![PgmOp::logprob(), PgmOp::gauss1(), PgmOp::gauss_lin2()],
        ),
        (NodeType::Output, vec![PgmOp::Op(Op::sum())]),
    ];

    NodeStore::from(st)
}

fn main() {
    random_provider::set_seed(40);

    let codec = PgmCodec::<f32>::new(4, 2, pgm_store_both(4, 2));

    let encoded = codec.encode();
    let decoded = codec.decode(&encoded);

    let inputs = vec![
        vec![Some(2), Some(0), Some(0), Some(1)],
        vec![Some(3), Some(0), Some(0), Some(2)],
        vec![Some(0), Some(4), Some(2), Some(3)],
    ];

    let prob_dataset = PgmDataSet::new(inputs);
    let loglik = PgmLogLik::new(prob_dataset.clone(), 4);
    let ll = loglik.evaluate(decoded);
    println!("Log Likelihood of decoded PGM on dataset: {}", ll);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(loglik)
        .minimizing()
        .alter(alters!(
            OperationMutator::new(0.3, 0.05),
            GraphCrossover::new(0.7, 0.1)
        ))
        .build();

    engine.iter().logging().take(150).last().inspect(|result| {
        println!("{}", result.metrics().dashboard());
        println!("Best PGM: {:?}", result.value());
    });

    // let encoded = codec.encode();
    // println!("Encoded Genotype: {:?}", encoded);
    // let decoded = codec.decode(&encoded);
    // // println!("Decoded Phenotype: {:?}", decoded);

    // let prob_dataset = ProbDataset::new(vec![
    //     vec![Some(2), Some(0), Some(0)],
    //     vec![Some(1), Some(0), Some(0)],
    //     vec![Some(0), Some(1), Some(2)],
    // ]);

    // let temp = LogInfoEval;

    // let ll = temp.log_likelihood(&decoded, &prob_dataset);
    // println!("Log Likelihood of decoded PGM on dataset: {}", ll);

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

fn get_dataset() -> DataSet<f32> {
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
