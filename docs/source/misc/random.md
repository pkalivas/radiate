
# Random

Random number generation is a crucial aspect of evolutionary algorithms. Radiate provides a random interface that governs all random number generation within the library through the `random_provider`. This allows for consistent and reproducible results across different runs of the algorithm.

Through the `random_provider`, you can access various random number generation methods, such as generating random floats, integers, bools, selecting random elements from a list, shuffling elements in a list, among others. This ensures that all stochastic processes within the library are controlled and can be easily managed.

Here's an example of how to use the `random_provider`:

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/misc/randomness.py:random"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/misc/random.rs:random"
    ```
