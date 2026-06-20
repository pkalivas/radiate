use radiate::*;

const MIN_SCORE: f32 = 1e-4_f32;
const MAX_SECONDS: f64 = 5.0;
const A: f32 = 1.0;
const B: f32 = 100.0;
const NUM_GENES: usize = 2;
const RANGE: f32 = 2.0;

fn main() {
    let codec = FloatCodec::vector(NUM_GENES, -RANGE..RANGE);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .alter(alters!(
            MeanCrossover::new(0.75),
            ArithmeticMutator::new(0.1)
        ))
        .fitness_fn(|genotype: Vec<f32>| {
            let x = genotype[0];
            let y = genotype[1];
            (A - x).powi(2) + B * (y - x.powi(2)).powi(2)
        })
        .build();

    engine
        .iter()
        .until(|view| {
            println!("[ {:?} ]: {:?}", view.index(), view.score().as_f32());
            view.score().as_f32() <= MIN_SCORE || view.seconds() > MAX_SECONDS
        })
        .last()
        .inspect(|view| println!("{:?}", view))
        .unwrap();
}
