use super::Codec;
use crate::genome::Gene;
use crate::genome::genotype::Genotype;
use crate::{BitChromosome, Chromosome};

/// A [Codec] for a `Genotype` of `BitGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<bool>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `BitGenes` in the chromosome - the `bool` values.
///
/// # Example
/// ``` rust
/// // In traditional genetic algorithms, a `BitCodec` would be used to create a `Genotype` of `BitGenes`, or a bit string.
/// // This would simply be created by the following:
/// use radiate_core::*;
///
/// // The number of bits (`BitGenes`) in the bit string
/// let length = 10;
///
/// // Create a new matrix `BitCodec` with a single chromosome and `length` genes
/// let codec = BitCodec::matrix(1, length);
///
/// // Create a new `Genotype` of `BitGenes` with a single chromosome and `length` genes
/// let genotype = codec.encode();
///
/// // Decode the `Genotype` to a `Vec<Vec<bool>>`, then get the first chromosome
/// let bit_string: Vec<bool> = codec.decode(&genotype)[0].clone();
/// ```
#[derive(Clone)]
pub struct BitCodec<T = ()> {
    num_chromosomes: usize,
    num_genes: usize,
    _marker: std::marker::PhantomData<T>,
}

impl BitCodec<Vec<Vec<bool>>> {
    pub fn matrix(num_chromosomes: usize, num_genes: usize) -> Self {
        BitCodec {
            num_chromosomes,
            num_genes,
            _marker: std::marker::PhantomData,
        }
    }
}

impl BitCodec<Vec<bool>> {
    pub fn vector(num_genes: usize) -> Self {
        BitCodec {
            num_chromosomes: 1,
            num_genes,
            _marker: std::marker::PhantomData,
        }
    }
}

impl BitCodec<bool> {
    pub fn scalar() -> Self {
        BitCodec {
            num_chromosomes: 1,
            num_genes: 1,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Codec<BitChromosome, Vec<Vec<bool>>> for BitCodec<Vec<Vec<bool>>> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| BitChromosome::new(self.num_genes))
                .collect::<Vec<BitChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<Vec<bool>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<Vec<bool>>>()
    }
}

impl Codec<BitChromosome, Vec<bool>> for BitCodec<Vec<bool>> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| BitChromosome::new(self.num_genes))
                .collect::<Vec<BitChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<bool> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<bool>>()
    }
}

impl Codec<BitChromosome, bool> for BitCodec<bool> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| BitChromosome::new(self.num_genes))
                .collect::<Vec<BitChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> bool {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .next()
            .unwrap_or(false)
    }
}

impl Codec<BitChromosome, Vec<Vec<bool>>> for Vec<BitChromosome> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::from(
            self.iter()
                .map(|chromosome| {
                    chromosome
                        .iter()
                        .map(|gene| gene.new_instance())
                        .collect::<BitChromosome>()
                })
                .collect::<Vec<BitChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<Vec<bool>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<Vec<bool>>>()
    }
}

impl Codec<BitChromosome, Vec<bool>> for BitChromosome {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::from(
            self.iter()
                .map(|gene| gene.new_instance())
                .collect::<BitChromosome>(),
        )
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<bool> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<bool>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Codec;

    #[test]
    fn test_bit_codec() {
        let codec = BitCodec::matrix(2, 3);
        let genotype = codec.encode();
        assert_eq!(genotype.len(), 2);
        assert_eq!(genotype[0].len(), 3);
        assert_eq!(genotype[1].len(), 3);

        let decoded = codec.decode(&genotype);
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].len(), 3);
        assert_eq!(decoded[1].len(), 3);
    }

    #[test]
    fn test_bit_codec_vector() {
        let codec = BitCodec::vector(5);
        let genotype = codec.encode();
        assert_eq!(genotype.len(), 1);
        assert_eq!(genotype[0].len(), 5);

        let decoded = codec.decode(&genotype);
        assert_eq!(decoded.len(), 5);
    }

    #[test]
    fn test_bit_codec_scalar() {
        let codec = BitCodec::scalar();
        let genotype = codec.encode();
        assert_eq!(genotype.len(), 1);
        assert_eq!(genotype[0].len(), 1);

        let decoded = codec.decode(&genotype);

        assert!(vec![true, false].contains(&decoded));
    }
}
