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

// Generation {
//   metrics: MetricSet {
//   carryover: 0.397  diversity: 0.782  unique_members: 75  unique_scores: 45  lifetime_unique: 9705  iter_time(mean): 280.438µs
// },
//   value: Graph {
//   [0  ] [440    ] "Inp" :: Zero  Var: X0(0)   V:true  R:false 0  8  < [],
//   [1  ] [443    ] "Out" :: Any   Fn: linear   V:true  R:false 7  0  < [0, 2, 3, 5, 8, 12, 14],
//   [2  ] [610    ] "Ver" :: Any   Fn: linear   V:true  R:false 2  2  < [0, 6],
//   [3  ] [694    ] "Ver" :: Any   Fn: linear   V:true  R:false 1  3  < [0],
//   [4  ] [940    ] "Edg" :: 1     w(-1.36)     V:true  R:false 1  1  < [2],
//   [5  ] [992    ] "Ver" :: 2     Fn: mul      V:true  R:false 2  1  < [0, 4],
//   [6  ] [1666   ] "Ver" :: Any   Fn: linear   V:true  R:false 2  1  < [0, 9],
//   [7  ] [1708   ] "Edg" :: 1     w(-1.22)     V:true  R:false 1  1  < [0],
//   [8  ] [2846   ] "Ver" :: Any   Fn: linear   V:true  R:false 3  2  < [7, 13, 15],
//   [9  ] [3098   ] "Ver" :: 2     Fn: mul      V:true  R:false 2  2  < [8, 11],
//   [10 ] [3118   ] "Edg" :: 1     w(-0.08)     V:true  R:false 1  1  < [3],
//   [11 ] [3208   ] "Ver" :: Any   Fn: linear   V:true  R:false 1  1  < [0],
//   [12 ] [3266   ] "Edg" :: 1     w(-0.89)     V:true  R:false 1  1  < [9],
//   [13 ] [3338   ] "Edg" :: 1     w(-1.14)     V:true  R:false 1  1  < [0],
//   [14 ] [3510   ] "Ver" :: Any   Fn: linear   V:true  R:false 1  1  < [10],
//   [15 ] [3834   ] "Edg" :: 1     w(-0.50)     V:true  R:false 1  1  < [3],
// },
//   score: [0.0005406429],
//   index: 205,
//   size: 100,
//   duration: 57.489753ms,
//   objective: Single(Minimize),
// }
// Regression Accuracy - "reg" {
// 	N: 20
// 	Accuracy: 99.81%
// 	R² Score: 0.99987
// 	RMSE: 0.02325
// 	Loss (MSE): 0.00054
// }
// [25 metrics, 9498 updates]
// [metrics]  carryover: 0.397  diversity: 0.782  unique_members: 75  unique_scores: 45  lifetime_unique: 9705  iter_time(mean): 280.438µs

// == Generation ==
// Name                     | Type   | Mean       | Min        | Max        | N      | Total        | StdDev     | Skew       | Kurt       | Entr
// -------------------------------------------------------------------------------------------------------------------------------------------------
// age                      | value  | 1.291      | 0.000      | 2.950      | 205    | -            | 0.496      | 0.000      | 0.000      | -
// carryover_rate           | value  | 0.397      | 0.000      | 0.657      | 205    | -            | 0.098      | 0.000      | 0.000      | -
// diversity_ratio          | value  | 0.782      | 0.620      | 0.920      | 205    | -            | 0.058      | 0.000      | 0.000      | -
// evaluation_count         | value  | 47.610     | 23.000     | 129.000    | 205    | -            | 10.620     | 2.071      | 19.495     | -
// genome_size              | value  | 8.399      | 2.060      | 16.080     | 205    | -            | 3.568      | 0.712      | 3.407      | -
// graph_crossover          | value  | 164.971    | 30.000     | 397.000    | 205    | -            | 76.586     | 0.774      | 3.309      | -
// graph_crossover          | time   | 24.769µs   | 9.042µs    | 68.709µs   | 205    | 5.078ms      | 7.111µs    | -          | -          | -
// graph_mutator            | value  | 26.180     | 6.000      | 56.000     | 205    | -            | 9.990      | 0.498      | 3.157      | -
// graph_mutator            | time   | 23.591µs   | 5.542µs    | 77.708µs   | 205    | 4.836ms      | 10.263µs   | -          | -          | -
// graph_mutator(_ivld)     | value  | 2.443      | 1.000      | 7.000      | 174    | -            | 1.358      | 2.527      | 14.924     | -
// new_children             | value  | 47.341     | 23.000     | 74.000     | 205    | -            | 9.146      | -0.109     | 3.249      | -
// op_mut_const             | value  | 12.492     | 1.000      | 40.000     | 193    | -            | 9.376      | 1.141      | 3.595      | -
// op_new_inst              | value  | 9.979      | 1.000      | 25.000     | 195    | -            | 5.144      | 0.719      | 3.409      | -
// operation_mutator        | value  | 46.307     | 5.000      | 103.000    | 205    | -            | 21.251     | 0.788      | 3.324      | -
// operation_mutator        | time   | 20.570µs   | 6.875µs    | 57.208µs   | 205    | 4.217ms      | 6.278µs    | -          | -          | -
// roulette_selector        | value  | 80.000     | 80.000     | 80.000     | 205    | -            | 0.000      | 0.000      | 0.000      | -
// roulette_selector        | time   | 34.776µs   | 16.542µs   | 77.208µs   | 205    | 7.129ms      | 10.425µs   | -          | -          | -
// score_volatility         | value  | 1.419      | 0.057      | 5.828      | 205    | -            | 1.061      | 22.936     | 391.994    | -
// scores                   | dist   | 0.966      | 0.001      | 37.237     | 100    | -            | 1.201      | 26.530     | 644.691    | 1.3
//                            ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▂▃█
// survivor_count           | value  | 30.863     | 0.000      | 44.000     | 205    | -            | 7.224      | -0.270     | 3.708      | -
// tournament_selector      | value  | 20.000     | 20.000     | 20.000     | 205    | -            | 0.000      | 0.000      | 0.000      | -
// tournament_selector      | time   | 8.891µs    | 3.875µs    | 39.750µs   | 205    | 1.823ms      | 3.468µs    | -          | -          | -
// unique_members           | value  | 78.205     | 62.000     | 92.000     | 205    | -            | 5.840      | -0.191     | 2.965      | -
// unique_scores            | value  | 31.385     | 3.000      | 68.000     | 205    | -            | 16.001     | 0.398      | 2.678      | -

// == Lifetime ==
// Name                     | Type   | Mean       | Min        | Max        | N      | Total        | StdDev     | Skew       | Kurt       | Entr
// -------------------------------------------------------------------------------------------------------------------------------------------------
// lifetime_unique          | value  | 4452.917   | 74.000     | 9705.000   | 205    | -            | 2815.259   | 0.159      | 1.891      | -
// time                     | time   | 280.438µs  | 95.375µs   | 744.125µs  | 205    | 57.490ms     | 82.874µs   | -          | -          | -

// == Step Timings ==
// Name                     | Type   | Mean       | Min        | Max        | N      | Total        | StdDev     | Skew       | Kurt       | Entr
// -------------------------------------------------------------------------------------------------------------------------------------------------
// audit_step               | time   | 26.683µs   | 15.958µs   | 130.584µs  | 205    | 5.470ms      | 14.124µs   | -          | -          | -
// evaluate_step            | time   | 123.579µs  | 21.125µs   | 224.083µs  | 205    | 25.334ms     | 46.648µs   | -          | -          | -
// filter_step              | time   | 5.411µs    | 1.333µs    | 15.333µs   | 205    | 1.109ms      | 1.757µs    | -          | -          | -
// recombine_step           | time   | 119.986µs  | 50.416µs   | 259.916µs  | 205    | 24.597ms     | 33.139µs   | -          | -          | -
