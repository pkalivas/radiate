use radiate::*;
use radiate::{FactorKind, PgmDataSet, PgmLogLik, random_provider};
use std::vec;

const MIN_SCORE: f32 = 0.001;

fn sample_dataset(cards: &[usize], n: usize) -> PgmDataSet {
    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        let mut row = Vec::with_capacity(cards.len());
        for &k in cards {
            row.push(Some(random_provider::range(0..k)));
        }
        rows.push(row);
    }

    PgmDataSet::new(rows)
}

fn main() {
    random_provider::set_seed(40);

    let prob_dataset = sample_dataset([5, 4, 3, 2, 5, 8, 9].as_slice(), 100);

    // 3 discrete variables
    // X0 ∈ {0,1}, X1 ∈ {0,1,2}, X2 ∈ {0,1}
    let cards = vec![2, 3, 2];

    let data = PgmDataSet::new(vec![
        vec![Some(0), Some(0), Some(1)],
        vec![Some(1), Some(1), Some(0)],
        vec![Some(1), Some(2), Some(1)],
        vec![Some(0), Some(1), Some(0)],
    ]);

    let codec = PgmCodec::new(&cards, /*num_factors=*/ 4, /*max_scope=*/ 3);

    let g = codec.encode();
    let p = codec.decode(&g);
    let ll = PgmLogLik::new(data.clone()).loglik(&p);
    println!("raw loglik: {ll}");

    let fitness = PgmLogLik::new(data);

    // Fitness: minimize negative mean log-likelihood

    let engine = GeneticEngine::builder()
        .codec(codec)
        // If your builder expects a FitnessFunction<Phenotype, f32>
        // and phenotype is PgmChromosome, this just works:
        .fitness_fn(fitness)
        .minimizing()
        .alter(alters!(
            // replace with your actual mutators for this chromosome
            // (examples):
            // PgmParamMutator::new(0.3, 0.1),
            // PgmScopeMutator::new(0.05),
        ))
        .build();

    engine
        .iter()
        .logging()
        .take(200)
        .last()
        .inspect(|generation| {
            println!("{}", generation.metrics().dashboard());
            println!("Best score: {}", generation.score()[0]);
            println!("Best model: {:#?}", generation.value());
        });

    // let vars = (0..prob_dataset.num_vars())
    //     .map(|i| Op::category("var_input", i, random_provider::range(2..10)))
    //     .collect::<Vec<_>>();

    // let store = vec![(NodeType::Input, vars), (NodeType::Output, vec![Op::sum()])];

    // let codec = PgmCodec::new(
    //     prob_dataset.num_vars(),
    //     3,
    //     vec![FactorType::Logp, FactorType::Gauss1],
    //     store,
    // );

    // let encoded = codec.encode();
    // let decoded = codec.decode(&encoded);

    // let loglik = PgmLogLik::new(prob_dataset.clone(), prob_dataset.num_vars());
    // let ll = loglik.evaluate(decoded);
    // println!("Log Likelihood of decoded PGM on dataset: {}", ll);

    // let engine = GeneticEngine::builder()
    //     .codec(codec)
    //     .fitness_fn(loglik)
    //     .minimizing()
    //     .alter(alters!(
    //         OperationMutator::new(0.3, 0.05),
    //         GraphCrossover::new(0.7, 0.1),
    //         // GraphMutator::new(0.2, 0.05)
    //     ))
    //     .build();

    // engine.iter().logging().take(150).last().inspect(|result| {
    //     println!("{}", result.metrics().dashboard());
    //     println!("Best PGM: {:?}", result.value());
    // });

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
