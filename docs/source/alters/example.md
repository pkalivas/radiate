# Example

Continuing with our example from the previous two sections - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll use the same `codec` and `fitness_function` as before, but this time we'll add alterers to the `GeneticEngine` to evolve the parameters.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/alters/example.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/alters/example.rs:example"
    ```
