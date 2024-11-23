use super::genome::genes::gene::Gene;
use super::genome::genotype::Genotype;
use super::genome::phenotype::Phenotype;
use super::genome::population::Population;

pub mod bit_codex;
pub mod char_codex;
pub mod float_codex;
pub mod int_codex;
pub mod subset_codex;

pub use bit_codex::*;
pub use char_codex::*;
pub use float_codex::*;
pub use int_codex::*;
pub use subset_codex::*;

/// The `Codex` is a core concept in Radiate, as it allows for the encoding and decoding from 
/// a `Genotype` to the type `T` (commonly called Phenotype in biology) that is being optimized.
/// 
/// In order to have a vaid `GeneticEngine`, a `Codex` must be supplied. In a sense, the encoding is the 
/// 'domain language' of the `GeneticEngine`. It is the way that the `GeneticEngine` interacts with the
/// problem space. The `Codex` is responsible for converting to and from this 'domain language'.
/// 
/// # Example
/// ``` rust
/// use radiate::*;
/// 
/// // A simple struct to represent the NQueens problem.
/// #[derive(Clone, Debug, PartialEq)]
/// struct NQueens(Vec<i32>);
/// 
/// // A Codex for the NQueens problem.
/// struct NQueensCodex {
///    size: i32,
/// }
/// 
/// // Implement the Codex trait for the NQueensCodex. The `encode` function creates a `Genotype`
/// // with a single chromosome of `size` genes. The `decode` function creates a `NQueens` from the
/// // `Genotype`.
/// impl Codex<IntGene<i32>, i32, NQueens> for NQueensCodex {
///     fn encode(&self) -> Genotype<IntGene<i32>, i32> {
///         let genes = (0..self.size).map(|_| IntGene::from_min_max(0, self.size)).collect();
///         let chromosomes = vec![Chromosome::from_genes(genes)];
///         Genotype::from_chromosomes(chromosomes)
///     }
/// 
///     fn decode(&self, genotype: &Genotype<IntGene<i32>, i32>) -> NQueens {
///         NQueens(genotype.chromosomes[0].genes.iter().map(|g| *g.allele()).collect())
///     }
/// }
/// 
/// // Create a new NQueensCodex with a size of 5.
/// let codex = NQueensCodex { size: 5 };
/// 
/// // encode a new Genotype of IntGenes with a size of 5. The result will be a genotype with a single chromosome with 5 genes.
/// // The genes will have a min value of 0, a max value of 5, an upper_bound of i32::Max, and a lower_bound of i32::Min.
/// // The alleles will be random values between 0 and 5. It will look something like:
/// // Genotype {
/// //     chromosomes: [
/// //         Chromosome {
/// //             genes: [
/// //                 IntGene { allele: 3, min: 0, max: 5, ... },
/// //                 IntGene { allele: 7, min: 0, max: 5, ... },
/// //                 IntGene { allele: 1, min: 0, max: 5, ... },
/// //                 IntGene { allele: 5, min: 0, max: 5, ... },
/// //                 IntGene { allele: 2, min: 0, max: 5, ... },
/// //             ]
/// //         }
/// //     ]
/// // }
/// let genotype = codex.encode();
/// 
/// // decode the genotype to a NQueens. The result will be a NQueens struct with a Vec<i32> of 8 random values between 0 and 8.
/// // It will look something like:
/// // NQueens([3, 7, 1, 5, 2])
/// let nqueens = codex.decode(&genotype);
/// ```
/// 
/// # Type Parameters
/// - `G`: The type of gene used in the genetic algorithm, which must implement the `Gene` trait.
/// - `A`: The type of the allele associated with the gene - the gene's "expression".
/// - `T`: The type of the Phenotype that is being optimized.
pub trait Codex<G, A, T>
where
    G: Gene<G, A>,
{
    fn encode(&self) -> Genotype<G, A>;

    fn decode(&self, genotype: &Genotype<G, A>) -> T;

    /// Spawn a new instance of `T` from the `Codex`. This will encode `num` new `Genotype`s and then
    /// decode it to a new instance of `T`.
    fn spawn(&self, num: i32) -> Vec<T> {
        (0..num)
            .into_iter()
            .map(|_| self.decode(&self.encode()))
            .collect::<Vec<T>>()
    }

    /// Spawn a new instance of `Genotype<G, A>` from the `Codex`. This will encode `num` a new `Genotype`s.
    fn spawn_genotypes(&self, num: i32) -> Vec<Genotype<G, A>> {
        (0..num)
            .into_iter()
            .map(|_| self.encode())
            .collect::<Vec<Genotype<G, A>>>()
    }

    /// Spawn a new instance of `Population<G, A>` from the `Codex`. This will encode `num` a new `Genotype`s
    fn spawn_population(&self, num: i32) -> Population<G, A> {
        (0..num)
            .into_iter()
            .map(|_| Phenotype::from_genotype(self.encode(), 0))
            .collect::<Population<G, A>>()
    }
}
