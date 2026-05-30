use radiate::prelude::*;
use radiate::stats::TagType;

fn main() {
    // Engine for the basic_metrics example (built outside the shown slice).
    let mut engine = GeneticEngine::builder()
        .codec(IntCodec::vector(10, 0..100))
        .minimizing()
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    // --8<-- [start:basic_metrics]
    // --- set up the engine ---

    let result = engine.run(|ctx| {
        // get the score metric from the generation context
        let temp = ctx.metrics().get("scores").unwrap();
        // get the standard deviation of the score distribution
        let std = temp.stddev();

        std < 0.01 // Example condition to stop the engine
    });

    // Access the metrics from the result
    let metrics: &MetricSet = result.metrics();

    // pretty-print the metrics dashboard
    println!("{}", metrics.dashboard());
    // --8<-- [end:basic_metrics]

    // --8<-- [start:metric_tags]
    // Create the evolution engine

    let mut engine = GeneticEngine::builder()
        .codec(IntCodec::vector(10, 0..100))
        .minimizing()
        .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
        .build();

    // Run the engine
    let result = engine.run(|generation| generation.index() >= 1000);

    // Access the metrics from the result
    let metrics: &MetricSet = result.metrics();

    // Get tags for a specific metric
    let tags = metrics.get("scores").unwrap().tags(); // [TagType::Score, TagType::Statistic, TagType::Distribution]
    for metric in metrics.iter_tagged(TagType::Alterer) {
        // ... access all metrics related to alterers (crossover, mutation) ...
    }

    // Collect unique tags contained in the MetricSet
    let unique_tags = metrics.tags().collect::<Vec<_>>();
    // --8<-- [end:metric_tags]
}
