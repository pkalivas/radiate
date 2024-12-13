use crate::{Chromosome, Codex, Genotype};

/// A `Codex` that uses functions to encode and decode a `Genotype` to and from a type `T`.
/// Most of the other codexes in this module are more specialized and are used to create `Genotypes` of specific types of `Chromosomes`.
/// This one, however, is more general and can be used to create `Genotypes` of any type of `Chromosome`.
///
/// # Example
/// ``` rust
/// use radiate::*;
///
/// const N_QUEENS: usize = 8;
///
/// fn main() {
///     // this is a simple example of the NQueens problem.
///     // The resulting codex type will be FnCodex<IntChromosome<i8>, Vec<i8>>.
///     let codex = FnCodex::new()
///         .with_encoder(|| {
///             Genotype::from_chromosomes(vec![IntChromosome::from_genes(
///                 (0..N_QUEENS)
///                     .map(|_| IntGene::from_min_max(0, N_QUEENS as i8))
///                     .collect(),
///             )])
///         })
///         .with_decoder(|genotype| {
///             genotype.chromosomes[0]
///                 .genes
///                 .iter()
///                 .map(|g| *g.allele())
///                 .collect::<Vec<i8>>()
///         });
///
///     // encode and decode
///     let genotype = codex.encode(); // Genotype<IntChromosome<i8>>
///     let decoded = codex.decode(&genotype); // Vec<i8>
/// }
/// ```
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type that the genotype will be decoded to.
///
#[derive(Default)]
pub struct FnCodex<C: Chromosome, T> {
    pub encoder: Option<Box<dyn Fn() -> Genotype<C>>>,
    pub decoder: Option<Box<dyn Fn(&Genotype<C>) -> T>>,
}

impl<C: Chromosome, T> FnCodex<C, T> {
    pub fn new() -> Self {
        FnCodex {
            encoder: None,
            decoder: None,
        }
    }

    pub fn with_encoder<F>(mut self, encoder: F) -> Self
    where
        F: Fn() -> Genotype<C> + 'static,
    {
        self.encoder = Some(Box::new(encoder));
        self
    }

    pub fn with_decoder<F>(mut self, decoder: F) -> Self
    where
        F: Fn(&Genotype<C>) -> T + 'static,
    {
        self.decoder = Some(Box::new(decoder));
        self
    }
}

impl<C: Chromosome, T> Codex<C, T> for FnCodex<C, T> {
    fn encode(&self) -> Genotype<C> {
        if self.encoder.is_none() {
            panic!("Encoder function is not set");
        }

        self.encoder.as_ref().unwrap()()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        if self.decoder.is_none() {
            panic!("Decoder function is not set");
        }

        self.decoder.as_ref().unwrap()(genotype)
    }
}
