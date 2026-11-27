use radiate::prelude::*;

fn main() {
    // --8<-- [start:random]
    // set a seed for reproducibility
    random_provider::seed(42);

    // generate random values in ranges
    let rand_float = random_provider::range(0.0..1.0);
    let rand_int = random_provider::range(0..10);

    // random bool with 50% chance of being true
    let rand_bool = random_provider::bool(0.5);

    // choose a random element from a slice and shuffle a vector
    let rand_choice = random_provider::choose(&[1, 2, 3, 4, 5]);

    // shuffle a vector in place
    let mut vec = vec![1, 2, 3, 4, 5];
    random_provider::shuffle(&mut vec);

    // random gaussian float with mean 0 and stddev 1
    let rand_gauss = random_provider::gaussian(0.0, 1.0);

    // randomly sample n elements from a slice - range of indices, probability of each index being included
    let conditional_indices = random_provider::cond_indices(0..10, 0.2);

    // get a scoped random instance which maintains state from the calling random provider
    // be aware that this is not thread-safe and should only be used in single-threaded contexts
    random_provider::with_rng(|rng| {
        let scoped_rand_float = rng.range(0.0..1.0);
        let gaussian_float = rng.gaussian(0.0, 1.0);
    });
    // --8<-- [end:random]
}
