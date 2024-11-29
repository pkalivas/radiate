As with any specialized library, the core components operate on their own domain language, Radiate is no exception. The domain language of Radiate can be called the Genome and includes the: `Population`, `Phenotype`, `Genotype`, `Chromosome`, `Gene`, and `Allele`. Each of these components is a building block of the genetic information that is used by the GeneticEngine. 

---

### Allele
: The `Allele` is the smallest unit of genetic information in Radiate. It is a single value that can be used to represent a trait or characteristic of an individual. For example, an `Allele` could represent a single bit in a binary string, a single character in a string, or a single number in a list of numbers. At its most basic level, an `Allele` is the "atom" of genetic information that is used to express the genetic makeup of an individual.

___
### Gene
: The `Gene` is a wrapper around an `Allele` that provides additional functionality for working with genetic information. A `Gene` can be thought of as a container for an `Allele` that allows the `Allele` to operate within the context of the Genome. For example, the `FloatGene` struct is a `Gene` that contains a floating-point number as its `Allele`. Radiate provides a number of built-in `Gene` types that can be used to represent different types of genetic information. However, custom `Gene` can also be defined to represent unique types of genetic information.

 : Certain `Genes` have additional functionality that allows them to be manipulated in specific ways, such as the `FloatGene` and `IntGene<I>` which implement the `NumericGene` and  `BoundedGene` traits respectively. The `NumericGene` trait provides methods for performing arithmetic operations on the `Gene`, while the `BoundedGene` trait provides methods for clamping the value of the `Gene` to a specified range if genetic operations result in values outside of the desired range.

    !!! info "Core Library `Gene` Implementations"

        === "BitGene"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct BitGene {
                allele: bool,
            }
            ```

            * **Allele**: `bool`
            * **Description**: Represents a single bit `true`/`false`
            * **Implements**: `Gene`

        === "FloatGene"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct FloatGene {
                pub allele: f32,
                pub min: f32,
                pub max: f32,
                pub upper_bound: f32,
                pub lower_bound: f32,
            }
            ```

            * **Allele**: `f32`
            * **Description**: Represents a single floating-point number
            * **Implements**: `Gene`, `NumericGene`, `BoundedGene`

        === "IntGene"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct IntGene<T: Integer<T>>
            where
                Standard: rand::distributions::Distribution<T>,
            {
                pub allele: T,
                pub min: T,
                pub max: T,
                pub upper_bound: T,
                pub lower_bound: T,
            }
            ```

            * **Allele**: `I` where `I` implements `Integer<I>`. `Integer` is a trait in Radiate and is implemented for `i8`, `i16`, `i32`, `i64`, `i128`.
            * **Description**: Represents a single integer number
            * **Implements**: `Gene`, `NumericGene`, `BoundedGene`
    
        === "CharGene"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct CharGene {
                pub allele: char,
            }
            ```

            * **Allele**: `char`
            * **Description**: Represents a single character
            * **Implements**: `Gene`
    
        === "PermutationGene"

            ```rust
            #[derive(Debug, Clone, PartialEq)]
            pub struct PermutationGene<A: PartialEq + Clone> {
                pub index: usize,
                pub alleles: Arc<Vec<A>>,
            }
            ```

            * **Allele**: `A`
            * **Description**: Given a list of `A`, represents a single value of the list
            * **Implements**: `Gene`

    User defined `Gene` types can be implemented by implementing the `Gene` trait.


___
### Chromosome
: Each `Gene` is contained within a `Chromosome` and as such, each `Gene` has its own `Chromosome`.
The `Chromosome` is a collection of `Genes` that represent a part or the whole of the genetic information of an individual. A `Chromosome` can be thought of as a "chunk" or vector of genetic information. For example, a `Chromosome` could represent a sequence of numbers, a string of characters, or a set of binary values among other things. The decision to a defined `Chromosome` for each `Gene` was made to allow for more flexibility in the genetic information that can be represented. 

    !!! info "Core library `Chromosome` implementations"

        === "BitChromosome"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct BitChromosome {
                pub genes: Vec<BitGene>,
            }
            ```

            * **Gene Type**: `BitGene`
            * **Description**: Represents a sequence of binary values

        === "FloatChromosome"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct FloatChromosome {
                pub genes: Vec<FloatGene>,
            }
            ```

            * **Gene Type**: `FloatGene`
            * **Description**: Represents a sequence of floating-point numbers (f32)

        === "IntChromosome"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct IntChromosome<I: Integer<I>>
            where
                Standard: rand::distributions::Distribution<I>,
            {
                pub genes: Vec<IntGene<I>>,
            } 
            ```

            * **Gene Type**: `IntGene<I>` where `I` implements `Integer<I>`. `Integer` is a trait in Radiate and is implemented for `i8`, `i16`, `i32`, `i64`, `i128`.
            * **Description**: Represents a sequence of integer numbers

        === "CharChromosome"

            ```rust
            #[derive(Clone, PartialEq)]
            pub struct CharChromosome {
                pub genes: Vec<CharGene>,
            }
            ```

            * **Gene Type**: `CharGene`
            * **Description**: Represents a sequence of characters

        === "PermutationChromosome"

            ```rust
            #[derive(Debug, Clone, PartialEq)]
            pub struct PermutationChromosome<A: PartialEq + Clone> {
                pub genes: Vec<PermutationGene<A>>,
                pub alleles: Arc<Vec<A>>,
            }
            ```

            * **Gene Type**: `PermutationGene<A>`
            * **Description**: Represents a sequence of unique values from a list of `A`.

    For user defined `Chromosome` types, the `Chromosome` trait can be implemented.

___
### Genotype
: The `Genotype` is a collection of `Chromosomes` that represent the complete genetic makeup of an individual. A `Genotype` can be thought of as a "blueprint" for an individual that contains all of the genetic information necessary to fully express the traits and characteristics of that individual. Because the `Genotype` is a collection of `Chromosomes`, it can be used to represent complex genetic information that is composed of multiple parts. It can be thought of as a "matrix" of `Genes` where each row is a `Chromosome`. For example, a `Genotype` of `FloatChromosome`s can be thought of as a `Vec<Vec<f32>>` or a matrix of floating-point numbers.

___
### Phenotype
: In Radiate, the `Phenotype` is the primary interface between the GeneticEngine and the individuals that it is evolving. It is responsible for managing the genetic information of the individual, evaluating the fitness of the individual, and providing a way for the `GeneticEngine` to interact with the individual. The `Phenotype` is the "body" of the individual that the `GeneticEngine` is evolving, and it is the main data structure that the `GeneticEngine` operates on.

___
## Population
: The `Population` is a collection of `Phenotypes` that represents a group of individuals that are being evolved by the `GeneticEngine`. The `Population` is the main data structure that the `GeneticEngine` operates on, and it is responsible for managing the individuals in the population, evaluating their fitness, and evolving them over time. The `Population` is the "ecosystem" in which the genetic algorithm operates, and it is the primary interface between the `GeneticEngine` and the individuals that it is evolving.
