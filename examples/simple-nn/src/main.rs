use radiate::prelude::*;

const MIN_SCORE: f32 = 0.0001;
const MAX_INDEX: usize = 500;
const MAX_SECONDS: f64 = 1.0;

fn main() {
    random_provider::set_seed(12345);

    let inputs = vec![
        vec![0.0, 0.0],
        vec![1.0, 1.0],
        vec![1.0, 0.0],
        vec![0.0, 1.0],
    ];

    let target = vec![0.0, 0.0, 1.0, 1.0];

    let codec = NeuralNetCodec {
        shapes: vec![(2, 8), (8, 8), (8, 1)],
        inputs: inputs.clone(),
        target: target.clone(),
    };

    let mut engine = GeneticEngine::builder()
        .minimizing()
        .codec(codec.clone())
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .alter(alters!(
            IntermediateCrossover::new(0.75, 0.1),
            ArithmeticMutator::new(0.03),
            GaussianMutator::new(0.03)
        ))
        .fitness_fn(move |net: NeuralNet| net.error(&inputs, &target))
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index(), ctx.score().as_f32());
        ctx.score().as_f32() < MIN_SCORE || ctx.index() == MAX_INDEX || ctx.seconds() > MAX_SECONDS
    });

    println!("Seconds: {:?}", result.seconds());
    println!("{}", result.metrics());
    let best = result.value().clone();
    for (input, target) in codec.inputs.iter().zip(codec.target.iter()) {
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
    pub fn feed_forward(&self, input: Vec<f32>) -> Vec<f32> {
        let mut output = input;
        for layer in self.layers.iter() {
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

                // Sigmoid activation
                new_output.push(1.0 / (1.0 + (-sum).exp()));
            }

            output = new_output;
        }

        output
    }

    pub fn error(&self, data: &[Vec<f32>], target: &[f32]) -> f32 {
        let mut score = 0_f32;
        for (input, target) in data.iter().zip(target.iter()) {
            let output = self.feed_forward(input.clone());
            score += (target - output[0]).powi(2);
        }

        score / data.len() as f32
    }
}

#[derive(Clone)]
pub struct NeuralNetCodec {
    pub shapes: Vec<(usize, usize)>,
    pub inputs: Vec<Vec<f32>>,
    pub target: Vec<f32>,
}

impl Codec<FloatChromosome, NeuralNet> for NeuralNetCodec {
    fn encode(&self) -> Genotype<FloatChromosome> {
        Genotype::from(
            self.shapes
                .iter()
                .map(|shape| FloatChromosome::from((shape.0 * shape.1, -1.0..1.0, -100.0..100.0)))
                .collect::<Vec<FloatChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<FloatChromosome>) -> NeuralNet {
        let mut layers = Vec::new();
        for (i, chromosome) in genotype.iter().enumerate() {
            layers.push(
                chromosome
                    .iter()
                    .as_slice()
                    .chunks(self.shapes[i].1 as usize)
                    .map(|chunk| {
                        chunk
                            .iter()
                            .map(|gene| *gene.allele())
                            .collect::<Vec<f32>>()
                    })
                    .collect::<Vec<Vec<f32>>>(),
            );
        }

        NeuralNet { layers }
    }
}
