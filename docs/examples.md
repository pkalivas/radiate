
Check the git repo [examples](https://github.com/pkalivas/radiate/tree/master/radiate-examples) for a more 
comprehensive list of examples.

## MinSum

> Objective - Find a set of numbers that sum to the minimum value (0).

??? example MinSum

    ```rust
    use radiate::*;

    const MIN_SCORE: i32 = 0;

    fn main() {
        let codex = IntCodex::new(1, 10, 0, 100).with_bounds(0, 100);

        let engine = GeneticEngine::from_codex(&codex)
            .population_size(150)
            .minimizing()
            .offspring_selector(EliteSelector::new())
            .survivor_selector(TournamentSelector::new(4))
            .alter(alters!(
                ArithmeticMutator::new(0.01),
                UniformCrossover::new(0.5),
            ))
            .fitness_fn(|genotype: Vec<Vec<i32>>| {
                Score::from_int(
                    genotype
                        .iter()
                        .fold(0, |acc, chromosome| acc + chromosome.iter().sum::<i32>()),
                )
            })
            .build();

        let result = engine.run(|output| {
            println!("[ {:?} ]: {:?}", output.index, output.best.first().unwrap());
            output.score().as_int() == MIN_SCORE
        });

        println!("{:?}", result);
    }
    ```

## NQueens

> Objective - Place `n` queens on an `n x n` chessboard such that no two queens threaten each other.

??? example

    ```rust
    use radiate::*;

    const N_QUEENS: usize = 16;

    fn main() {
        let codex = IntCodex::<i8>::new(1, N_QUEENS, 0, N_QUEENS as i8);

        let engine = GeneticEngine::from_codex(&codex)
            .minimizing()
            .num_threads(10)
            .offspring_selector(RouletteSelector::new())
            .alter(alters!(
                MultiPointCrossover::new(0.75, 2),
                UniformMutator::new(0.01)
            ))
            .fitness_fn(|genotype: Vec<Vec<i8>>| {
                let queens = &genotype[0];
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

                Score::from_usize(score)
            })
            .build();

        let result = engine.run(|output| {
            println!("[ {:?} ]: {:?}", output.index, output.score().as_usize());

            output.score().as_usize() == 0
        });

        println!("\nResult Queens Board ({:.3?}):", result.timer.duration());

        let board = &result.best[0];
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
    ```


## Knapsack

> Objective - Find the optimal combination of items to maximize the total value without exceeding the weight limit.

??? example

    ```rust
    use std::sync::LazyLock;

    use radiate::*;

    const KNAPSACK_SIZE: usize = 15;
    const MAX_EPOCHS: i32 = 50;

    static KNAPSACK: LazyLock<Knapsack> = LazyLock::new(|| Knapsack::new(KNAPSACK_SIZE));

    fn main() {
        seed_rng(12345);
        let codex = SubSetCodex::new(&KNAPSACK.items);

        let engine = GeneticEngine::from_codex(&codex)
            .max_age(MAX_EPOCHS)
            .fitness_fn(move |genotype: Vec<&Item>| Knapsack::fitness(&KNAPSACK.capacity, &genotype))
            .build();

        let result = engine.run(|output| output.index == MAX_EPOCHS);

        println!(
            "Result Value Total=[ {:?} ]",
            Knapsack::value_total(&result.best)
        );
        println!(
            "Result Weigh Total=[ {:?} ]",
            Knapsack::weight_total(&result.best)
        );
        println!("Max Weight=[{:?}]", KNAPSACK.capacity);
    }

    pub struct Knapsack {
        pub capacity: f32,
        pub size: usize,
        pub items: Vec<Item>,
    }

    impl Knapsack {
        pub fn new(size: usize) -> Self {
            let items = Item::random_collection(size);
            Knapsack {
                capacity: size as f32 * 100_f32 / 3_f32,
                size,
                items,
            }
        }

        pub fn fitness(capacity: &f32, genotype: &Vec<&Item>) -> Score {
            let mut sum = 0_f32;
            let mut weight = 0_f32;
            for item in genotype {
                sum += item.value;
                weight += item.weight;
            }

            if weight > *capacity {
                Score::from_f32(0_f32)
            } else {
                Score::from_f32(sum)
            }
        }

        pub fn value_total(items: &Vec<&Item>) -> f32 {
            items.iter().fold(0_f32, |acc, item| acc + item.value)
        }

        pub fn weight_total(items: &Vec<&Item>) -> f32 {
            items.iter().fold(0_f32, |acc, item| acc + item.weight)
        }
    }

    impl std::fmt::Debug for Knapsack {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut sum = 0_f32;
            for item in &self.items {
                sum += item.value;
            }

            write!(
                f,
                "Knapsack[capacity={:.2}, size={:.2}, sum={:.2}]",
                self.capacity, self.size, sum
            )
        }
    }

    #[derive(Debug, Clone)]
    pub struct Item {
        pub weight: f32,
        pub value: f32,
    }

    impl Item {
        pub fn new(weight: f32, value: f32) -> Self {
            Item { weight, value }
        }

        pub fn random_collection(size: usize) -> Vec<Item> {
            (0..size)
                .map(|_| {
                    Item::new(
                        random_provider::random::<f32>() * 100.0,
                        random_provider::random::<f32>() * 100.0,
                    )
                })
                .collect()
        }
    }

    ```

## Rastrigin

> Objective - Find the global minimum of the Rastrigin function.

??? example

    ```rust
    use radiate::*;

    const MIN_SCORE: f32 = 0.00;
    const MAX_SECONDS: f64 = 5.0;
    const A: f32 = 10.0;
    const RANGE: f32 = 5.12;
    const N_GENES: usize = 2;

    fn main() {
        let codex = FloatCodex::new(1, N_GENES, -RANGE, RANGE).with_bounds(-RANGE, RANGE);

        let engine = GeneticEngine::from_codex(&codex)
            .minimizing()
            .population_size(500)
            .alter(alters!(
                UniformCrossover::new(0.5),
                ArithmeticMutator::new(0.01)
            ))
            .fitness_fn(move |genotype: Vec<Vec<f32>>| {
                let mut value = A * N_GENES as f32;
                for i in 0..N_GENES {
                    value += genotype[0][i].powi(2)
                        - A * (2.0 * std::f32::consts::PI * genotype[0][i]).cos();
                }

                Score::from_f32(value)
            })
            .build();

        let result = engine.run(|output| {
            println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
            output.score().as_float() <= MIN_SCORE || output.seconds() > MAX_SECONDS
        });

        println!("{:?}", result);
    }
    ```

## XOR Problem (Neural Network)

> Objective - Evolve a traditional neural network to solve the XOR problem.

??? example

    ```rust
    use radiate::*;

    const MIN_SCORE: f32 = 0.0001;
    const MAX_INDEX: i32 = 500;
    const MAX_SECONDS: u64 = 5;

    fn main() {
        let inputs = vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];

        let target = vec![0.0, 0.0, 1.0, 1.0];

        let codex = NeuralNetCodex {
            shapes: vec![(2, 8), (8, 8), (8, 1)],
            inputs: inputs.clone(),
            target: target.clone(),
        };

        let engine = GeneticEngine::from_codex(&codex)
            .population_size(100)
            .minimizing()
            .offspring_selector(BoltzmannSelector::new(4_f32))
            .alter(alters!(
                IntermediateCrossover::new(0.75, 0.1),
                ArithmeticMutator::new(0.05),
            ))
            .fitness_fn(move |genotype: NeuralNet| genotype.error(&inputs, &target))
            .build();

        let result = engine.run(|output| {
            println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
            output.score().as_float() < MIN_SCORE
                || output.index == MAX_INDEX
                || output.timer.duration().as_secs() > MAX_SECONDS
        });

        println!("{:?}", result.metrics);
        let best = result.best;
        for (input, target) in codex.inputs.iter().zip(codex.target.iter()) {
            let output = best.feed_forward(input.clone());
            println!(
                "{:?} -> expected: {:?}, actual: {:.3?}",
                input, target, output
            );
        }
    }

    #[derive(Clone)]
    pub struct NeuralNet {
        pub layers: Vec<Vec<Vec<f32>>>,
    }

    impl NeuralNet {
        pub fn new(layers: Vec<Vec<Vec<f32>>>) -> Self {
            NeuralNet { layers }
        }

        pub fn feed_forward(&self, input: Vec<f32>) -> Vec<f32> {
            let mut output = input;

            for layer in &self.layers {
                let layer_height = layer.len();
                let layer_width = layer[0].len();

                if output.len() != layer_height {
                    panic!(
                        "Input size does not match layer size: {} != {}",
                        output.len(),
                        layer_width
                    );
                }

                let mut new_output = Vec::new();
                for i in 0..layer_width {
                    let mut sum = 0_f32;
                    for j in 0..layer_height {
                        sum += layer[j][i] * output[j];
                    }

                    if i == layer_width - 1 {
                        new_output.push(if sum > 0.0 { sum } else { 0.0 });
                    } else {
                        new_output.push(1.0 / (1.0 + (-sum).exp()));
                    }
                }

                output = new_output;
            }

            output
        }

        pub fn error(&self, data: &[Vec<f32>], target: &[f32]) -> Score {
            let mut score = 0_f32;
            for (input, target) in data.iter().zip(target.iter()) {
                let output = self.feed_forward(input.clone());
                score += (target - output[0]).powi(2);
            }

            Score::from_f32(score / data.len() as f32)
        }
    }

    pub struct NeuralNetCodex {
        pub shapes: Vec<(i32, i32)>,
        pub inputs: Vec<Vec<f32>>,
        pub target: Vec<f32>,
    }

    impl Codex<FloatChromosome, NeuralNet> for NeuralNetCodex {
        fn encode(&self) -> Genotype<FloatChromosome> {
            let mut chromosomes = Vec::<FloatChromosome>::new();
            for shape in &self.shapes {
                chromosomes.push(FloatChromosome::from_genes(
                    (0..shape.0 * shape.1)
                        .map(|_| FloatGene::new(-1.0, 1.0))
                        .collect::<Vec<FloatGene>>(),
                ));
            }

            Genotype::from_chromosomes(chromosomes)
        }

        fn decode(&self, genotype: &Genotype<FloatChromosome>) -> NeuralNet {
            let mut layers = Vec::new();
            for (i, chromosome) in genotype.iter().enumerate() {
                let layer = chromosome
                    .iter()
                    .as_slice()
                    .chunks(self.shapes[i].1 as usize)
                    .map(|chunk| chunk.iter().map(|gene| gene.allele).collect::<Vec<f32>>())
                    .collect::<Vec<Vec<f32>>>();

                layers.push(layer);
            }

            NeuralNet { layers }
        }
    }
    ```

## XOR Problem (NeuroEvolution)

> Objective - Evolve a `Graph<f32>` to solve the XOR problem (NeuroEvolution).
>
>  Warning - only available with the `radiate-extensions` crate

??? example

    ```rust
    use radiate::*;
    use radiate_gp::*;

    const MAX_INDEX: i32 = 500;
    const MIN_SCORE: f32 = 0.01;

    fn main() {
        let graph_codex = GraphCodex::regression(2, 1).set_outputs(vec![op::sigmoid()]);

        let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

        let engine = GeneticEngine::from_codex(&graph_codex)
            .minimizing()
            .alter(alters!(
                GraphCrossover::new(0.5, 0.5),
                NodeMutator::new(0.1, 0.05),
                GraphMutator::new(vec![
                    NodeMutate::Forward(NodeType::Weight, 0.05),
                    NodeMutate::Forward(NodeType::Aggregate, 0.03),
                    NodeMutate::Forward(NodeType::Gate, 0.03),
                ]),
            ))
            .fitness_fn(move |genotype: Graph<f32>| {
                let mut reducer = GraphReducer::new(&genotype);
                Score::from_f32(regression.error(|input| reducer.reduce(input)))
            })
            .build();

        let result = engine.run(|output| {
            println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
            output.index == MAX_INDEX || output.score().as_float() < MIN_SCORE
        });

        display(&result);
    }

    fn display(result: &EngineOutput<NodeChromosome<f32>, Graph<f32>>) {
        let mut reducer = GraphReducer::new(&result.best);
        for sample in get_sample_set().get_samples().iter() {
            let output = &reducer.reduce(&sample.1);
            println!(
                "{:?} -> epected: {:?}, actual: {:.3?}",
                sample.1, sample.2, output
            );
        }

        println!("{:?}", result)
    }

    fn get_sample_set() -> DataSet<f32> {
        let inputs = vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];

        let answers = vec![vec![0.0], vec![0.0], vec![1.0], vec![1.0]];

        DataSet::from_vecs(inputs, answers)
    }
    ```