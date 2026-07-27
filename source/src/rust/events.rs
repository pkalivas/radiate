use radiate::prelude::*;

fn your_fitness_fn(genotype: Vec<f32>) -> f32 {
    genotype.iter().map(|x| x * x).sum()
}

fn main() {
    // --8<-- [start:callback]
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(6, -5.0..5.0))
        .fitness_fn(your_fitness_fn)
        .subscribe(|event: &EngineEvent<Vec<f32>>| {
            if let EngineEventInner::EpochComplete(index, best, metrics, score, objective) =
                event.inner()
            {
                println!("Printing from event handler! [ {:?} ]: {:?}", index, score);
            }
        })
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| generation.index() >= 100);
    // --8<-- [end:callback]

    // --8<-- [start:handler]
    struct MyHandler;

    impl EventHandler<Vec<f32>> for MyHandler {
        fn handle(&mut self, event: EngineEvent<Vec<f32>>) {
            if let EngineEventInner::EpochComplete(index, best, metrics, score, objective) =
                event.inner()
            {
                println!("Printing from event handler! [ {:?} ]: {:?}", index, score);
            }
        }
    }

    // Create and configure the engine
    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(6, -5.0..5.0))
        .subscribe(MyHandler) // Add your handler here
        .fitness_fn(your_fitness_fn)
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| generation.index() >= 100);
    // --8<-- [end:handler]
}
