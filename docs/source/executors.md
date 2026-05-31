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
    --8<-- "rust/executors.rs:example"
    ```

    You can also use the convenient `.parallel()` method on the engine builder. If `rayon` is enabled, this will use `rayon`'s global thread pool, otherwise it will use `radiate`'s internal thread pool with # cpu threads. The performance difference between the two is negligible for our use cases.

    ```rust
    --8<-- "rust/executors.rs:parallel"
    ```