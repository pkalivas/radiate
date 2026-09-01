use radiate::prelude::*;

fn your_fitness_fn(genotype: Vec<f32>) -> f32 {
    genotype.iter().map(|x| x * x).sum()
}

fn main() {
    // --8<-- [start:callback]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(6, -5.0..5.0))
        .fitness_fn(your_fitness_fn)
        // ... other parameters ...
        .build();

    engine.subscribe::<EpochComplete<Vec<f32>>>(|event: &EpochComplete<Vec<f32>>| {
        println!(
            "Printing from event handler! [ {:?} ]: {:?}",
            event.index, event.score
        );
    });

    // Run the engine
    let result = engine.run(|generation| generation.index() >= 100);
    // --8<-- [end:callback]

    // --8<-- [start:handler]
    struct MyHandler;

    impl Handler<EpochComplete<Vec<f32>>> for MyHandler {
        fn handle(&mut self, event: &EpochComplete<Vec<f32>>, _ctx: &EventContext<'_, Self>) {
            println!(
                "Printing from event handler! [ {:?} ]: {:?}",
                event.index, event.score
            );
        }
    }

    // Create and configure the engine
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(6, -5.0..5.0))
        .fitness_fn(your_fitness_fn)
        .subscribe(MyHandler)
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| generation.index() >= 100);
    // --8<-- [end:handler]
}
