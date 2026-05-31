  
# Problem API

For certain optimization problems, it is useful to have a more structured way to define a `problem`. For instance, it may be useful to hold stateful information within a fitness function, store data in a more unified way, or evaluate a `Genotype<C>` directly without decoding. The `problem` interface provides a way to do just that. Under the hood of the `GeneticEngine`, the builder constructs a `problem` object that holds the `codec` and fitness function. Because of this, when using the `problem` API, we don't need a `codec` or a fitness function - the `problem` will take care of that for us. 

See the [image evolution example](https://github.com/pkalivas/radiate/tree/master/examples/image-evo) for a more detailed example of using the `problem` API.

=== ":fontawesome-brands-python: Python"

    The `Problem` interface is not available in python because it isn't needed.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/misc/problem.rs:problem"
    ```
