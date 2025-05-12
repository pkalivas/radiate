use super::Codex;
use crate::genome::char::CharGene;
use crate::genome::gene::Gene;
use crate::genome::genotype::Genotype;
use crate::{CharChromosome, Chromosome, char};
use std::sync::Arc;

/// A `Codex` for a `Genotype` of `CharGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `String` from the `Genotype` where the `String`
/// contains the alleles of the `CharGenes` in the chromosome.
#[derive(Clone)]
pub struct CharCodex<T = ()> {
    num_chromosomes: usize,
    num_genes: usize,
    char_set: Arc<[char]>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> CharCodex<T> {
    pub fn with_char_set(mut self, char_set: impl Into<Arc<[char]>>) -> Self {
        self.char_set = char_set.into();
        self
    }
}

impl CharCodex<Vec<Vec<char>>> {
    pub fn matrix(num_chromosomes: usize, num_genes: usize) -> Self {
        CharCodex {
            num_chromosomes,
            num_genes,
            char_set: char::ALPHABET.chars().collect::<Vec<char>>().into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl CharCodex<Vec<char>> {
    pub fn vector(num_genes: usize) -> Self {
        CharCodex {
            num_chromosomes: 1,
            num_genes,
            char_set: char::ALPHABET.chars().collect::<Vec<char>>().into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl Codex<CharChromosome, Vec<Vec<char>>> for CharCodex<Vec<Vec<char>>> {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| CharChromosome {
                    genes: (0..self.num_genes)
                        .map(|_| CharGene::new(Arc::clone(&self.char_set)))
                        .collect::<Vec<CharGene>>(),
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

impl Codex<CharChromosome, Vec<char>> for CharCodex<Vec<char>> {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| CharChromosome {
                    genes: (0..self.num_genes)
                        .map(|_| CharGene::new(Arc::clone(&self.char_set)))
                        .collect::<Vec<CharGene>>(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Codex;
    use crate::genome::gene::Gene;

    #[test]
    fn test_char_codex_matrix() {
        let char_set = "abcde".chars().collect::<Vec<char>>();
        let codex = CharCodex::matrix(3, 5).with_char_set(char_set.clone());
        let genotype = codex.encode();
        assert_eq!(genotype.len(), 3);
        assert_eq!(genotype[0].len(), 5);
        for gene in genotype[0].iter() {
            assert!(gene.char_set.eq(&Arc::from(char_set.clone())));
            assert!(char_set.contains(gene.allele()));
        }
    }

    #[test]
    fn test_char_codex() {
        let codex = CharCodex::vector(5);
        let genotype = codex.encode();
        assert_eq!(genotype.len(), 1);
        assert_eq!(genotype[0].len(), 5);
        for gene in genotype[0].iter() {
            assert!(codex.char_set.contains(gene.allele()));
        }
    }
}
