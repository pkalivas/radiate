use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;

fn main() {
    random_provider::set_seed(567123);

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
        // .diversity(NeatDistance::new(0.1, 0.1, 0.3))
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
    let accuracy_result = Accuracy::new("reg")
        .on(&dataset().into())
        .loss(Loss::MSE)
        .calc(&mut evaluator);

    println!("{result:?}\n{accuracy_result:?}");
    println!("{}", result.metrics());
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

// [25 metrics, 9498 updates]
// [metrics]  carryover: 0.397  diversity: 0.782  unique_members: 75  unique_scores: 45  lifetime_unique: 9705  iter_time(mean): 360.830µs

// == Generation ==
// Name                     | Type   | Mean       | Min        | Max        | N      | Total        | StdDev     | Skew       | Kurt       | Entr
// -------------------------------------------------------------------------------------------------------------------------------------------------
// age                      | value  | 1.291      | 0.000      | 2.950      | 205    | -            | 0.496      | 0.000      | 0.000      | -
// carryover_rate           | value  | 0.397      | 0.000      | 0.657      | 205    | -            | 0.098      | 0.000      | 0.000      | -
// diversity_ratio          | value  | 0.782      | 0.620      | 0.920      | 205    | -            | 0.058      | 0.000      | 0.000      | -
// evaluation_count         | value  | 47.610     | 23.000     | 129.000    | 205    | -            | 10.620     | 2.071      | 19.495     | -
// genome_size              | value  | 8.399      | 2.060      | 16.080     | 205    | -            | 3.568      | 0.712      | 3.407      | -
// graph_crossover          | value  | 164.971    | 30.000     | 397.000    | 205    | -            | 76.586     | 0.774      | 3.309      | -
// graph_crossover          | time   | 30.738µs   | 13.584µs   | 86.000µs   | 205    | 6.301ms      | 8.445µs    | -          | -          | -
// graph_mutator            | value  | 26.180     | 6.000      | 56.000     | 205    | -            | 9.990      | 0.498      | 3.157      | -
// graph_mutator            | time   | 31.211µs   | 10.000µs   | 216.125µs  | 205    | 6.398ms      | 19.351µs   | -          | -          | -
// graph_mutator(_ivld)     | value  | 2.443      | 1.000      | 7.000      | 174    | -            | 1.358      | 2.527      | 14.924     | -
// new_children             | value  | 47.341     | 23.000     | 74.000     | 205    | -            | 9.146      | -0.109     | 3.249      | -
// op_mut_const             | value  | 12.492     | 1.000      | 40.000     | 193    | -            | 9.376      | 1.141      | 3.595      | -
// op_new_inst              | value  | 9.979      | 1.000      | 25.000     | 195    | -            | 5.144      | 0.719      | 3.409      | -
// operation_mutator        | value  | 46.307     | 5.000      | 103.000    | 205    | -            | 21.251     | 0.788      | 3.324      | -
// operation_mutator        | time   | 25.528µs   | 9.959µs    | 59.375µs   | 205    | 5.233ms      | 6.364µs    | -          | -          | -
// roulette_selector        | value  | 80.000     | 80.000     | 80.000     | 205    | -            | 0.000      | 0.000      | 0.000      | -
// roulette_selector        | time   | 47.644µs   | 22.666µs   | 876.917µs  | 205    | 9.767ms      | 60.523µs   | -          | -          | -
// score_volatility         | value  | 1.419      | 0.057      | 5.828      | 205    | -            | 1.061      | 22.936     | 391.994    | -
// scores                   | dist   | 0.966      | 0.001      | 37.237     | 100    | -            | 1.201      | 26.530     | 644.691    | 1.3
//                            ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▂▃█
// survivor_count           | value  | 30.863     | 0.000      | 44.000     | 205    | -            | 7.224      | -0.270     | 3.708      | -
// tournament_selector      | value  | 20.000     | 20.000     | 20.000     | 205    | -            | 0.000      | 0.000      | 0.000      | -
// tournament_selector      | time   | 11.158µs   | 5.333µs    | 43.916µs   | 205    | 2.287ms      | 4.099µs    | -          | -          | -
// unique_members           | value  | 78.205     | 62.000     | 92.000     | 205    | -            | 5.840      | -0.191     | 2.965      | -
// unique_scores            | value  | 31.385     | 3.000      | 68.000     | 205    | -            | 16.001     | 0.398      | 2.678      | -

// == Lifetime ==
// Name                     | Type   | Mean       | Min        | Max        | N      | Total        | StdDev     | Skew       | Kurt       | Entr
// -------------------------------------------------------------------------------------------------------------------------------------------------
// lifetime_unique          | value  | 4452.917   | 74.000     | 9705.000   | 205    | -            | 2815.259   | 0.159      | 1.891      | -
// time                     | time   | 360.830µs  | 143.209µs  | 1.436ms    | 205    | 73.970ms     | 146.882µs  | -          | -          | -

// == Step Timings ==
// Name                     | Type   | Mean       | Min        | Max        | N      | Total        | StdDev     | Skew       | Kurt       | Entr
// -------------------------------------------------------------------------------------------------------------------------------------------------
// audit_step               | time   | 33.954µs   | 17.167µs   | 148.291µs  | 205    | 6.961ms      | 18.075µs   | -          | -          | -
// evaluate_step            | time   | 158.199µs  | 45.250µs   | 977.750µs  | 205    | 32.431ms     | 88.636µs   | -          | -          | -
// filter_step              | time   | 6.642µs    | 2.875µs    | 17.083µs   | 205    | 1.362ms      | 1.924µs    | -          | -          | -
// recombine_step           | time   | 155.989µs  | 73.708µs   | 1.071ms    | 205    | 31.978ms     | 78.575µs   | -          | -          | -
