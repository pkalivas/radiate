# Executors

---

During the process of evolution, various operations are pushed into an `executor` to be run. Things like evaluating fitness, dispatching events, etc. Executors are responsible for managing how these operations are run, whether that be in a single thread or multiple threads and how those threads are managed.

Currently, radiate supports three executors:

- `Serial`: Runs all operations in the main thread, one at a time.
    * This is the default executor if none is specified.
- `WorkerPool`: Uses a [rayon](https://github.com/rayon-rs/rayon/tree/main) thread pool to run operations concurrently.
    * note that in rust this requires the `rayon` feature to be enabled in `Cargo.toml`. Python includes this by default.
- `FixedSizedWorkerPool(num_threads)`: Uses an internal thread pool with a fixed number of threads to run operations concurrently.

## Example

Continuing with our example from the previous sections - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll keep the previous inputs the same as before, but now we add an `executor` to the `GeneticEngine`.

=== ":fontawesome-brands-python: Python"

    !!! tip "Python concurrency"

        The `WorkerPool` and `FixedSizedWorkerPool` executors use multiple threads to run the fitness function concurrently. If you are not using a free-threaded interpreter (ie: `python3.13t/3.14t`) or the GIL is enabled, the engine will raise an exception. There are a few caveats to this with genetic programming problems - see the [GP regression](gp/regression.md) section for more details. 

        If you are in fact using a free-threaded interpreter, your engine can take advantage of multiple threads to evaluate fitness concurrently. This can **significantly** speed up evolution, especially if your fitness function is computationally expensive. However, your fitness function **must** be thread-safe.

    ```python
    --8<-- "python/executors.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

    To use the `WorkerPool` executor in rust (which uses rayon), ensure you have the `rayon` feature enabled in your `Cargo.toml`:

    ```toml
    [dependencies]
    radiate = { version = "x", features = ["rayon"] }
    ```

    ```rust
    use radiate::*;

    // Define a fitness function that uses the decoded values
    fn fitness_fn(individual: Vec<f32>) -> f32 {
        let a = individual[0];
        let b = individual[1];
        calculate_error(a, b)  // Your error calculation here
    }

    // This will produce a Genotype<FloatChromosome> with 1 FloatChromosome which
    // holds 2 FloatGenes (a and b), each with a value between -1.0 and 1.0 and a bound between -10.0 and 10.0
    let codec = FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0);

    // Define the executor - here we use a fixed size worker pool with 4 threads
    let executor = Executor::FixedSizedWorkerPool(4);
    // Alternatively, you can use a WorkerPool (which uses rayon's global thread pool)
    let executor = Executor::WorkerPool;
    // Or for single-threaded execution, use Serial - this is the default if none is specified
    let executor = Executor::Serial;

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(BoltzmannSelector::new(4.0))
        .survivor_selector(TournamentSelector::new(3))
        .fitness_fn(fitness_fn)
		.alterers(alters!(
            GaussianMutator::new(0.1),
		    BlendCrossover::new(0.8, 0.5)
        )) 
        .executor(executor)         // Set the executor here
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 1000 || generation.score().as_f32() <= 0.01
    });
    ```

    You can also use the convenient `.parallel()` method on the engine builder. If the `rayon` feature is enabled, this will set the executor to `WorkerPool`, otherwise it will set it to `FixedSizedWorkerPool(std::thread::available_parallelism().unwrap().get())`.:

    ```rust
    let mut engine = GeneticEngine::builder()
        // ... other builder methods ...
        .parallel() 
        .build();
    ```