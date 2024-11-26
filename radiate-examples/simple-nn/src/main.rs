use radiate::*;

fn main() {
    let inputs = vec![
        vec![0.0, 0.0],
        vec![0.0, 1.0],
        vec![1.0, 0.0],
        vec![1.0, 1.0],
    ];

    let target = vec![0.0, 1.0, 1.0, 0.0];

    let codex = NeuralNetCodex {
        shapes: vec![(2, 5), (5, 7), (7, 5), (5, 1)],
        inputs: inputs.clone(),
        target: target.clone(),
    };

    let engine = GeneticEngine::from_codex(&codex)
        .population_size(150)
        .minimizing()
        .offspring_selector(RouletteSelector::new())
        .survivor_selector(TournamentSelector::new(4))
        .alterer(vec![
            Alterer::mutation(NumericMutator::new(0.01)),
            Alterer::crossover(MultiPointCrossover::new(0.5, 2)),
        ])
        .fitness_fn(move |genotype: NeuralNet| {
            let score = genotype.error(&inputs, &target);
            Score::from_f32(score)
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.score().as_float());
        output.score().as_float() == 0.0 || output.index == 1000
    });

    // Print prediction vs target
    let best = result.best;

    for input in codex.inputs.iter() {
        let output = best.feed_forward(input.clone());
        println!("{:?} -> {:?}", input, output);
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
            let mut new_output = vec![0.0; layer[0].len()];
            for (i, _) in layer[0].iter().enumerate() {
                let mut sum = 0.0;
                for (j, input_node) in output.iter().enumerate() {
                    sum += input_node * layer[j][i];
                }
                new_output[i] = 1.0 / (1.0 + (-sum).exp());
            }
            output = new_output;
        }

        output
    }

    pub fn error(&self, data: &[Vec<f32>], target: &[f32]) -> f32 {
        let mut score = 0_f32;

        for input in data.iter() {
            let output = self.feed_forward(input.clone());

            // MSE
            score += output
                .iter()
                .zip(target.iter())
                .fold(0.0, |acc, (o, t)| acc + (o - t).powi(2));
        }

        score / data.len() as f32
    }
}

pub struct NeuralNetCodex {
    pub shapes: Vec<(i32, i32)>,
    pub inputs: Vec<Vec<f32>>,
    pub target: Vec<f32>,
}

impl Codex<FloatChromosome, NeuralNet> for NeuralNetCodex {
    fn encode(&self) -> Genotype<FloatChromosome> {
        let mut chromosomes = Vec::new();
        for shape in &self.shapes {
            let mut chrome = FloatChromosome::from_range(0..shape.0 * shape.1);
            chrome.normalize();
            chromosomes.push(chrome);
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
