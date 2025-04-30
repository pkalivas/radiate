use super::genome::genotype::Genotype;

pub mod bit;
pub mod char;
pub mod float;
pub mod function;
pub mod int;
pub mod permutation;
pub mod subset;

use crate::Chromosome;
pub use bit::BitCodex;
pub use char::CharCodex;
pub use float::FloatCodex;
pub use function::FnCodex;
pub use int::IntCodex;
pub use permutation::PermutationCodex;
pub use subset::SubSetCodex;

/// The `Codex` is a core concept in Radiate, as it allows for the encoding and decoding from
/// a `Genotype` to the type `T` (commonly called Phenotype in biology) that is being optimized.
///
/// In order to have a valid `GeneticEngine`, a `Codex` must be supplied. In a sense, the encoding is the
/// 'domain language' of the `GeneticEngine`. It is the way that the `GeneticEngine` interacts with the
/// problem space. The `Codex` is responsible for converting to and from this 'domain language'.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
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
/// impl Codex<IntChromosome<i32>, NQueens> for NQueensCodex {
///     fn encode(&self) -> Genotype<IntChromosome<i32>> {
///         let genes = (0..self.size).map(|_| IntGene::from(0..self.size)).collect();
///         let chromosomes = vec![IntChromosome { genes }];
///         Genotype::new(chromosomes)
///     }
///
///     fn decode(&self, genotype: &Genotype<IntChromosome<i32>>) -> NQueens {
///         NQueens(genotype[0].iter().map(|g| *g.allele()).collect())
///     }
/// }
///
/// // Create a new NQueensCodex with a size of 5.
/// let codex = NQueensCodex { size: 5 };
///
/// // encode a new Genotype of IntGenes with a size of 5. The result will be a genotype with a single chromosome with 5 genes.
/// // The genes will have a min value of 0, a max value of 5, an upper_bound of 5, and a lower_bound of 0.
/// // The alleles will be random values between 0 and 5. It will look something like:
/// // Genotype {
/// //     chromosomes: [
/// //         IntChromosome<i32> {
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
/// - `C`: The type of the Chromosome that is being optimized - the 'problem space'.
/// - `T`: The type of the Phenotype that is being optimized the expression of the 'problem space'.
///
pub trait Codex<C: Chromosome, T> {
    fn encode(&self) -> Genotype<C>;

    fn decode(&self, genotype: &Genotype<C>) -> T;
}
