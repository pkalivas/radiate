
# Random

Random number generation is a crucial aspect of evolutionary algorithms. Radiate provides a random interface that governs all random number generation within the library through the `random_provider`. This allows for consistent and reproducible results across different runs of the algorithm.

Through the `random_provider`, you can access various random number generation methods, such as generating random floats, integers, bools, selecting random elements from a list, shuffling elements in a list, among others. This ensures that all stochastic processes within the library are controlled and can be easily managed.

Here's an example of how to use the `random_provider`:

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # set a seed for reproducibility
    rd.random.seed(42)  

    # random float
    rand_float = rd.random.float(min=0.0, max=1.0)

    # random integer
    rand_int = rd.random.int(min=0, max=10)

    # random bool with 50% chance of being true
    rand_bool = rd.random.bool(prob=0.5)

    # randomly sample 2 elements from a list
    rand_choice = rd.random.sample(data=[1, 2, 3, 4, 5], count=2)

    # choose a random element from a list
    rand_element = rd.random.choose(data=[1, 2, 3, 4, 5])
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // set a seed for reproducibility
    random_provider::set_seed(42);

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
    ```
