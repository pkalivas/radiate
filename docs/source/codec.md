
# Codecs

---

## What is a Codec?

Radiate's `GeneticEngine` operates on an abstract representation of your domain problem using the 'Genome'. To bridge the gap between your domain and radiate's, we use a `Codec` - encoder-decoder. A `Codec` is a mechanism that encodes and decodes genetic information between the 'problem space' (your domain) and the 'solution space' (Radiate's internal representation).

Essentially, this is a component that defines how genetic information is structured and represented in your evolutionary algorithm. Think of it as a blueprint that tells the algorithm:

- What type of data you're evolving (numbers, characters, etc.)
- How that data is organized (single values, arrays, matrices, etc.)
- Any other chromosome or gene level information needed for the algorithm to work effectively.

---

## Why Do We Need Codecs?

In genetic algorithms, we need to represent potential solutions to our problem in a way that can be:

1. **Evolved**: Modified through operations like mutation and crossover
2. **Evaluated**: Tested to see how good the solution is
3. **Consistent**: Able to be encoded to chromosomes and genes which the engine can understand and operate on, then decoded back into a format that can be used in the real-world problem (e.g., your fitness function).

For example, if you're evolving neural network weights, you need to:

- Represent the weights as numbers
- Organize them in the correct structure (matrices for layers)
- Keep them within reasonable ranges (e.g., between -1 and 1)
  
See [this example](https://github.com/pkalivas/radiate/blob/master/examples/simple-nn/src/main.rs) for a simple neural network evolution using a custom codec.

---

## How Codecs Fit Into the Genetic Algorithm

Here's a simple breakdown of how codecs work in the evolution process:

1. **Initialization**: When you create a population, the codec defines how each individual's genetic information is structured and created within the population. For example, if you're evolving a list of floating-point numbers, the codec will specify how many numbers, their ranges, and how they are represented.
2. **Evaluation**: Your fitness function receives the decoded values in a format you can work with and have possibly defined.

---

## Types of Codecs

Radiate provides several codec types out of the box that should be able to cover most use cases. Each codec type is designed to handle specific data types and structures, making it easier to evolve solutions for various problems. The core codecs include:

??? note "FloatCodec"

    Use this when you need to evolve floating-point numbers. Perfect for:

    - Neural network weights
    - Mathematical function parameters
    - Continuous optimization problems
    - Real-valued parameters

    In all `FloatCodec` varients, the `bound_range` is optional and defaults to the `value_range` if not specified.


    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # For a single parameter
        codec = rd.FloatCodec.scalar(value_range=(0.0, 1.0), bound_range=(-10.0, 10.0))

        # For a list of parameters
        codec = rd.FloatCodec.vector(length=5, value_range=(-1.0, 1.0), bound_range=(-10.0, 10.0))

        # For a matrix of parameters (like neural network weights)
        codec = rd.FloatCodec.matrix(shape=(3, 2), value_range=(-0.1, 0.1), bound_range=(-1.0, 1.0))
        # -- or --
        # supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes)
        codec = rd.FloatCodec.matrix([2, 2, 2], value_range=(-0.1, 0.1), bound_range=(-1.0, 1.0))
        ```

    === ":fontawesome-brands-rust: Rust"

        Every `FloatCodec` will `encode()` a `Genotype<FloatChromosome>`.

        ```rust
        use radiate::*;

        // single float parameter
        let codec_scalar = FloatCodec::scalar(-1.0..1.0).with_bounds(-10.0..10.0); 
        let encoded_scalar: Genotype<FloatChromosome> = codec_scalar.encode();
        let decoded_scalar: f32 = codec_scalar.decode(&encoded_scalar);     

        // vector of 5 floats
        let codec_vector = FloatCodec::vector(5, -1.0..1.0).with_bounds(-10.0..10.0);   
        let encoded_vector: Genotype<FloatChromosome> = codec_vector.encode();
        let decoded_vector: Vec<f32> = codec_vector.decode(&encoded_vector);

        // 3x2 matrix of floats
        let codec_matrix = FloatCodec::matrix(3, 2, -0.1..0.1).with_bounds(-1.0..1.0);  
        let encoded_matrix: Genotype<FloatChromosome> = codec_matrix.encode();
        let decoded_matrix: Vec<Vec<f32>> = codec_matrix.decode(&encoded_matrix);
        ```

??? note "IntCodec"

    Use this when you need to evolve integer values. Good for:

    - Discrete optimization problems
    - Array indices
    - Configuration parameters that must be whole numbers

    In all `IntCodec` varients, the `bound_range` is optional and defaults to the `value_range` if not specified. 

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # For a single parameter
        codec = rd.IntCodec.scalar(value_range=(0, 1), bound_range=(-10, 10))

        # For a list of parameters
        codec = rd.IntCodec.vector(length=5, value_range=(-1, 1), bound_range=(-10, 10))

        # For a matrix of ints
        codec = rd.IntCodec.matrix(shape=(3, 2), value_range=(-1, 1), bound_range=(-10, 10))
        # -- or --
        # supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes)
        codec = rd.IntCodec.matrix([2, 2, 2], value_range=(-1, 1), bound_range=(-10, 10))
        ```

    === ":fontawesome-brands-rust: Rust"

        The type of int can be specified as `i8`, `i16`, `i32`, `i64`, `i128` or `u8`, `u16`, `u32`, `u64`, `u128` depending on your needs. Every `IntCodec<I>` will `encode()` a `Genotype<IntChromosome<I>>`.

        ```rust
        use radiate::*;

        // single float parameter
        let codec_scalar = IntCodec::scalar(-1..1).with_bounds(-10..10);
        let encoded_scalar: Genotype<IntChromosome<i32>> = codec_scalar.encode();
        let decoded_scalar: i32 = codec_scalar.decode(&encoded_scalar);

        // vector of 5 floats - specify the int type
        let codec_vector = IntCodec::<i128>::vector(5, -1..1).with_bounds(-10..10);
        let encoded_vector: Genotype<IntChromosome<i128>> = codec_vector.encode();
        let decoded_vector: Vec<i128> = codec_vector.decode(&encoded_vector);

        // 3x2 matrix of floats
        let codec_matrix = IntCodec::matrix(3, 2, -1..1).with_bounds(-10..10);
        let encoded_matrix: Genotype<IntChromosome<i32>> = codec_matrix.encode();
        let decoded_matrix: Vec<Vec<i32>> = codec_matrix.decode(&encoded_matrix);
        ```

??? note "CharCodec"

    Use this when you need to evolve character strings. Useful for:

    - Text generation
    - String-based problems

    There is an optional `char_set` parameter that allows you to specify the set of characters to use for encoding. If not specified, it defaults to lowercase letters (a-z), uppercase letters (A-Z), digits (0-9), and common punctuation ( !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~).

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # For a list of parameters
        codec = rd.CharCodec.vector(length=5, char_set='abcdefghijklmnopqrstuvwxyz')

        # For a matrix of chars
        codec = rd.CharCodec.matrix(shape=(3, 2), char_set={'a', 'b', 'c', 'd'})
        # -- or --
        # supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes) - use the default char_set
        codec = rd.CharCodec.matrix([2, 2, 2])
        ```

    === ":fontawesome-brands-rust: Rust"

        Every `CharCodec` will `encode()` a `Genotype<CharChromosome>`.

        ```rust
        use radiate::*;

        // vector of 5 chars - specify the char set
        let codec_vector = CharCodec::vector(5).with_char_set("abcdefghijklmnopqrstuvwxyz");
        let encoded_vector: Genotype<CharChromosome> = codec_vector.encode();
        let decoded_vector: Vec<char> = codec_vector.decode(&encoded_vector);

        // 3x2 matrix of chars
        let codec_matrix = CharCodec::matrix(3, 2);
        let encoded_matrix: Genotype<CharChromosome> = codec_matrix.encode();
        let decoded_matrix: Vec<Vec<char>> = codec_matrix.decode(&encoded_matrix);
        ```

??? note "BitCodec"

    Use this when you need to evolve binary data. Each `Gene` is a `BitGene` where the `Allele`, or value being evolved, is a bool. Ideal for:

    - Binary optimization problems
    - Feature selection
    - Boolean configurations
    - Subset selection problems (e.g., Knapsack problem)

    There is no `scalar` varient of the `BitCodec` because...that doesn't seem useful at all.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # For a list of parameters
        codec = rd.BitCodec.vector(5)

        # For a matrix of bools
        codec = rd.BitCodec.matrix(shape=(3, 2))
        # -- or --
        # supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes)
        codec = rd.BitCodec.matrix([2, 2, 2])
        ```

    === ":fontawesome-brands-rust: Rust"

        Every `BitCodec` will `encode()` a `Genotype<BitChromosome>`.

        ```rust
        use radiate::*;

        // vector of 5 bools
        let codec_vector = BitCodec::vector(5);
        let encoded_vector: Genotype<BitChromosome> = codec_vector.encode();
        let decoded_vector: Vec<bool> = codec_vector.decode(&encoded_vector);

        // 3x2 matrix of bools
        let codec_matrix = BitCodec::matrix(3, 2);
        let encoded_matrix: Genotype<BitChromosome> = codec_matrix.encode();
        let decoded_matrix: Vec<Vec<bool>> = codec_matrix.decode(&encoded_matrix);
        ```

??? note "SubSetCodec"

    For when you need to optimize a subset or smaller collection from a larger set. Underneath the hood, the `SubSetCodec` uses a `BitCodec` to represent the selection of items. This codec allows you to evolve a selection of items from a larger pool, where each gene represents whether an item is included (1) or excluded (0) in the subset.

    - Feature selection in machine learning
    - Knapsack problem
    - Combinatorial optimization

    === ":fontawesome-brands-python: Python"

        !!! warning ":construction: Under Construction :construction:"

            This codec is currently under construction and not yet available in the Python API.

    === ":fontawesome-brands-rust: Rust"

        Each `SubSetCodec` will `encode()` a `Genotype<BitChromosome>` and `decode()` to a `Vec<Arc<T>>` of the selected items, 
        where a selected item is "selected" if the corresponding gene in the `BitChromosome` is `true`.

        ```rust
        use radiate::*;

        #[derive(Debug, Clone)]
        pub struct Item {
            pub weight: f32,
            pub value: f32,
        }

        let items = vec![
            Item { weight: 2.0, value: 3.0 },
            Item { weight: 3.0, value: 4.0 },
            Item { weight: 4.0, value: 5.0 },
            Item { weight: 5.0, value: 6.0 },
            Item { weight: 6.0, value: 7.0 },
            Item { weight: 7.0, value: 8.0 },
            Item { weight: 8.0, value: 9.0 },
            Item { weight: 9.0, value: 10.0 },
        ];

        let subset_codec = SubSetCodec::vector(items);

        let genotype: Genotype<BitChromosome> = subset_codec.encode();
        let decoded: Vec<Arc<Item>> = subset_codec.decode(&genotype);
        ```

??? note "PermutationCodec"

    The `PermutationCodec<T>` ensures that each gene in the chromosome is a unique item from the set. Use this when you need to evolve permutations of a set of items. This codec is particularly useful for problems where the order of items matters, such as:

    - Traveling Salesman Problem (TSP)
    - Job scheduling
    - Sequence alignment

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # For a list of unique items
        codec = rd.PermutationCodec(alleles=[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
        # This will produce a Genotype<PermutationChromosome> with 1 PermutationChromosome which
        # holds 10 unique genes (0-9) in a random order.
        genotype = codec.encode()
        # Decode to a list of unique items
        decoded = codec.decode(genotype)
        # decoded will be a list of unique items from the original alleles.
        # e.g. [3, 0, 7, 1, 9, 2, 5, 6, 4, 8]
        # Note: The order of the decoded items will be the same as the order of the
        # genes in the PermutationChromosome, which is a random permutation of the original alleles.
        ```

    === ":fontawesome-brands-rust: Rust"

        Every `PermutationCodec<T>` will `encode()` a `Genotype<PermutationChromosome<T>>` and `decode()` to a `Vec<T>` where each `T` is a unique item from the given set of `allele`s.

        ```rust
        use radiate::*;

        let codec: PermutationCodec<usize> = PermutationCodec::new((0..10).collect());

        // Encode a genotype of Genotype<PermutationChromosome> and decode to a Vec<usize> where each usize is a unique index
        // from the original value_range.
        // This will ensure that the permutation is valid and does not contain duplicates.
        let genotype: Genotype<PermutationChromosome<usize>> = codec.encode();
        let decoded: Vec<usize> = codec.decode(&genotype);

        ```

??? note "FnCodec"

    The `FnCodec` is a flexible codec that allows you to define custom encoding and decoding functions for your problem. This is particularly useful when your solution space does not fit neatly into the other codec types or when you need to handle complex data structures. It allows you to specify how to encode and decode your genetic information using user-defined functions. This codec is ideal for:

    - Complex data structures that don't fit into standard codecs
    - Custom encoding/decoding logic
    - Problems where the representation is not easily defined by simple types

    === ":fontawesome-brands-python: Python"

        !!! warning ":construction: Under Construction :construction:"

            This codec is currently under construction and not yet available in the Python API.

    === ":fontawesome-brands-rust: Rust"

        Each `FnCodec<I, O>` will `encode()` a `Genotype<C>` where `C` is the `chromosome` that you choose and `decode()` to an `O`. In the below case, the type `C` is an `IntChromosome<i8>` and `O` is the output type (e.g., `NQueens`).

        ```rust
        use radiate::*;

        // A simple struct to represent the NQueens problem - this struct will be the input to your fitness function.
        const N_QUEENS: usize = 8;

        #[derive(Clone, Debug, PartialEq)]
        struct NQueens(Vec<i8>);

        // this is a simple example of the NQueens problem.
        // The resulting codec type will be FnCodec<IntChromosome<i8>, NQueens>.
        let codec: FnCodec<IntChromosome<i8>, NQueens> = FnCodec::new()
            .with_encoder(|| {
                Genotype::new(vec![IntChromosome::new((0..N_QUEENS)
                        .map(|_| IntGene::from(0..N_QUEENS as i8))
                        .collect(),
                )])
            })
            .with_decoder(|genotype| {
                NQueens(genotype[0]
                    .genes()
                    .iter()
                    .map(|g| *g.allele())
                    .collect::<Vec<i8>>())
            });

        // encode and decode
        let genotype: Genotype<IntChromosome<i8>> = codec.encode();
        let decoded: NQueens = codec.decode(&genotype);
        ```

---

## Best Practices

1. **Start Simple**: Begin with a simple codec structure and expand as needed
2. **Choose Appropriate Ranges (IntCodec & FloatCodec)**:
    - `value_range`: Set this to reasonable initial values
    - `bound_range`: Set this to the valid range for your problem
3. **Match Your Problem**: Choose the codec type that best represents your solution space
4. **Consider Structure**: Use the appropriate configuration (scalar/vector/matrix) for your problem

## Common Pitfalls to Avoid

1. **Too Wide Ranges**: Starting with very wide value ranges can make evolution slower
2. **Too Narrow Bounds**: Restrictive bound ranges might prevent finding optimal solutions
3. **Mismatched Structure**: Using the wrong codec structure can make it impossible to represent valid solutions

---

## Example

Let's look at a basic example of how to use the `Codec` for evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`.

=== ":fontawesome-brands-python: Python"

    Python also allows you to pass a flag to most codecs to specify if you want a `numpy.array` or a `list` to be returned when decoding. You can do this by passing `use_numpy=True` to the codec constructor. 

    E.g. `rd.FloatCodec.vector(length=2, value_range=(-1.0, 1.0), bound_range=(-10.0, 10.0), use_numpy=True)` will return a `numpy.array` when decoding. You can also just write the decoded value in your `fitnesss_func` in a `numpy.arry(my_decoded_value)` format to get a `numpy.array` back. The performance difference between the two is negligible, so you can choose the one that best fits your needs.

    ```python
    from typing import List
    import radiate as rd

    # Define a fitness function that uses the decoded values
    def fitness_function(individual: List[float]) -> float:    
        # Calculate how well these parameters fit your data
        a = individual[0]
        b = individual[1]
        return calculate_error(a, b)  # Your error calculation here

    # Create a codec for two parameters (a and b)
    codec = rd.FloatCodec.vector(
        length=2,                   # We need two parameters: a and b
        value_range=(-1.0, 1.0),    # Start with values between -1 and 1
        bound_range=(-10.0, 10.0)   # Allow evolution to modify the values between -10 and 10
    )

    # Create the evolution engine
    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=fitness_function,
        # ... other parameters ...
    )

    # Run the engine
    result = engine.run([rd.ScoreLimit(0.01), rd.GenerationsLimit(1000)])
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
