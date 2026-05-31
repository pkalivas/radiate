use radiate::prelude::*;

fn main() {
    // --8<-- [start:fixed_rate]
    let fixed_rate = Rate::Fixed(0.01);
    // --8<-- [end:fixed_rate]

    // --8<-- [start:linear_rate]
    // start, end, steps
    let linear_rate = Rate::Linear(0.1, 0.001, 100);
    // --8<-- [end:linear_rate]

    // --8<-- [start:step_rate]
    // vec [(step_idx, rate)]
    let steps = vec![(0, 0.1), (25, 0.5), (75, 0.9)];
    let step_rate = Rate::Stepwise(steps);
    // --8<-- [end:step_rate]

    // --8<-- [start:cyclical_rate]
    // min, max, period, shape
    let sine_rate = Rate::Cyclical(0.1, 0.9, 25, rate::CycleShape::Sine);
    // --8<-- [end:cyclical_rate]

    // --8<-- [start:triangular_cyclical_rate]
    // min, max, period, shape
    let triangular_rate = Rate::Cyclical(0.1, 0.9, 25, rate::CycleShape::Triangle);
    // --8<-- [end:triangular_cyclical_rate]

    // --8<-- [start:exponential_rate]
    // start, end, half_life
    let exp_rate = Rate::Exponential(0.5, 0.1, 25);
    // --8<-- [end:exponential_rate]
}
