use radiate::*;

const MAX_INDEX: i32 = 25;

fn main() {
    let codex = FloatCodex::scalar(0.0, 2.0 * std::f32::consts::PI);

    let engine = GeneticEngine::from_codex(&codex)
        .alterer(alters![
            ArithmeticMutator::new(0.01),
            MeanCrossover::new(0.5)
        ])
        .fitness_fn(|genotype: Vec<Vec<f32>>| {
            let value = genotype.first().unwrap().first().unwrap();
            Score::from_f32(fitness(*value))
        })
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]: {:?}", output.index, output.best.first().unwrap());
        output.index == MAX_INDEX
    });

    let function_input = result.best.first().unwrap().first().unwrap();
    let function_output = fitness(*function_input);

    println!("Function input: {:?}", function_input);
    println!("Function output: {:?}", function_output);
}

// f(x) = cos(.5 + sin(x)) * cos(x)
// Min = -.938
// Max = .938
fn fitness(value: f32) -> f32 {
    (0.5 + (value).sin()).cos() * (value).cos()
}
