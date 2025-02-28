use radiate::*;

const MIN_SCORE: f32 = 1e-6_f32;
const MAX_SECONDS: f64 = 5.0;
const A: f32 = 1.0;
const B: f32 = 100.0;
const NUM_CHROMOSOMES: usize = 1;
const NUM_GENES: usize = 2;
const RANGE: f32 = 2.0;

fn main() {
    let codex = FloatCodex::new(NUM_CHROMOSOMES, NUM_GENES, -RANGE..RANGE);

    let engine = GeneticEngine::from_codex(codex)
        .minimizing()
        .offspring_selector(BoltzmannSelector::new(4_f32))
        .alter(alters!(
            MeanCrossover::new(0.75),
            ArithmeticMutator::new(0.1)
        ))
        .fitness_fn(|genotype: Vec<Vec<f32>>| {
            let x = genotype[0][0];
            let y = genotype[0][1];
            (A - x).powi(2) + B * (y - x.powi(2)).powi(2)
        })
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.score().as_f32() <= MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    println!("{:?}", result);
}
