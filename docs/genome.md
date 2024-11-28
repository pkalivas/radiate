As with any specialized library, the core components operate on their own domain language, Radiate is no exception. The domain language of Radiate can be called the Genome and includes the: `Population`, `Phenotype`, `Genotype`, `Chromosome`, `Gene`, and `Allele`. Each of these components is a building block of the genetic information that is used by the GeneticEngine. 

---

### Allele
The `Allele` is the smallest unit of genetic information in Radiate. It is a single value that can be used to represent a trait or characteristic of an individual. For example, an `Allele` could represent a single bit in a binary string, a single character in a string, or a single number in a list of numbers. At its most basic level, an `Allele` is the "atom" of genetic information that is used to express the genetic makeup of an individual.

### Gene
The `Gene` is a wrapper around an `Allele` that provides additional functionality for working with genetic information. A `Gene` can be thought of as a container for an `Allele` that allows the `Allele` to operate within the context of the Genome. For example, the `FloatGene` struct is a `Gene` that contains a floating-point number as its `Allele`. Radiate provides a number of built-in `Gene` types that can be used to represent different types of genetic information. However, custom `Gene` can also be defined to represent unique types of genetic information.

 Certain `Genes` have additional functionality that allows them to be manipulated in specific ways, such as the `FloatGene` and `IntGene<I>` which implement the `NumericGene` and  `BoundedGene` traits respectively. The `NumericGene` trait provides methods for performing arithmetic operations on the `Gene`, while the `BoundedGene` trait provides methods for clamping the value of the `Gene` to a specified range if genetic operations result in values outside of the desired range.

Core library `Gene` types:

| Gene Type | Allele(s) | Description | Impls |
|-----------|-----------|-------------| ------ |
| `BinaryGene` | `bool` | Represents a single bit `true`/`false` | `Gene` |
| `FloatGene` | `f32` | Represents a single floating-point number | `Gene`, `NumericGene`, `BoundedGene` |
| `IntegerGene<I: Integer<I>>` | `i8`, `i16`, `i32`, `i64`, `i128` | Represents a single integer number | `Gene`, `NumericGene`, `BoundedGene` |
| `CharGene` | `char` | Represents a single character | `Gene` |
| `PermutationGene<A>` | `A` | Given a list of `A`, represents a single value of the list | `Gene` |

For user defined `Gene` types, the `Gene` trait can be implemented.

### Chromosome
Each `Gene` is contained within a `Chromosome` and as such, each `Gene` has its own `Chromosome`.
The `Chromosome` is a collection of `Genes` that represent a part or the whole of the genetic information of an individual. A `Chromosome` can be thought of as a "chunk" or vector of genetic information. For example, a `Chromosome` could represent a sequence of numbers, a string of characters, or a set of binary values among other things. The decision to a defined `Chromosome` for each `Gene` was made to allow for more flexibility in the genetic information that can be represented. 

Core library `Chromosome` types:

| Chromosome Type | Gene Type | Description |
|-----------------|-----------|-------------|
| `BinaryChromosome` | `BinaryGene` | Represents a sequence of binary values |
| `FloatChromosome` | `FloatGene` | Represents a sequence of floating-point numbers |
| `IntegerChromosome<I: Integer<I>>` | `IntegerGene<I>` | Represents a sequence of integer numbers |
| `CharChromosome` | `CharGene` | Represents a sequence of characters |
| `PermutationChromosome<A>` | `PermutationGene<A>` | Represents a unique sequence of values from a list |

For user defined `Chromosome` types, the `Chromosome` trait can be implemented.

### Genotype
The `Genotype` is a collection of `Chromosomes` that represent the complete genetic makeup of an individual. A `Genotype` can be thought of as a "blueprint" for an individual that contains all of the genetic information necessary to fully express the traits and characteristics of that individual. Because the `Genotype` is a collection of `Chromosomes`, it can be used to represent complex genetic information that is composed of multiple parts. It can be thought of as a "matrix" of `Genes` where each row is a `Chromosome`. For example, a `Genotype` of `FloatChromosome`s can be thought of as a `Vec<Vec<f32>>` or a matrix of floating-point numbers.

### Phenotype
In Radiate, the `Phenotype` is the primary interface between the GeneticEngine and the individuals that it is evolving. It is responsible for managing the genetic information of the individual, evaluating the fitness of the individual, and providing a way for the `GeneticEngine` to interact with the individual. The `Phenotype` is the "body" of the individual that the `GeneticEngine` is evolving, and it is the main data structure that the `GeneticEngine` operates on.

## Population
The `Population` is a collection of `Phenotypes` that represents a group of individuals that are being evolved by the `GeneticEngine`. The `Population` is the main data structure that the `GeneticEngine` operates on, and it is responsible for managing the individuals in the population, evaluating their fitness, and evolving them over time. The `Population` is the "ecosystem" in which the genetic algorithm operates, and it is the primary interface between the `GeneticEngine` and the individuals that it is evolving.
