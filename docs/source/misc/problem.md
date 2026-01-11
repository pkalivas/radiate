  
# Problem API

For certain optimization problems, it is useful to have a more structured way to define a `problem`. For instance, it may be useful to hold stateful information within a fitness function, store data in a more unified way, or evaluate a `Genotype<C>` directly without decoding. The `problem` interface provides a way to do just that. Under the hood of the `GeneticEngine`, the builder constructs a `problem` object that holds the `codec` and fitness function. Because of this, when using the `problem` API, we don't need a `codec` or a fitness function - the `problem` will take care of that for us. 

See the [image evolution example](https://github.com/pkalivas/radiate/tree/master/examples/image-evo) for a more detailed example of using the `problem` API.

=== ":fontawesome-brands-python: Python"

    The `Problem` interface is not available in python because it isn't needed.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Define a problem struct that holds stateful information
    struct MyFloatProblem {
        num_genes: usize,
        value_range: Range<f32>,
    }

    impl Problem<FloatChromosome, Vec<f32>> for MyFloatProblem {
        fn encode(&self) -> Genotype<FloatChromosome> {
            Genotype::from(FloatChromosome::from((self.num_genes, self.value_range.clone())))
        }
        
        fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<f32> {
            genotype.genes().iter().map(|gene| gene.value()).collect()
        }

        fn eval(&self, genotype: &Genotype<FloatChromosome>) -> Score {
            // Evaluate the genotype directly without decoding
            my_fitness_fn(&genotype)
        }
    }

    // The `Problem<C, T>` trait requires `Send` and `Sync` implementations
    unsafe impl Send for MyFloatProblem {}
    unsafe impl Sync for MyFloatProblem {}

    // Create an engine with the problem
    let mut engine = GeneticEngine::builder()
        .problem(MyProblem { num_genes: 10, value_range: 0.0..1.0 })
        .build();

    // Run the engine
    let result = engine.run(|epoch| epoch.index() >= 100);
    ```
