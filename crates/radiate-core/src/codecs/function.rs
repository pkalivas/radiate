use crate::{Chromosome, Codec, Genotype};
use std::sync::Arc;

/// A `Codec` that uses functions to encode and decode a `Genotype` to and from a type `T`.
/// Most of the other codecs in this module are more specialized and are used to create `Genotypes` of specific types of `Chromosomes`.
/// This one, however, is more general and can be used to create `Genotypes` of any type of `Chromosome`.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// const N_QUEENS: usize = 8;
///
/// fn main() {
///     // this is a simple example of the NQueens problem.
///     // The resulting codec type will be FnCodec<IntChromosome<i8>, Vec<i8>>.
///     let codec = FnCodec::new()
///         .with_encoder(|| {
///             Genotype::from(IntChromosome::new((0..N_QUEENS)
///                     .map(|_| IntGene::from(0..N_QUEENS as i8))
///                     .collect(),
///             ))
///         })
///         .with_decoder(|genotype| {
///             genotype[0]
///                 .genes()
///                 .iter()
///                 .map(|g| *g.allele())
///                 .collect::<Vec<i8>>()
///         });
///
///     // encode and decode
///     let genotype: Genotype<IntChromosome<i8>> = codec.encode();
///     let decoded: Vec<i8> = codec.decode(&genotype);
/// }
/// ```
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
/// - `T`: The type that the genotype will be decoded to.
#[derive(Default, Clone)]
pub struct FnCodec<C: Chromosome, T> {
    encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    decoder: Option<Arc<dyn Fn(&Genotype<C>) -> T>>,
}

impl<C: Chromosome, T> FnCodec<C, T> {
    pub fn new() -> Self {
        FnCodec {
            encoder: None,
            decoder: None,
        }
    }

    pub fn with_encoder<F>(mut self, encoder: F) -> Self
    where
        F: Fn() -> Genotype<C> + 'static,
    {
        self.encoder = Some(Arc::new(encoder));
        self
    }

    pub fn with_decoder<F>(mut self, decoder: F) -> Self
    where
        F: Fn(&Genotype<C>) -> T + 'static,
    {
        self.decoder = Some(Arc::new(decoder));
        self
    }
}

impl<C: Chromosome, T> Codec<C, T> for FnCodec<C, T> {
    fn encode(&self) -> Genotype<C> {
        match &self.encoder {
            Some(encoder) => encoder(),
            None => panic!("Encoder function is not set"),
        }
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        match &self.decoder {
            Some(decoder) => decoder(genotype),
            None => panic!("Decoder function is not set"),
        }
    }
}
