use radiate::prelude::*;

// --8<-- [start:minsum]
fn minsum() {
    const MIN_SCORE: i32 = 0;

    let mut engine = GeneticEngine::builder()
        .codec(IntCodec::vector(10, 0..100))
        .minimizing()
        .offspring_selector(EliteSelector::new())
        .mutator(SwapMutator::new(0.05))
        .crossover(UniformCrossover::new(0.5))
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    let result = engine.run(|epoch| {
        println!("[ {:?} ]: {:?}", epoch.index(), epoch.value());
        epoch.score().as_i32() == MIN_SCORE
    });

    println!("{:?}", result);
}
// --8<-- [end:minsum]

// --8<-- [start:nqueens]
const N_QUEENS: usize = 45;

fn nqueens() {
    random_provider::seed(12345);

    let engine = GeneticEngine::builder()
        .codec(IntChromosome::from((N_QUEENS, 0..N_QUEENS as i8)))
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4.0))
        .crossover(MultiPointCrossover::new(0.75, 2))
        .mutator(UniformMutator::new(0.05))
        .fitness_fn(|queens: Vec<i8>| {
            let mut score = 0;

            for i in 0..N_QUEENS {
                for j in (i + 1)..N_QUEENS {
                    if queens[i] == queens[j] {
                        score += 1;
                    }
                    if (i as i8 - j as i8).abs() == (queens[i] - queens[j]).abs() {
                        score += 1;
                    }
                }
            }

            score
        })
        .build();

    let result = engine.iter().logging().until_score(0).last().unwrap();

    println!("Best Score: {:?}", result);
    println!("\nResult Queens Board ({:.3?}):", result.time());

    let board = &result.value();
    for i in 0..N_QUEENS {
        for j in 0..N_QUEENS {
            if board[j] == i as i8 {
                print!("Q ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
}
// --8<-- [end:nqueens]

// --8<-- [start:rastrigin]
fn rastrigin() {
    const MIN_SCORE: f32 = 0.00;
    const MAX_SECONDS: f64 = 1.0;
    const A: f32 = 10.0;
    const RANGE: f32 = 5.12;
    const N_GENES: usize = 2;

    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(N_GENES, -RANGE..RANGE))
        .minimizing()
        .population_size(500)
        .alter(alters!(
            UniformCrossover::new(0.5),
            ArithmeticMutator::new(0.01)
        ))
        .fitness_fn(move |genotype: Vec<f32>| {
            let mut value = A * N_GENES as f32;
            for i in 0..N_GENES {
                value += genotype[i].powi(2) - A * (2.0 * std::f32::consts::PI * genotype[i]).cos();
            }

            value
        })
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index(), ctx.score().as_f32());
        ctx.score().as_f32() <= MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    println!("{:?}", result);
}
// --8<-- [end:rastrigin]

// --8<-- [start:dtlz1]
fn dtlz1() {
    const VARIABLES: usize = 4;
    const OBJECTIVES: usize = 3;
    const K: usize = VARIABLES - OBJECTIVES + 1;

    fn dtlz_1(values: &[f32]) -> Vec<f32> {
        let mut g = 0.0;
        for i in VARIABLES - K..VARIABLES {
            g +=
                (values[i] - 0.5).powi(2) - (20.0 * std::f32::consts::PI * (values[i] - 0.5)).cos();
        }

        g = 100.0 * (K as f32 + g);

        let mut f = vec![0.0; OBJECTIVES];
        for i in 0..OBJECTIVES {
            f[i] = 0.5 * (1.0 + g);
            for j in 0..OBJECTIVES - 1 - i {
                f[i] *= values[j];
            }

            if i != 0 {
                f[i] *= 1.0 - values[OBJECTIVES - 1 - i];
            }
        }

        f
    }

    let codec = FloatCodec::vector(VARIABLES, 0_f32..1_f32).with_bounds(-100.0..100.0);

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
        .offspring_selector(TournamentSelector::new(5))
        .survivor_selector(NSGA2Selector::new())
        .alter(alters!(
            SimulatedBinaryCrossover::new(1_f32, 1.0),
            UniformMutator::new(0.1),
        ))
        .fitness_fn(|geno: Vec<f32>| dtlz_1(&geno))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]", ctx.index());
        ctx.index() > 1000
    });

    // When running an MO problem, we can get the resulting pareto from from the
    // engine's epoch result. This is stored in the 'front()' field of the result here:
    let front = result.front();
}
// --8<-- [end:dtlz1]

// --8<-- [start:graph_xor]
fn graph_xor() {
    const MAX_INDEX: i32 = 500;
    const MIN_SCORE: f32 = 0.01;

    random_provider::seed(501);

    let store = vec![
        (NodeType::Input, vec![Op::var(0), Op::var(1)]),
        (NodeType::Edge, vec![Op::weight(), Op::identity()]),
        (NodeType::Vertex, ops::all_ops()),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let graph_codec = GraphCodec::directed(2, 1, store);
    let regression = Regression::new(get_dataset(), Loss::MSE);

    let engine = GeneticEngine::builder()
        .codec(graph_codec)
        .fitness_fn(regression)
        .minimizing()
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.05, 0.05),
            GraphMutator::new(0.06, 0.01).allow_recurrent(false),
        ))
        .build();

    // Using the engine iterator
    engine
        .iter()
        .logging()
        .until_score(MIN_SCORE)
        .last()
        .inspect(display);

    fn display(result: &Generation<GraphChromosome<Op<f32>>, Graph<Op<f32>>>) {
        let mut reducer = GraphEvaluator::new(result.value());
        for sample in get_dataset().iter() {
            let output = &reducer.eval_mut(sample.input())[0];
            println!(
                "{:?} -> epected: {:?}, actual: {:.3?}",
                sample.input(),
                sample.output(),
                output
            );
        }

        println!("{result:?}");
    }

    fn get_dataset() -> DataSet<f32> {
        let inputs = vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];

        let answers = vec![vec![0.0], vec![0.0], vec![1.0], vec![1.0]];

        DataSet::new(inputs, answers)
    }
}
// --8<-- [end:graph_xor]

// --8<-- [start:tree]
fn tree() {
    const MIN_SCORE: f32 = 0.01;
    const MAX_SECONDS: f64 = 1.0;

    random_provider::seed(518);

    let store = vec![
        (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
        (NodeType::Leaf, vec![Op::var(0)]),
    ];

    let tree_codec = TreeCodec::single(3, store).constraint(|root| root.size() < 30);
    let regression = Regression::new(get_dataset(), Loss::MSE);

    let mut engine = GeneticEngine::builder()
        .codec(tree_codec)
        .fitness_fn(regression)
        .minimizing()
        .mutator(HoistMutator::new(0.01))
        .crossover(TreeCrossover::new(0.7))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index(), ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    display(&result);

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
}
// --8<-- [end:tree]

fn main() {}
