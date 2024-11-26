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
///     let engine = GeneticEngine::from_codex(&codex)
///         .minimizing()
///         .num_threads(10)
///         .offspring_selector(RouletteSelector::new())
///         .alterer(vec![
///             Alterer::MultiPointCrossover(0.75, 2),
///             Alterer::Mutator(0.01),
///         ])
///         .fitness_fn(|genotype: Vec<i8>| {
///             let queens = &genotype;
///             let mut score = 0;
///
///             for i in 0..N_QUEENS {
///                 for j in (i + 1)..N_QUEENS {
///                     if queens[i] == queens[j] {
///                         score += 1;
///                     }
///                     if (i as i8 - j as i8).abs() == (queens[i] - queens[j]).abs() {
///                         score += 1;
///                     }
///                 }
///             }
///
///             Score::from_usize(score)
///         })
///         .build();
///
///     let result = engine.run(|output| output.score().as_usize() == 0);
/// }
/// ```
/// 
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
        self.encoder.as_ref().unwrap()()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.decoder.as_ref().unwrap()(genotype)
    }
}
