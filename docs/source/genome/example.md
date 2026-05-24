
# Example

Let's look at a basic example of how to use the `Codec` for evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`.

=== ":fontawesome-brands-python: Python"

    Python also allows you to pass a flag to most codecs to specify if you want a `numpy.array` or a `list` to be returned when decoding. You can do this by passing `use_numpy=True` to the codec constructor. 

    E.g. `rd.FloatCodec(shape=2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), use_numpy=True)` will return a `numpy.array` when decoding. You can also just write the decoded value in your `fitnesss_func` in a `numpy.arry(my_decoded_value)` format to get a `numpy.array` back. The performance difference between the two is negligible, so you can choose the one that best fits your needs.

    ```python
    --8<-- "python/genome/example.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

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

    let mut engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(fitness_fn)
        // ... other parameters ...
        .build();

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 1000 || generation.score().as_f32() <= 0.01
    });
    ```

    Some chromosomes are able to be used directly as codecs aswell. This means that you could create 
    engines using methods as seen below. For the duration of the user guide however, we'll use the above method.

    ```rust
    // This is the same as using a FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0);
    let mut engine = GeneticEngine::builder()
        .codec(FloatChromosome::from((2, -1.0..1.0, -10..10)))
        .fitness_fn(fitness_fn)
        // ... other parameters ...
        .build()

    // To create a matrix codec using a Chromosome just use a Vec
    let mut engine = GeneticEngine::builder()
        .codec(vec![
            FloatChromosome::from((2, -1.0..1.0, -10..10)),
            FloatChromosome::from(vec![
                FloatGene::from(-3.0..3.0),
                FloatGene::from((-5.0..5.0, -10.0..10.0))
            ])
        ])
        .fitness_fn(|phenotype: Vec<Vec<f32>>| {
            // ... your fitness calc ...
        })
        // ... other parameters ...
        .build()
    ```
