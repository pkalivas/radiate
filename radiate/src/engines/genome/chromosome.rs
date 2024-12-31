use std::ops::Range;
use std::sync::Arc;

use super::genes::gene::Gene;
use super::FloatGene;
use super::IntGene;
use super::Integer;
use super::PermutationGene;
use crate::BitGene;
use crate::CharGene;
use crate::Valid;
use rand::distributions::Standard;

/// The `Chromosome` struct represents a collection of `Gene` instances. The `Chromosome` is part of the
/// genetic makeup of an individual. It is a collection of `Gene` instances, it is essentially a
/// light wrapper around a Vec of `Gene`s. The `Chromosome` struct, however, has some additional
/// functionality and terminology that aligns with the biological concept of a chromosome.
///
/// In traditional biological terms, a `Chromosome` is a long DNA molecule with part or all of the
/// genetic material of an organism. The `Chromosome` is the 'genetic' part of the individual that is
/// being evolved by the genetic algorithm.
///
/// We can think of a `Chromosome` as a Vec of structs which implement the `Gene` trait. For example,
/// if we have a `Chromosome` with 3 `Gene`s, it is represented as follows:
/// ```text
/// Chromosome: [Gene, Gene, Gene]
/// ```
///
pub trait Chromosome:
    Clone + PartialEq + Valid + AsRef<[Self::Gene]> + AsMut<[Self::Gene]>
{
    type Gene: Gene;
    /// Retrieves the gene at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the gene to retrieve.
    ///
    /// # Returns
    ///
    /// * A reference to the `FloatGene` at the given index.
    ///
    /// # Example
    /// ```
    /// # use radiate::{FloatGene, FloatChromosome};
    /// let genes = vec![FloatGene::new(-1.0, 1.0)];
    /// let chromosome = FloatChromosome::new(genes);
    /// assert_eq!(chromosome.get_gene(0).allele, -1.0);
    /// ```
    fn get_gene(&self, index: usize) -> &Self::Gene {
        &self.as_ref()[index]
    }

    /// Sets the gene at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the gene to set.
    /// * `gene` - The `FloatGene` to replace the existing gene.
    ///
    /// # Example
    ///
    /// ```
    /// # use radiate::{FloatGene, FloatChromosome};
    /// let genes = vec![FloatGene::new(-1.0, 1.0)];
    /// let mut chromosome = FloatChromosome::new(genes);
    /// chromosome.set_gene(0, FloatGene::new(0.5, 1.0));
    /// assert_eq!(chromosome.get_gene(0).allele, 0.5);
    /// ```
    fn set_gene(&mut self, index: usize, gene: Self::Gene) {
        self.as_mut()[index] = gene;
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn iter(&self) -> std::slice::Iter<Self::Gene> {
        self.as_ref().iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Self::Gene> {
        self.as_mut().iter_mut()
    }
}

/// A `Chromosome` that contains `BitGenes`.
/// A `BitChromosome` is a collection of `BitGenes` that represent the genetic
/// material of an individual in the population.
///
#[derive(Clone, PartialEq)]
pub struct BitChromosome {
    pub genes: Vec<BitGene>,
}

impl Chromosome for BitChromosome {
    type Gene = BitGene;
}

impl Valid for BitChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl AsRef<[BitGene]> for BitChromosome {
    fn as_ref(&self) -> &[BitGene] {
        &self.genes
    }
}

impl AsMut<[BitGene]> for BitChromosome {
    fn as_mut(&mut self) -> &mut [BitGene] {
        &mut self.genes
    }
}

impl From<&[bool]> for BitChromosome {
    fn from(alleles: &[bool]) -> Self {
        let genes = alleles.iter().map(BitGene::from).collect();
        BitChromosome { genes }
    }
}

impl From<Vec<bool>> for BitChromosome {
    fn from(alleles: Vec<bool>) -> Self {
        let genes = alleles.iter().map(BitGene::from).collect();
        BitChromosome { genes }
    }
}

/// A `Chromosome` that contains `CharGenes`.
#[derive(Clone, PartialEq)]
pub struct CharChromosome {
    pub genes: Vec<CharGene>,
}

impl Chromosome for CharChromosome {
    type Gene = CharGene;
}

impl Valid for CharChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl AsRef<[CharGene]> for CharChromosome {
    fn as_ref(&self) -> &[CharGene] {
        &self.genes
    }
}

impl AsMut<[CharGene]> for CharChromosome {
    fn as_mut(&mut self) -> &mut [CharGene] {
        &mut self.genes
    }
}

impl From<&'static str> for CharChromosome {
    fn from(alleles: &'static str) -> Self {
        let genes = alleles.chars().map(CharGene::from).collect();
        CharChromosome { genes }
    }
}

impl From<&[char]> for CharChromosome {
    fn from(alleles: &[char]) -> Self {
        let genes = alleles.iter().map(CharGene::from).collect();
        CharChromosome { genes }
    }
}

/// Represents a chromosome composed of floating-point genes.
///
/// A `FloatChromosome` contains a vector of `FloatGene` instances, each representing
/// a single floating-point value. This structure is typically used in problems where
/// solutions are encoded as real numbers.
///
/// # Fields
///
/// * `genes` - A vector of `FloatGene` representing the individual's genetic information.
#[derive(Clone, PartialEq)]
pub struct FloatChromosome {
    pub genes: Vec<FloatGene>,
}

impl FloatChromosome {
    pub fn new(genes: Vec<FloatGene>) -> Self {
        FloatChromosome { genes }
    }

    pub fn normalize(mut self) -> Self {
        let mut sum = 0.0;
        for gene in &self.genes {
            sum += gene.allele;
        }

        for gene in &mut self.genes {
            gene.allele /= sum;
        }

        self
    }

    pub fn standardize(mut self) -> Self {
        let mut sum = 0.0;
        for gene in &self.genes {
            sum += gene.allele;
        }

        let mean = sum / self.genes.len() as f32;

        let mut variance = 0.0;
        for gene in &self.genes {
            variance += (gene.allele - mean).powi(2);
        }

        let std_dev = (variance / self.genes.len() as f32).sqrt();

        for gene in &mut self.genes {
            gene.allele = (gene.allele - mean) / std_dev;
        }

        self
    }
}

impl Chromosome for FloatChromosome {
    type Gene = FloatGene;
}

impl Valid for FloatChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl AsRef<[FloatGene]> for FloatChromosome {
    fn as_ref(&self) -> &[FloatGene] {
        &self.genes
    }
}

impl AsMut<[FloatGene]> for FloatChromosome {
    fn as_mut(&mut self) -> &mut [FloatGene] {
        &mut self.genes
    }
}

impl From<Range<i32>> for FloatChromosome {
    fn from(range: Range<i32>) -> Self {
        let mut genes = Vec::new();
        for _ in range.start..range.end {
            genes.push(FloatGene::new(range.start as f32, range.end as f32));
        }

        FloatChromosome { genes }
    }
}

impl From<&[f32]> for FloatChromosome {
    fn from(alleles: &[f32]) -> Self {
        let genes = alleles.iter().map(FloatGene::from).collect();
        FloatChromosome { genes }
    }
}

/// Represents a chromosome composed of integer genes.
///
/// An `IntChromosome` is generic over the integer type `T` and contains a vector of `IntGene<T>`
/// instances. This structure is suitable for optimization problems where solutions are encoded
/// as integers.
///
/// # Type Parameters
///
/// * `T` - The integer type used for genes (e.g., `i32`, `u32`).
///
/// # Fields
///
/// * `genes` - A vector of `IntGene<T>` representing the individual's genetic informationn.
///
#[derive(Clone, PartialEq)]
pub struct IntChromosome<I: Integer<I>>
where
    Standard: rand::distributions::Distribution<I>,
{
    pub genes: Vec<IntGene<I>>,
}

impl<I: Integer<I>> IntChromosome<I>
where
    Standard: rand::distributions::Distribution<I>,
{
    pub fn new(genes: Vec<IntGene<I>>) -> Self {
        IntChromosome { genes }
    }
}

impl<I: Integer<I>> Chromosome for IntChromosome<I>
where
    Standard: rand::distributions::Distribution<I>,
{
    type Gene = IntGene<I>;
}

impl<T: Integer<T>> Valid for IntChromosome<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl<T: Integer<T>> AsRef<[IntGene<T>]> for IntChromosome<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn as_ref(&self) -> &[IntGene<T>] {
        &self.genes
    }
}

impl<T: Integer<T>> AsMut<[IntGene<T>]> for IntChromosome<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn as_mut(&mut self) -> &mut [IntGene<T>] {
        &mut self.genes
    }
}

impl<T: Integer<T>> From<&[T]> for IntChromosome<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn from(alleles: &[T]) -> Self {
        let genes = alleles.iter().map(IntGene::from).collect();
        IntChromosome { genes }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PermutationChromosome<A: PartialEq + Clone> {
    pub genes: Vec<PermutationGene<A>>,
    pub alleles: Arc<Vec<A>>,
}

impl<A: PartialEq + Clone> PermutationChromosome<A> {
    pub fn new(genes: Vec<PermutationGene<A>>, alleles: Arc<Vec<A>>) -> Self {
        PermutationChromosome { genes, alleles }
    }
}

impl<A: PartialEq + Clone> Chromosome for PermutationChromosome<A> {
    type Gene = PermutationGene<A>;
}

impl<A: PartialEq + Clone> Valid for PermutationChromosome<A> {
    fn is_valid(&self) -> bool {
        // Check if the genes are a valid permutation of the alleles
        let mut bit_set = vec![false; self.alleles.len()];
        self.genes.iter().all(|gene| {
            let index = gene.index;
            if bit_set[index] {
                return false;
            }
            bit_set[index] = true;
            true
        })
    }
}

impl<A: PartialEq + Clone> AsRef<[PermutationGene<A>]> for PermutationChromosome<A> {
    fn as_ref(&self) -> &[PermutationGene<A>] {
        &self.genes
    }
}

impl<A: PartialEq + Clone> AsMut<[PermutationGene<A>]> for PermutationChromosome<A> {
    fn as_mut(&mut self) -> &mut [PermutationGene<A>] {
        &mut self.genes
    }
}
