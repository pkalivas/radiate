# Example

Let's continue with our example from the previous section - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll use the same `codec` and `fitness_function` as before, but now we'll incorporate a `selector` to choose individuals for the next generation.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/selectors/example.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/selectors/example.rs:example"
    ```