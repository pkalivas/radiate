# Example

Lets add on to our example - evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`. We'll keep the previous inputs the same as before, but now we add `diversity` to the `GeneticEngine`.

!!! note "Speciation"

    In this example we'll include speciation, but for all of the following examples we'll keep the same problem but leave speciation off (not adding the `diversity` input). This is also a good time to again highlight that speciation is **opt-in** and not free - it has a computational cost. 

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/diversity/example.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/diversity/example.rs:example"
    ```

