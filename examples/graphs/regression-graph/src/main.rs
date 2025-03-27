use std::collections::HashSet;

use radiate::*;
use radiate_gp::*;

const MIN_SCORE: f32 = 0.001;
const MAX_SECONDS: f64 = 5.0;

fn main() {
    random_provider::set_seed(1000);

    let values = vec![
        (NodeType::Input, vec![Op::var(0)]),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
        (NodeType::Output, vec![Op::linear()]),
    ];

    let graph_codex = GraphCodex::directed(1, 1, values);

    let problem = RegressionProblem::new(get_dataset(), Loss::MSE, graph_codex);

    let engine = GeneticEngine::from_problem(problem)
        .minimizing()
        // .distance(NeatDistance::new(1.8, 1.0, 1.0, 3.0))
        .num_threads(10)
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.07, 0.05),
            GraphMutator::new(0.1, 0.1).allow_recurrent(false),
        ))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    display(&result);
}

fn display(result: &EngineContext<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
    let mut total_ids = Vec::new();
    let mut unique_ids = HashSet::new();

    for pheno in result.population.iter() {
        for chromosome in pheno.genotype().iter() {
            for node in chromosome.iter() {
                total_ids.push(node.id());
                unique_ids.insert(node.id());
            }
        }
    }

    println!("Total Nodes: {}", total_ids.len());
    println!("Unique Nodes: {}", unique_ids.len());

    let mut evaluator = GraphEvaluator::new(&result.best);

    let data_set = get_dataset();
    let accuracy = Accuracy::new("reg", &data_set, Loss::MSE);
    let accuracy_result = accuracy.calc(|input| evaluator.eval_mut(input));

    println!("{:?}", result);
    println!("{:?}", accuracy_result);
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
