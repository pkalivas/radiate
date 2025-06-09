
# Codecs

## What is a Codec?

Radiate's `GeneticEngine` operates on an abstract representation of your domain problem using its own domain language we can term the 'Genome'. To bridge the gap between your domain and radiate's, we use a `Codec` - encoder-decoder. A `Codec` is a mechanism that encodes and decodes genetic information between the 'problem space' (your domain) and the 'solution space' (Radiate's internal representation).

Essentially, this is a component that defines how genetic information is structured and represented in your evolutionary algorithm. Think of it as a blueprint that tells the algorithm:

- What type of data you're evolving (numbers, characters, etc.)
- How that data is organized (single values, arrays, matrices, etc.)
- Any other chromosome or gene level information needed for the algorithm to work effectively.

## Why Do We Need Codecs?

In genetic algorithms, we need to represent potential solutions to our problem in a way that can be:

1. **Evolved**: Modified through operations like mutation and crossover
2. **Evaluated**: Tested to see how good the solution is
3. **Consistent**: Able to be encoded to chromosomes and genes which the engine can understand and operate on, then decoded back into a format that can be used in the real-world problem (e.g., your fitness function).

For example, if you're evolving neural network weights, you need to:

- Represent the weights as numbers
- Organize them in the correct structure (matrices for layers)
- Keep them within reasonable ranges (e.g., between -1 and 1)
  
See [this example](https://github.com/pkalivas/radiate/blob/master/examples/simple-nn/src/main.rs) for a simple neural network evolution using Radiate.

## How Codecs Fit Into the Genetic Algorithm

Here's a simple breakdown of how codecs work in the evolution process:

1. **Initialization**: When you create a population, the codec defines how each individual's genetic information is structured and created within the population. For example, if you're evolving a list of floating-point numbers, the codec will specify how many numbers, their ranges, and how they are represented.
2. **Evaluation**: Your fitness function receives the decoded values in a format you can work with and have possibly defined.

## Types of Codecs

Radiate provides several codec types out of the box that should be able to cover most use cases. Each codec type is designed to handle specific data types and structures, making it easier to evolve solutions for various problems. They include:

### 1. FloatCodec
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

    ```rust
    use radiate::*;

    // single float parameter
    let codec_scalar = FloatCodec::scalar(-1.0..1.0).with_bounds(-10.0..10.0);      

    // vector of 5 floats
    let codec_vector = FloatCodec::vector(5, -1.0..1.0).with_bounds(-10.0..10.0);   

    // 3x2 matrix of floats
    let codec_matrix = FloatCodec::matrix(3, 2, -0.1..0.1).with_bounds(-1.0..1.0);  
    ```

### 2. IntCodec
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

    The type of int can be specified as `i8`, `i16`, `i32`, `i64`, `i128` or `u8`, `u16`, `u32`, `u64`, `u128` depending on your needs.

    ```rust
    use radiate::*;

    // single float parameter
    let codec_scalar = IntCodec::scalar(-1..1).with_bounds(-10..10);

    // vector of 5 floats - specify the int type
    let codec_vector = IntCodec::<i128>::vector(5, -1..1).with_bounds(-10..10);

    // 3x2 matrix of floats
    let codec_matrix = IntCodec::matrix(3, 2, -1..1).with_bounds(-10..10);
    ```

### 3. CharCodec
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

    ```rust
    use radiate::*;

    // vector of 5 chars - specify the char set
    let codec_vector = CharCodec::vector(5).with_char_set("abcdefghijklmnopqrstuvwxyz");

    // 3x2 matrix of chars
    let codec_matrix = CharCodec::matrix(3, 2);
    ```

### 4. BitCodec
Use this when you need to evolve binary data. Each `Gene` is a `BitGene` where the `Allele`, or value being evolved, is a bool. Ideal for:

- Binary optimization problems
- Feature selection
- Boolean configurations
- Subset selection problems (e.g., Knapsack problem)

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # For a list of parameters
    codec = rd.BitCodec.vector(5)

    # For a matrix of bools
    codec = rd.BitCodec.matrix(shape=(3, 2))
    # -- or --
    # supply a list of shapes for jagged matrices e.g. matrix with three rows (chromosomes) and two columns (genes) - use the default char_set
    codec = rd.BitCodec.matrix([2, 2, 2])
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // vector of 5 chars - specify the char set
    let codec_vector = BitCodec::vector(5);

    // 3x2 matrix of chars
    let codec_matrix = BitCodec::matrix(3, 2);
    ```

### 5. SubSetCodec
For when you need to optimize a subset or smaller collection from a larger set. Underneath the hood, the `SubSetCodec` uses a `BitCodec` to represent the selection of items. This codec allows you to evolve a selection of items from a larger pool, where each gene represents whether an item is included (1) or excluded (0) in the subset.

- Feature selection in machine learning
- Knapsack problem
- Combinatorial optimization

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        This codec is currently under construction and not yet available in the Python API.

=== ":fontawesome-brands-rust: Rust"

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

    // encoding for this subset will produce a genotype with a single BitChromosome while decoding will return
    // a Vec<Arc<Item>> of the selected items.
    let genotype = subset_codec.encode();           // Genotype<BitChromosome>
    let decoded = subset_codec.decode(&genotype);   // Vec<Arc<Item>>
    ```

### 6. PermutationCodec
The `PermutationCodec<T>` ensures that each gene in the chromosome is a unique item from the set. Use this when you need to evolve permutations of a set of items. This codec is particularly useful for problems where the order of items matters, such as:

- Traveling Salesman Problem (TSP)
- Job scheduling
- Sequence alignment

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        This codec is currently under construction and not yet available in the Python API.

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    let codec: PermutationCodec<usize> = PermutationCodec::new((0..10).collect());

    // Encode a genotype of Genotype<PermutationChromosome> and decode to a Vec<usize> where each usize is a unique index
    // from the original value_range.
    // This will ensure that the permutation is valid and does not contain duplicates.
    let genotype: Genotype<PermutationChromosome<usize>> = codec.encode();
    let decoded: Vec<usize> = codec.decode(&genotype);

    ```

### 7. FnCodec
The `FnCodec` is a flexible codec that allows you to define custom encoding and decoding functions for your problem. This is particularly useful when your solution space does not fit neatly into the other codec types or when you need to handle complex data structures. It allows you to specify how to encode and decode your genetic information using user-defined functions. This codec is ideal for:

- Complex data structures that don't fit into standard codecs
- Custom encoding/decoding logic
- Problems where the representation is not easily defined by simple types

=== ":fontawesome-brands-python: Python"

    !!! warning ":construction: Under Construction :construction:"

        This codec is currently under construction and not yet available in the Python API.

=== ":fontawesome-brands-rust: Rust"

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

## A Simple Example

Let's look at a basic example of how to use the `Codec` for evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`.

=== ":fontawesome-brands-python: Python"

    ```python
    from typing import List
    import radiate as rd

    # Define a fitness function that uses the decoded values
    def fitness_function(individual: List[float]) -> float:    
        # Calculate how well these parameters fit your data
        return calculate_error(individual[0], individual[1])  # Your error calculation here

    # Create a codec for two parameters (a and b)
    codec = rd.FloatCodec.vector(
        length=2,                   # We need two parameters: a and b
        value_range=(-1.0, 1.0),    # Start with values between -1 and 1
        bound_range=(-10.0, 10.0)   # Allow evolution to modify the values between -10 and 10
    )

    # Create the evolution engine
    engine = rd.EvolutionEngine(
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

    let mut engine = GeneticEngine::builder()
        .codec(FloatCodec::vector(2, -1.0..1.0).with_bounds(-10.0..10.0))   // a and b
        .fitness_function(|individual: Vec<f32>| {
            // Calculate how well these parameters fit your data
            calculate_error(individual[0], individual[1])                   // Your error calculation here
        })
        .build();

    // Run the engine
    let result = engine.run(|generation| {
        generation.index() >= 1000 || generation.score().as_f32() <= 0.01
    });
    ```

## Best Practices

1. **Start Simple**: Begin with a simple codec structure and expand as needed
2. **Choose Appropriate Ranges (IntCodec & FloatCodec)**:
   >- `value_range`: Set this to reasonable initial values
   >- `bound_range`: Set this to the valid range for your problem
3. **Match Your Problem**: Choose the codec type that best represents your solution space
4. **Consider Structure**: Use the appropriate configuration (scalar/vector/matrix) for your problem

## Common Pitfalls to Avoid

1. **Too Wide Ranges**: Starting with very wide value ranges can make evolution slower
2. **Too Narrow Bounds**: Restrictive bound ranges might prevent finding optimal solutions
3. **Mismatched Structure**: Using the wrong codec structure can make it impossible to represent valid solutions

## Next Steps

Now that you understand codecs, you can:

1. Define your problem's solution space
2. Choose the appropriate codec type and structure
3. Set up your evolution engine
4. Define your fitness function to work with the decoded values

Remember: The codec is your bridge between the genetic algorithm's internal representation and your problem's solution space. Choose it wisely!


!!! warning ":construction: Under Construction :construction:"

    These docs are under construction - anything below this point is subject to change and is currently being worked on.


___
### Build your own Codec

??? example "FloatCodec"

    Let's take a look at a simplified version of Raditate's built-in `FloatCodec` which encodes and decodes floating-point numbers. The `FloatCodec` takes in the number of chromosomes, number of genes per chromosome, the max allele value, and the min allele value. The `encode` method creates a new `Genotype` of `FloatChromosomes` with `FloatGenes` that have random alleles between the max and min. The `decode` method takes a `Genotype` and returns a `Vec<Vec<f32>>` of the gene values.

    ```rust
    use std::ops::Range;
    use radiate::*;

    pub struct FloatCodec {
        pub num_chromosomes: usize,
        pub num_genes: usize,
        pub min: f32,
        pub max: f32,
        pub lower_bound: f32,
        pub upper_bound: f32,
    }

    impl FloatCodec {
        /// Create a new `FloatCodec` with the given number of chromosomes, genes, min, and max values.
        /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
        pub fn new(num_chromosomes: usize, num_genes: usize, range: Range<f32>) -> Self {
            let (min, max) = (range.start, range.end);
            FloatCodec {
                num_chromosomes,
                num_genes,
                min,
                max,
                lower_bound: f32::MIN,
                upper_bound: f32::MAX,
            }
        }
    }

    impl Codec<FloatChromosome, Vec<Vec<f32>>> for FloatCodec {
        fn encode(&self) -> Genotype<FloatChromosome> {
            Genotype {
                chromosomes: (0..self.num_chromosomes)
                    .map(|_| {
                        FloatChromosome {
                            genes: (0..self.num_genes)
                                .map(|_| {
                                    FloatGene::from((self.min..self.max, self.lower_bound..self.upper_bound))
                                })
                                .collect::<Vec<FloatGene>>(),
                        }
                    })
                    .collect::<Vec<FloatChromosome>>(),
            }
        }

        fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<Vec<f32>> {
            genotype
                .iter()
                .map(|chromosome| {
                    chromosome
                        .iter()
                        .map(|gene| *gene.allele())
                        .collect::<Vec<f32>>()
                })
                .collect::<Vec<Vec<f32>>>()
        }
    }
    ```

    Lets take a look at how we can use the `FloatCodec` to encode and decode a `Genotype` of `FloatChromosomes`
    with 2 chromosomes and 3 genes per chromosome:

    ```rust
    fn main() {
        let codec = FloatCodec::new(2, 3, 0.0..1.0);

        let genotype: Genotype<FloatChromosome> = codec.encode();
        let decoded: Vec<Vec<f32>> = codec.decode(&genotype);
    }
    ```

    The `genotype` in this case will look something like this:

    ``` rust
    Genotype {
        chromosomes: [
            FloatChromosome {
                genes: [
                    FloatGene { allele: 0.123, min: 0.0, max: 1.0, lower_bound: f32::MIN, upper_bound: f32::MAX },
                    FloatGene { allele: 0.456, min: 0.0, max: 1.0, lower_bound: f32::MIN, upper_bound: f32::MAX }, 
                    FloatGene { allele: 0.789, min: 0.0, max: 1.0, lower_bound: f32::MIN, upper_bound: f32::MAX },
                ],
            },
            FloatChromosome {
                genes: [
                    FloatGene { allele: 0.321, min: 0.0, max: 1.0, lower_bound: f32::MIN, upper_bound: f32::MAX },
                    FloatGene { allele: 0.654, min: 0.0, max: 1.0, lower_bound: f32::MIN, upper_bound: f32::MAX },
                    FloatGene { allele: 0.987, min: 0.0, max: 1.0, lower_bound: f32::MIN, upper_bound: f32::MAX },
                ],
            },
        ],
    }
    ```

    And the `decoded` is a `Vec<Vec<f32>>` of the gene values and is what will be passed to the `fitness_fn`:

    ``` rust
    [
        [0.123, 0.456, 0.789],
        [0.321, 0.654, 0.987],
    ]
    ```

??? example "FnCodec - NQueens" 
    
    ```rust
    use radiate::*;

    const N_QUEENS: usize = 8;

    fn main() {
        // this is a simple example of the NQueens problem.
        // The resulting codec type will be FnCodec<IntChromosome<i8>, Vec<i8>>.
        let codec = FnCodec::new()
            .with_encoder(|| {
                Genotype::from_chromosomes(vec![IntChromosome {
                    genes: (0..N_QUEENS)
                        .map(|_| IntGene::from(0..N_QUEENS as i8))
                        .collect(),
                }])
            })
            .with_decoder(|genotype| {
                genotype.chromosomes[0]
                    .genes
                    .iter()
                    .map(|g| *g.allele())
                    .collect::<Vec<i8>>()
            });
        // encode and decode
        let genotype = codec.encode(); // Genotype<IntChromosome<i8>>
        let decoded = codec.decode(&genotype); // Vec<i8>
    }
    ```

??? example "NQueens"

    A simple struct to represent the NQueens problem.
    ```rust
    use radiate::*;

    #[derive(Clone, Debug, PartialEq)]
    struct NQueens(Vec<i32>);
    ```

    A Codec for the NQueens problem.
    ```rust
    struct NQueensCodec {
        size: i32,
    }
    ```

    Implement the Codec trait for the NQueensCodec. The `encode` function creates a `Genotype`
    with a single chromosome of `size` genes. The `decode` function creates a `NQueens` from the
    `Genotype`.
    ```rust
    impl Codec<IntChromosome<i32>, NQueens> for NQueensCodec {
        fn encode(&self) -> Genotype<IntChromosome<i32>> {
            let genes = (0..self.size).map(|_| IntGene::from(0..self.size)).collect();
            let chromosomes = vec![IntChromosome { genes }];
            Genotype::from_chromosomes(chromosomes)
        }

        fn decode(&self, genotype: &Genotype<IntChromosome<i32>>) -> NQueens {
            NQueens(genotype.chromosomes[0].iter().map(|g| *g.allele()).collect())
        }
    }
    ```
    Create a new NQueensCodec with a size of 5.
    ```rust
    let codec = NQueensCodec { size: 5 };
    ```

    encode a new Genotype of IntGenes with a size of 5. The result will be a genotype with a single chromosome with 5 genes.
    The genes will have a min value of 0, a max value of 5, an upper_bound of 5, and a lower_bound of 0
    The alleles will be random values between 0 and 5. It will look something like:
    ```text
    Genotype {
        chromosomes: [
            IntChromosome<i32> {
                genes: [
                    IntGene { allele: 3, min: 0, max: 5, ... },
                    IntGene { allele: 7, min: 0, max: 5, ... },
                    IntGene { allele: 1, min: 0, max: 5, ... },
                    IntGene { allele: 5, min: 0, max: 5, ... },
                    IntGene { allele: 2, min: 0, max: 5, ... },
                ]
            }
        ]
    }
    ```
    ```rust
    let genotype = codec.encode();
    ```
    decode the genotype to a NQueens. The result will be a NQueens struct with a Vec<i32> of 8 random values between 0 and 8.
    It will look something like:
    ```text
    NQueens([3, 7, 1, 5, 2])
    ```
    ```rust
    let nqueens = codec.decode(&genotype);
    ```