use divan::AllocProfiler;
use radiate::*;

// const MIN_SCORE: f32 = 0.0001;
// const MAX_INDEX: i32 = 500;
// const MAX_SECONDS: u64 = 1;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

// fn run() {
//     random_provider::set_seed(200);

//     let inputs = vec![
//         vec![0.0, 0.0],
//         vec![1.0, 1.0],
//         vec![1.0, 0.0],
//         vec![0.0, 1.0],
//     ];

//     let target = vec![0.0, 0.0, 1.0, 1.0];

//     let codex = NeuralNetCodex {
//         shapes: vec![(2, 8), (8, 8), (8, 1)],
//         inputs: inputs.clone(),
//         target: target.clone(),
//     };

//     let engine = GeneticEngine::from_codex(&codex)
//         .minimizing()
//         .num_threads(1)
//         .offspring_selector(BoltzmannSelector::new(4_f32))
//         .alter(alters!(
//             IntermediateCrossover::new(0.75, 0.1),
//             ArithmeticMutator::new(0.03),
//         ))
//         .fitness_fn(move |genotype: NeuralNet| genotype.error(&inputs, &target))
//         .build();

//     engine.run(|output| {
//         output.score().as_float() < MIN_SCORE
//             || output.index == MAX_INDEX
//             || output.timer.duration().as_secs() > MAX_SECONDS
//     });
// }

// #[derive(Clone)]
// pub struct NeuralNet {
//     pub layers: Vec<Vec<Vec<f32>>>,
// }

// impl NeuralNet {
//     pub fn feed_forward(&self, input: Vec<f32>) -> Vec<f32> {
//         let mut output = input;

//         for layer in &self.layers {
//             let layer_height = layer.len();
//             let layer_width = layer[0].len();

//             if output.len() != layer_height {
//                 panic!(
//                     "Input size does not match layer size: {} != {}",
//                     output.len(),
//                     layer_width
//                 );
//             }

//             let mut new_output = Vec::new();
//             for i in 0..layer_width {
//                 let mut sum = 0_f32;
//                 for j in 0..layer_height {
//                     sum += layer[j][i] * output[j];
//                 }

//                 if i == layer_width - 1 {
//                     new_output.push(if sum > 0.0 { sum } else { 0.0 });
//                 } else {
//                     new_output.push(1.0 / (1.0 + (-sum).exp()));
//                 }
//             }

//             output = new_output;
//         }

//         output
//     }

//     pub fn error(&self, data: &[Vec<f32>], target: &[f32]) -> Score {
//         let mut score = 0_f32;
//         for (input, target) in data.iter().zip(target.iter()) {
//             let output = self.feed_forward(input.clone());
//             score += (target - output[0]).powi(2);
//         }

//         Score::from_f32(score / data.len() as f32)
//     }
// }

// pub struct NeuralNetCodex {
//     pub shapes: Vec<(i32, i32)>,
//     pub inputs: Vec<Vec<f32>>,
//     pub target: Vec<f32>,
// }

// impl Codex<FloatChromosome, NeuralNet> for NeuralNetCodex {
//     fn encode(&self) -> Genotype<FloatChromosome> {
//         let mut chromosomes = Vec::<FloatChromosome>::new();
//         for shape in &self.shapes {
//             chromosomes.push(FloatChromosome::from_genes(
//                 (0..shape.0 * shape.1)
//                     .map(|_| FloatGene::new(-1.0, 1.0))
//                     .collect::<Vec<FloatGene>>(),
//             ));
//         }

//         Genotype::from_chromosomes(chromosomes)
//     }

//     fn decode(&self, genotype: &Genotype<FloatChromosome>) -> NeuralNet {
//         let mut layers = Vec::new();
//         for (i, chromosome) in genotype.iter().enumerate() {
//             let layer = chromosome
//                 .iter()
//                 .as_slice()
//                 .chunks(self.shapes[i].1 as usize)
//                 .map(|chunk| chunk.iter().map(|gene| gene.allele).collect::<Vec<f32>>())
//                 .collect::<Vec<Vec<f32>>>();

//             layers.push(layer);
//         }

//         NeuralNet { layers }
//     }
// }
use radiate::random_provider::set_seed;
use radiate_extensions::architects::cells::expr;
use radiate_extensions::*;

const MIN_SCORE: f32 = 0.01;
const MAX_SECONDS: f64 = 1.0;

#[divan::bench]
fn run() {
    set_seed(200);
    let graph_codex =
        TreeCodex::regression(1, 3).set_gates(vec![expr::add(), expr::sub(), expr::mul()]);

    let regression = Regression::new(get_sample_set(), ErrorFunction::MSE);

    let engine = GeneticEngine::from_codex(&graph_codex)
        .minimizing()
        .num_threads(1)
        .alter(alters!(
            TreeCrossover::new(0.5, 10),
            NodeMutator::new(0.01, 0.05),
        ))
        .fitness_fn(move |genotype: Tree<f32>| {
            let mut reducer = TreeReducer::new(&genotype);
            Score::from_f32(regression.error(|input| reducer.reduce(input)))
        })
        .build();

    engine.run(|output| output.score().as_float() < MIN_SCORE || output.seconds() > MAX_SECONDS);
}

fn get_sample_set() -> DataSet<f32> {
    let mut inputs = Vec::new();
    let mut answers = Vec::new();

    let mut input = -1.0;
    for _ in -10..10 {
        input += 0.1;
        inputs.push(vec![input]);
        answers.push(vec![compupute(input)]);
    }

    DataSet::from_vecs(inputs, answers)
}

fn compupute(x: f32) -> f32 {
    4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x
}
