use radiate::prelude::*;

fn my_fitness_fn(genotype: f32) -> f32 {
    genotype // placeholder: your real fitness logic here
}

// `iter()` consumes the engine, so each example below starts from a freshly built one.
fn build_engine() -> GeneticEngine<FloatChromosome<f32>, f32> {
    GeneticEngine::builder()
        .codec(FloatCodec::scalar(0.0..1.0))
        .fitness_fn(|genotype: f32| my_fitness_fn(genotype))
        .build()
}

fn main() {
    // --8<-- [start:single_limit]
    let engine = build_engine();
    let result = engine.iter().until_generation(50).last().unwrap();
    // --8<-- [end:single_limit]

    // --8<-- [start:combined_limits]
    // Stops on whichever trips first - almost always the score target here, well
    // before the 10,000-generation ceiling.
    let engine = build_engine();
    let result = engine
        .iter()
        .limit((Limit::Generation(10_000), Limit::Score(0.01.into())))
        .last()
        .unwrap();
    // --8<-- [end:combined_limits]

    // --8<-- [start:until_closure]
    // `until` takes an arbitrary predicate over a borrowed `GenerationView` - still
    // routed through the cheap, Limit-driven `run()`/`.last()`, so no `Generation` is
    // built until it trips.
    let engine = build_engine();
    let result = engine
        .iter()
        .until(|view: GenerationView<FloatChromosome<f32>, f32>| {
            view.index() >= 20 && view.score().as_f32() < 0.05
        })
        .last()
        .unwrap();
    // --8<-- [end:until_closure]

    // --8<-- [start:iterator_fallback]
    // A `Limit` can only answer "should I stop?" - it can't hand you the intermediate
    // state. Collecting the score history needs the real `Generation` at every step,
    // which means the iterator, not `run()`/`.last()`.
    let engine = build_engine();
    let mut score_history = Vec::new();
    for epoch in engine.iter().take(50) {
        score_history.push(epoch.score().as_f32());
    }
    assert_eq!(score_history.len(), 50);
    // --8<-- [end:iterator_fallback]
}
