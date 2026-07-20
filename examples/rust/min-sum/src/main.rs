// use radiate::*;

// const MIN_SCORE: i32 = 0;

// fn main() {
//     random_provider::seed(42);

//     let engine = GeneticEngine::builder()
//         .codec(IntCodec::vector(10, 0..100))
//         .population_size(150)
//         .minimizing()
//         .offspring_selector(EliteSelector::new())
//         .mutator(SwapMutator::new(0.05))
//         .crossover(UniformCrossover::new(0.5))
//         .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
//         .build();

//     let result = engine
//         .iter()
//         .logging()
//         .until(|view| view.score().as_i32() == MIN_SCORE || view.seconds() >= 3.0)
//         .run();

//     println!("{:?}", result);
// }

use radiate::prelude::*;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel::<Vec<f32>>();

    // Thread 1: owns the engine and runs evolution to completion.
    let engine_thread = thread::spawn(move || {
        let engine = GeneticEngine::builder()
            .codec(FloatCodec::vector(5, 0.0..1.0))
            .maximizing()
            .fitness_fn(|geno: Vec<f32>| geno.iter().sum::<f32>())
            .build();

        engine
            .iter()
            .take(50)
            .inspect(|view| {
                // Send whatever you want from the engine's generation view to the other thread. Here we just send the best genome.
                tx.send(view.value().clone()).unwrap();
            })
            .last()
    });

    // Thread 2: has no idea an engine exists - it just reacts to scores as
    // they arrive, completely decoupled from the engine's own loop.
    let mut best = f32::MIN;
    while let Ok(value) = rx.recv() {
        let score = value.iter().sum::<f32>();
        if score > best {
            best = score;
            println!("new best: {best:.4}");
        }
    }

    let result = engine_thread.join().unwrap();
    println!("final: {result:?}");
}
