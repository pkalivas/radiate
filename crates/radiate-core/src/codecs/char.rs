use super::Codec;
use crate::chromosomes::char;
use crate::genome::CharGene;
use crate::genome::Gene;
use crate::genome::genotype::Genotype;
use crate::{CharChromosome, Chromosome};
use std::sync::Arc;

/// A `Codec` for a `Genotype` of `CharGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `String` from the `Genotype` where the `String`
/// contains the alleles of the `CharGenes` in the chromosome.
#[derive(Clone)]
pub struct CharCodec<T = ()> {
    num_chromosomes: usize,
    num_genes: usize,
    char_set: Arc<[char]>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> CharCodec<T> {
    pub fn with_char_set(mut self, char_set: impl Into<Arc<[char]>>) -> Self {
        self.char_set = char_set.into();
        self
    }
}

impl CharCodec<Vec<Vec<char>>> {
    pub fn matrix(num_chromosomes: usize, num_genes: usize) -> Self {
        CharCodec {
            num_chromosomes,
            num_genes,
            char_set: char::ALPHABET.chars().collect::<Vec<char>>().into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl CharCodec<Vec<char>> {
    pub fn vector(num_genes: usize) -> Self {
        CharCodec {
            num_chromosomes: 1,
            num_genes,
            char_set: char::ALPHABET.chars().collect::<Vec<char>>().into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl Codec<CharChromosome, Vec<Vec<char>>> for CharCodec<Vec<Vec<char>>> {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| {
                    CharChromosome::new(
                        (0..self.num_genes)
                            .map(|_| CharGene::new(Arc::clone(&self.char_set)))
                            .collect::<Vec<CharGene>>(),
                    )
                })
                .collect::<Vec<CharChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<CharChromosome>) -> Vec<Vec<char>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<char>>()
            })
            .collect::<Vec<Vec<char>>>()
    }
}

impl Codec<CharChromosome, Vec<char>> for CharCodec<Vec<char>> {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| {
                    CharChromosome::new(
                        (0..self.num_genes)
                            .map(|_| CharGene::new(Arc::clone(&self.char_set)))
                            .collect::<Vec<CharGene>>(),
                    )
                })
                .collect::<Vec<CharChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<CharChromosome>) -> Vec<char> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<char>>()
            })
            .collect::<Vec<char>>()
    }
}

impl Codec<CharChromosome, Vec<Vec<char>>> for Vec<CharChromosome> {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype::from(
            self.iter()
                .map(|chromosome| {
                    chromosome
                        .iter()
                        .map(|gene| gene.new_instance())
                        .collect::<CharChromosome>()
                })
                .collect::<Vec<CharChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<CharChromosome>) -> Vec<Vec<char>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<char>>()
            })
            .collect::<Vec<Vec<char>>>()
    }
}

impl Codec<CharChromosome, Vec<char>> for CharChromosome {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype::from(
            self.iter()
                .map(|gene| gene.new_instance())
                .collect::<CharChromosome>(),
        )
    }

    fn decode(&self, genotype: &Genotype<CharChromosome>) -> Vec<char> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<char>>()
            })
            .collect::<Vec<char>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Codec;
    use crate::genome::Gene;

    #[test]
    fn test_char_codec_matrix() {
        let char_set = "abcde".chars().collect::<Vec<char>>();
        let codec = CharCodec::matrix(3, 5).with_char_set(char_set.clone());
        let genotype = codec.encode();
        assert_eq!(genotype.len(), 3);
        assert_eq!(genotype[0].len(), 5);
        for gene in genotype[0].iter() {
            assert!(gene.char_set().eq(&char_set));
            assert!(char_set.contains(gene.allele()));
        }
    }

    #[test]
    fn test_char_codec_vector() {
        let codec = CharCodec::vector(5);
        let genotype = codec.encode();
        assert_eq!(genotype.len(), 1);
        assert_eq!(genotype[0].len(), 5);
        for gene in genotype[0].iter() {
            assert!(codec.char_set.contains(gene.allele()));
        }
    }
}
