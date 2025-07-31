use radiate::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    random_provider::set_seed(1000);

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

fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut evaluator = GraphEvaluator::new(result.value());

    let data_set = dataset().into();
    let accuracy = Accuracy::new("reg", &data_set, Loss::MSE);
    let accuracy_result = accuracy.calc(|input| evaluator.eval_mut(input));

    println!("{:?}", result);
    println!("{:?}", accuracy_result);
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

// .diversity(NeatDistance::new(0.1, 0.1, 0.5))

// use radiate::prelude::*;

// fn main() {
//     let store = vec![
//         (NodeType::Input, vec![Op::var(0), Op::var(1)]),
//         (NodeType::Edge, vec![Op::weight()]),
//         (NodeType::Vertex, ops::all_ops()),
//         (NodeType::Output, vec![Op::linear()]),
//     ];

//     let architecture_novelty = GraphArchitectureNovelty;
//     let architecture_search =
//         NoveltySearch::new(architecture_novelty, 20, 0.1).with_max_archive_size(1000);

//     let engine = GeneticEngine::builder()
//         .codec(GraphCodec::directed(2, 1, store.clone()))
//         .fitness_fn(architecture_search.clone())
//         .replace_strategy(GraphReplacement)
//         .alter(alters![
//             GraphCrossover::new(0.5, 0.5),
//             OperationMutator::new(0.1, 0.1),
//             GraphMutator::new(0.1, 0.1).allow_recurrent(false)
//         ])
//         .build();

//     let result = engine.iter().logging().take(500).last().unwrap();

//     analyze_population_diversity(&result, &architecture_search);
// }

// fn analyze_population_diversity(
//     result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>,
//     architecture_search: &NoveltySearch<Graph<Op<f32>>, GraphArchitectureNovelty>,
// ) {
//     let individuals = result
//         .population()
//         .iter()
//         .map(|pheno| Graph::new(pheno.genotype().get(0).unwrap().genes().to_vec()))
//         .collect::<Vec<Graph<Op<f32>>>>();

//     let architecture_scores: Vec<f32> = individuals
//         .iter()
//         .map(|ind| architecture_search.evaluate(ind))
//         .collect();

//     let avg_architecture =
//         architecture_scores.iter().sum::<f32>() / architecture_scores.len() as f32;

//     println!(
//         "Archive size: {}",
//         architecture_search.archive.read().unwrap().len()
//     );
//     println!("Average architecture novelty: {:.4}", avg_architecture);

//     for (i, individual) in individuals.iter().take(3).enumerate() {
//         let architecture_score = architecture_search.evaluate(individual);

//         println!("Graph {}: Architecture={:.4}", i + 1, architecture_score);
//         println!(
//             "Score: {:.4}",
//             result.population()[i].score().unwrap().as_f32()
//         );
//         println!("Structure: {:?}", individual);
//     }
// }
