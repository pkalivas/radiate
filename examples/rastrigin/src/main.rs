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

            value
        })
        .build();

    let result = engine.run(|ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score().as_f32());
        ctx.score().as_f32() <= MIN_SCORE || ctx.seconds() > MAX_SECONDS
    });

    println!("{:?}", result);
}
