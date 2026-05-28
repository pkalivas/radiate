use radiate::prelude::*;

fn main() {
    // --8<-- [start:blend_crossover]
    // rate, alpha
    let crossover = BlendCrossover::new(0.1, 0.5);
    // --8<-- [end:blend_crossover]

    // --8<-- [start:intermediate_crossover]
    // rate, alpha
    let crossover = IntermediateCrossover::new(0.1, 0.5);
    // --8<-- [end:intermediate_crossover]

    // --8<-- [start:mean_crossover]
    // rate
    let crossover = MeanCrossover::new(0.1);
    // --8<-- [end:mean_crossover]

    // --8<-- [start:multipoint_crossover]
    // rate, num_points
    let crossover = MultiPointCrossover::new(0.1, 2);
    // --8<-- [end:multipoint_crossover]

    // --8<-- [start:pmx_crossover]
    // rate
    let crossover = PMXCrossover::new(0.1);
    // --8<-- [end:pmx_crossover]

    // --8<-- [start:edge_recombination_crossover]
    // rate
    let crossover = EdgeRecombinationCrossover::new(0.1);
    // --8<-- [end:edge_recombination_crossover]

    // --8<-- [start:shuffle_crossover]
    // rate
    let crossover = ShuffleCrossover::new(0.1);
    // --8<-- [end:shuffle_crossover]

    // --8<-- [start:sbx_crossover]
    // rate, contiguity
    let crossover = SimulatedBinaryCrossover::new(0.1, 0.5);
    // --8<-- [end:sbx_crossover]

    // --8<-- [start:uniform_crossover]
    // rate
    let crossover = UniformCrossover::new(0.1);
    // --8<-- [end:uniform_crossover]
}
