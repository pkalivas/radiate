
# Ecosystem

---

## Allele

> The basic unit.

The `allele` is the smallest unit of genetic information in Radiate. It is a single value that can be used to represent a trait or characteristic of an individual. For example, an `allele` could represent a single bit in a binary string, a single character in a string, or a single number in a list of numbers. At its most basic level, an `allele` is the "atom" of genetic information that is used to express the genetic makeup of an individual - think of it as the "letter" in a genetic "word". For example, it could be:

- A number (like `42` or `3.14`)
- A character (like `A` or `?`)
- A boolean value (`true`/`false`)
- Any other basic value type

---

## Gene

> The container.

A `Gene` is a wrapper around an `allele` that adds functionality which is compatible with the genetic algorithm. It's like a container that not only holds the value, or `allele`, but also knows how to:

- Create new instances of itself
- Validate itself and its `allele`
- Perform operations on its `allele` like addition, subtraction, or mutation
- Maintain constraints (like value ranges)

Certain `Genes` have additional functionality that allows them to be manipulated in specific ways, such as the `FloatGene` and `IntGene<I>` which implement the `ArithmeticGene`. The `ArithmeticGene` trait provides methods for performing arithmetic operations on the `Gene`. Radiate provides several built-in gene types, however, you can also create custom genes to suit your specific needs. The core built-in genes include:

??? info "FloatGene"

    For evolving floating-point numbers. If the `allele` is not specified, it will be randomly initialized within the `value_range`. If the `value_range` is not specified, it will default to (`-1e10`, `1e10`). If the `bound_range` is not specified, it will default to `value_range`.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create a float gene that can evolve between -1.0 and 1.0 but 
        # must stay within -10.0 to 10.0 during evolution
        gene = rd.gene.float(
            allele=0.5,                   # Current value
            init_range=(-1.0, 1.0),      # Initial range
            bounds=(-10.0, 10.0)     # Evolution bounds
        )
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create a float gene that can evolve between -1.0 and 1.0 but 
        // must stay within -10.0 to 10.0 during evolution
        let gene = FloatGene::new(0.5, -1.0..1.0, -10.0..10.0);

        // Create a float gene with a randomly generated allele between -1.0 and 1.0
        // and bounds between -1.0 and 1.0
        let gene = FloatGene::from(-1.0..1.0)

        // Create a float gene with a randomly generated allele between -1.0 and 1.0 with bounds between -10.0 and 10.0
        let gene = FloatGene::from(-1.0..1.0, -10.0..10.0);

        // Create a float gene with an allele of 0.5 allele between -1.0 and 1.0 with bounds between -10.0 and 10.0
        let gene = FloatGene::from((0.5, -1.0..1.0, -10.0..10.0));
        ```


??? info "IntGene"

    For evolving integer values. If the `allele` is not specified, it will be randomly initialized within the `value_range`. If the `value_range` is not specified, it will default to (`-1e10`, `1e10`). If the `bound_range` is not specified, it will default to `value_range`. The `IntGene` holds a generic type `I` that implements the `Integer<I>` trait, which allows it to work with various integer types such as `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, and `u128`.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create an integer gene that can evolve between -100 and 100
        gene = rd.gene.int(
            allele=42,                     # Current value
            init_range=(-10, 10),        # Initial range
            bounds=(-100, 100)       # Evolution bounds
        )
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create an integer gene that can evolve between -100 and 100
        let gene = IntGene::new(42, -10..10, -100..100);

        // Create an integer gene with a randomly generated allele between -10 and 10 - specify the int type
        let gene = IntGene::<i8>::from(-10..10);

        // Create an integer gene with a randomly generated allele between -10 and 10 with bounds between -100 and 100
        let gene = IntGene::from(-10..10, -100..100);

        // Create an integer gene with an allele of 42 between -10 and 10 with bounds between -100 and 100
        let gene = IntGene::from((42, -10..10, -100..100));
        ``` 

??? info "BitGene"

    For evolving binary values. Radiate uses a `bool` as the allele for `BitGene`, which can be either `True` or `False`. 

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create an bit gene with an allele of True - if the allele isn't specified, it will 
        # be randomly initialized to True or False
        gene = rd.gene.bit(allele=True)
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create an bit gene with a randomly generated allele of true or false.
        let gene = BitGene::new();

        // Create a bit gene with an allele of true
        let gene = BitGene::from(true); 
        ```

??? info "CharGene"

    For evolving character values. The `CharGene` uses a `char` as its allele, which can represent any single Unicode character. If the `allele` is not specified, it will be randomly initialized to a character within the specified `char_set`. If the `char_set` is not specified, it will default to the ASCII printable characters.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create a character gene with an allele of 'A'
        gene = rd.gene.char(allele='A')

        # Create a character gene with a randomly generated allele from the set 'abc'
        gene = rd.gene.char(char_set='abc')  
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create an char gene with a randomly generated allele from the ASCII printable characters
        let gene = CharGene::default();

        // Create a char gene with a char_set of 'abc' of which the allele will be randomly chosen from
        let gene = CharGene::from("abc");
        ```

??? info "PermutationGene"

    For evolving permutations of a set of values. The `PermutationGene` allows you to represent a single value from a list of unique values. It is useful for problems where the order of elements matters, such as the Traveling Salesman Problem.

    === ":fontawesome-brands-python: Python"

        !!! warning ":construction: Under Construction :construction:"

            This Gene is currently under construction and not yet available in the Python API.

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Define a list of alleles the associated genes
        let alleles = Arc::new(vec![1, 2, 3, 4]);
        let genes = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
        ];
        ```

---

## Chromosome

> The Collection.

Each `Gene` is contained within a `Chromosome` and as such, each `Gene` has its own `Chromosome`.
The `Chromosome` is a collection of `Genes` that represent a part or the whole of the genetic information of an individual. A `Chromosome` can be thought of as a "chunk" or vector of genetic information. For example, a `Chromosome` could represent a sequence of numbers, a string of characters, or a set of binary values among other things. The decision to define a `Chromosome` for each `Gene` was made to allow for more flexibility in the genetic information that can be represented. Think of it as a "sentence" made up of multiple "words" (genes). Each chromosome represents a specific part of your solution.

For example, if you're evolving a neural network, you might have:

- One chromosome for the weights of the first layer
- Another chromosome for the weights of the second layer
- Each chromosome contains multiple genes (the individual weights)

Because each `Chromosome` has an associated `Gene`, the built int chromosomes are defined as follows:

??? info "FloatChromosome"

    For evolving a sequence of floating-point numbers. It contains a vector of `FloatGene` instances.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create a float chromosome with 5 genes, each initialized to a random value between -1.0 and 1.0
        chromosome = rd.chromosome.float(
            length=5, 
            init_range=(-1.0, 1.0), 
            bounds=(-10.0, 10.0)
        )
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create a float chromosome with 5 genes, each initialized to a random value between -1.0 and 1.0
        let chromosome = FloatChromosome::from((5, -1.0..1.0));

        let bounded_chromosome = FloatChromosome::from((5, -1.0..1.0, -10.0..10.0));
        ```

??? info "IntChromosome"

    For evolving a sequence of integer values. It contains a vector of `IntGene<I>` instances, where `I` is a generic type that implements the `Integer<I>` trait.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create an integer chromosome with 5 genes, each initialized to a random value between -10 and 10
        chromosome = rd.chromosome.int(
            length=5,
            init_range=(-10, 10),
            bounds=(-100, 100)
        )
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create an integer chromosome with 5 genes, each initialized to a random value between -10 and 10
        let chromosome = IntChromosome::<i32>::from((5, -10..10));

        let bounded_chromosome = IntChromosome::<i32>::from((5, -10..10, -100..100));
        ```

??? info "BitChromosome"

    For evolving a sequence of binary values. It contains a vector of `BitGene` instances.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create a bit chromosome with 5 genes, each initialized to a random value of True or False
        chromosome = rd.chromosome.bit(length=5)
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create a bit chromosome with 5 genes, each initialized to a random value of true or false
        let chromosome = BitChromosome::new(5);
        ```

??? info "CharChromosome"

    For evolving a sequence of character values. It contains a vector of `CharGene` instances.

    === ":fontawesome-brands-python: Python"

        ```python
        import radiate as rd

        # Create a character chromosome with 5 genes, each initialized to a random character from the ASCII printable characters
        chromosome = rd.chromosome.char(length=5)

        chromosome_with_set = rd.chromosome.char(length=5, char_set='abc')
        ```

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Create a character chromosome with 5 genes, each initialized to
        // a random character from the provided char_set
        let chromosome = CharChromosome::new((5, vec!['a', 'b', 'c']));
        let chromosome_with_set = CharChromosome::from((5, "abc"));
        ```

??? info "PermutationChromosome<T>"

    For evolving a sequence of unique values. It contains a vector of `PermutationGene<T>` instances, where `T` is the type of the values in the permutation.

    === ":fontawesome-brands-python: Python"

        !!! warning ":construction: Under Construction :construction:"

            This Chromosome is currently under construction and not yet available in the Python API.

    === ":fontawesome-brands-rust: Rust"

        ```rust
        use radiate::*;

        // Define a list of alleles the associated genes
        let alleles = Arc::new(vec![1, 2, 3, 4]);
        let chromosome = PermutationChromosome::from((4, Arc::clone(&alleles)));
        ```

---

## Genotype

> The Complete Blueprint

The `Genotype` is a collection of `Chromosomes` that represent the complete genetic makeup of an individual. A `Genotype` can be thought of as a "blueprint" for an individual that contains all of the genetic information necessary to fully express the traits and characteristics of that individual.  In essance, the `Genotype` is the "sentence" made up of multiple "words" (chromosomes). Each `Genotype` contains one or more `Chromosomes`, and each `Chromosome` contains one or more `Genes`. It is the "DNA" of the individual that the `GeneticEngine` is evolving.

Because of the typed nature of the `Genotype`, it can only hold a collection of the same type of `Chromosome`. This means that you can have a `Genotype` that contains only `FloatChromosome`s, or only `IntChromosome`s. You cannot have a `Genotype` that contains both `FloatChromosome`s and `IntChromosome`s at the same time - this is by design.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create a genotype with a single FloatChromosome and a 5 FloatGenes
    genotype = rd.Genotype(
        rd.chromosome.float(length=5, init_range=(-1.0, 1.0))
    )

    # Create a genotype with a single FloatChromosome and a single FloatGene with a 
    # randomly generated allele between -1.0 and 1.0
    genotype = rd.Genotype(
        rd.Chromosome([rd.gene.float(init_range=(-1.0, 1.0))])
    )

    # Create a genotype with multiple chromosomes of lengths 5, 15, and 3
    three_chromosome_genotype = rd.Genotype([
        rd.chromosome.float(length=5, init_range=(-1.0, 1.0)),
        rd.chromosome.float(length=15, init_range=(-1.0, 1.0)),
        rd.chromosome.float(length=3, init_range=(-1.0, 1.0))
    ])
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create a genotype with a single FloatChromosome and a 5 FloatGenes 
    let genotype = Genotype::from(FloatChromosome::from((5, -1.0..1.0)));
    // -- or --
    let genotype = Genotype::from(vec![FloatChromosome::new(vec![FloatGene::new(0.1, -1.0..1.0)])]);

    // Create a genotype with multiple chromosomes of lengths 5, 15, and 3
    let three_chromosome_genotype = Genotype::new(vec![
        FloatChromosome::from((5, -1.0..1.0)),
        FloatChromosome::from((15, -1.0..1.0)),
        FloatChromosome::from((3, -1.0..1.0))
    ])

    let genotype_length = three_chromosome_genotype.len(); // 3

    // Get the second chromosome from the genotype
    let second_chromosome = three_chromosome_genotype.get(1).unwrap(); // or use `three_chromosome_genotype[1]`
    let mut second_chromosome_mut = three_chromosome_genotype.get_mut(1).unwrap();

    for chromosome in three_chromosome_genotype.iter() { // or iter_mut()
        // Do something with each chromosome
    }
    ```

---

## Phenotype

> The Living Solution

The `Phenotype` is the representation of an individual in the population that is being evolved by the `GeneticEngine`. It is a concrete implementation of a `Genotype` that includes additional functionality for the individual, such as calculating its fitness score. The `Phenotype` is the "living" version of the `Genotype`, and it is what the `GeneticEngine` interacts with during the evolution process.

The `Phenotype` is responsible for:

- Providing a way for the `GeneticEngine` to interact with the individual
- Holding the genetic information of the individual, including its `Genotype` and fitness or `score`
- Managing the individual's state during the evolution process

You shouldn't need to create a `Phenotype` directly, as the `GeneticEngine` will handle this for you.

---

## Population

> The Community

The `Population` is a collection of `Phenotype`s that represent the current state of the genetic algorithm. It is the "community" of solutions that the `GeneticEngine` is evolving. Its really just a vector of `Phenotype`s. The `Population` is responsible for:

- Sorting individuals based on their fitness scores
- Holding the current individuals that are being evolved
- Providing a way for the `GeneticEngine` to interact with the individuals being evolved

The `Population` is created and managed by the `GeneticEngine`, and you shouldn't need to create a `Population` directly. Instead, you will interact with the `GeneticEngine` to manage the population and evolve the individuals.

---
## Species

> The Diverse Groups

The `Species` is an optional component of the genome system that contains a `Population` of `Phenotype`s that are similar to each other. It is used to group individuals that are similar in some way, such as having similar `Genotype` structures or fitness scores. The `Species` is responsible for:

- Grouping individuals that are similar to each other
- Allowing the `GeneticEngine` to evolve individuals within a specific group
- Providing a way to manage diversity within the population
- Sharing fitness information between individuals in the same species

The `Species` is not required for the genome system to function, but it can be useful for certain types of problems where grouping similar individuals can help improve the evolution process. For different `Species` to be created, your `GeneticEngine` must contain a struct which implements the `Diversity` trait - this will allow the `GeneticEngine` to create and manage `Species` based on the diversity of the individuals in the population. More on this later.

---
## Ecosystem

> The Environment

The `Ecosystem` is the highest level of the genome system and represents the entire environment in which the genetic algorithm operates. It contains zero to many `Species`, each with its own `Population` of `Phenotype`s, and a single `Population` containing all `Phenotype`s. The `Ecosystem` is responsible for:

- Wrapping the entire genetic algorithm environment
- Managing the overall population of individuals
- Optionally Coordinating the interactions between different `Species` and managing their diversity

The `Ecosystem` is created and managed by the `GeneticEngine`, and you shouldn't need to create an `Ecosystem` directly. Instead, you will interact with the `GeneticEngine` to manage the ecosystem and evolve the individuals within it.