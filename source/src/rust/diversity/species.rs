use radiate::prelude::*;

fn your_fitness_fn(individual: Vec<f32>) -> f32 {
    individual.iter().map(|v| v.abs()).sum()
}

fn your_char_fit_fn(individual: Vec<char>) -> usize {
    individual.iter().map(|c| *c as usize).sum()
}

fn main() {
    // --8<-- [start:threshold]
    let engine = GeneticEngine::builder()
        .codec(CharCodec::vector(10))
        .fitness_fn(your_char_fit_fn)
        // A distance measure turns speciation on; the threshold sets how close
        // two individuals must be (per the measure) to share a species.
        .diversity(HammingDistance)
        .species_threshold(0.5)
        // ... other parameters ...
        .build();
    // --8<-- [end:threshold]

    // --8<-- [start:dynamic_threshold]
    // `species_threshold` accepts anything that implements `Into<Expr>` — the
    // same mechanism covered in depth on the Rates page. Here it widens from
    // 0.3 to 0.9 across the first 100 generations: start fine-grained (many
    // small species), then coarsen to encourage convergence.
    let (start, end, duration) = (0.3_f32, 0.9_f32, 100.0_f32);
    let progress = Expr::select(metric_names::INDEX)
        .div(duration)
        .clamp(0.0_f32, 1.0_f32);
    let widening_threshold = Expr::lit(start).add(Expr::lit(end).sub(start).mul(progress));

    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0))
        .fitness_fn(your_fitness_fn)
        .diversity(EuclideanDistance)
        .species_threshold(widening_threshold)
        // ... other parameters ...
        .build();
    // --8<-- [end:dynamic_threshold]

    // --8<-- [start:age]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0))
        .fitness_fn(your_fitness_fn)
        .diversity(EuclideanDistance)
        .species_threshold(0.5)
        // A species that survives this many generations without improving its best
        // score is culled, and its members sit out crossover/mutation that generation.
        .max_species_age(25)
        // ... other parameters ...
        .build();
    // --8<-- [end:age]

    // --8<-- [start:target_species_count]
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0))
        .fitness_fn(your_fitness_fn)
        .diversity(EuclideanDistance)
        // Instead of a distance threshold, you can specify a target number of species.
        .target_species(4)
        // ... other parameters ...
        .build();

    // --- Below is what happens internally in the engine, you do not need to do this ---

    // `target_species` builds this exact `Expr` and passes it to `species_threshold`
    // under the hood — a small PID-style controller that nudges the threshold based
    // on the error between the current and target species count:
    let target = 4;
    let target_f32 = target as f32;
    let base_val = 0.5_f32; // the initial species_threshold

    let raw_error = Expr::select(metric_names::SPECIES_COUNT).error(target_f32);

    // Proportional: smoothed count so single-generation bursts don't cause hard jumps
    let proportional = Expr::select(metric_names::SPECIES_COUNT)
        .rolling(3)
        .mean()
        .error(target_f32)
        * 0.05_f32;

    // Integral: accumulated recent error. Derivative: velocity of the error,
    // anticipating a rising/falling count before it overshoots.
    let integral = raw_error.clone().rolling(20).sum() * 0.005_f32;
    let derivative = raw_error.rolling(5).slope() * 0.02_f32;

    let species_threshold = Expr::when(Expr::select(metric_names::INDEX).lt(2_i32))
        .then(base_val)
        .otherwise(
            Expr::select(metric_names::SPECIES_THRESHOLD) + proportional + integral + derivative,
        )
        .clamp(0.0_f32, target_f32 * 2.5_f32);
    // --8<-- [end:target_species_count]
}
