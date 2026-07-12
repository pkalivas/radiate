use radiate::prelude::*;

fn main() {
    // --8<-- [start:uniform_mutator]
    // rate
    let mutator = UniformMutator::new(0.1);
    // let mutator = UniformMutator::new(Rate::Fixed(0.1));
    // --8<-- [end:uniform_mutator]

    // --8<-- [start:gaussian_mutator]
    // rate
    let mutator = GaussianMutator::new(0.1);
    // --8<-- [end:gaussian_mutator]

    // --8<-- [start:arithmetic_mutator]
    // rate
    let mutator = ArithmeticMutator::new(0.1);
    // --8<-- [end:arithmetic_mutator]

    // --8<-- [start:swap_mutator]
    // rate
    let mutator = SwapMutator::new(0.1);
    // --8<-- [end:swap_mutator]

    // --8<-- [start:scramble_mutator]
    // rate
    let mutator = ScrambleMutator::new(0.1);
    // --8<-- [end:scramble_mutator]

    // --8<-- [start:invert_mutator]
    // rate
    let mutator = InversionMutator::new(0.1);
    // --8<-- [end:invert_mutator]

    // --8<-- [start:polynomial_mutator]
    // rate, eta
    let mutator = PolynomialMutator::new(0.1, 20.0);
    // --8<-- [end:polynomial_mutator]

    // --8<-- [start:jitter_mutator]
    // rate, magnitude
    let mutator = JitterMutator::new(0.1, 0.5);
    // --8<-- [end:jitter_mutator]
}
