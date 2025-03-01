use super::{
    Chromosome, Integer,
    gene::{Gene, NumericGene, Valid},
};
use crate::random_provider;
use std::ops::{Bound, Range, RangeBounds};

/// A `Gene` that represents an integer value. This gene just wraps an integer value and provides
/// functionality for it to be used in a genetic algorithm. In this `Gene` implementation, the
/// `allele` is the integer value itself, the min and max values are the minimum and maximum values
/// that the integer can be generated from, and the upper and lower bounds are the upper and lower bounds the gene will
/// be subject to during crossover and mutation. If the `allele` exceedes the bounds, the `Gene` will be considered invalid.
///
/// `IntGene` is generic over `T` - the type of integer. The `Integer` trait is implemented
/// for `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, and `u128`.
///
/// # Example
/// ``` rust
/// use radiate::*;
///
/// // Create a new IntGene with an allele of 5, the min value will be i32::MIN
/// // and the max value will be i32::MAX - same for the upper and lower bounds.
/// let gene: IntGene<i32> = 5.into();
///
/// // Create the same gene, but with a different method
/// let gene = IntGene::from(5);
///
/// // Create a gene, but with a min value of 0 and a max value of 10. In this case,
/// // the allele will be a random value between 0 and 10. The min and max values will
/// // be set to 0 and 10, and the upper and lower bounds will be set to i32::MAX and i32::MIN.
/// let gene = IntGene::from(0..10);
///
/// // Create a gene with a min value of 0 and a max value of 10, but with upper and lower bounds of 10 and 0.
/// // In this case, the allele will be a random value between 0 and 10, but the lower and upper bounds will be -10 and 10.
/// let gene = IntGene::from((0..10, -10..10));
/// ```
///
/// # Type Parameters
/// - `T`: The type of integer used in the gene.
///
#[derive(Clone, PartialEq)]
pub struct IntGene<T: Integer<T>> {
    pub allele: T,
    pub value_range: Range<T>,
    pub bounds: Range<T>,
}

/// Implement the `Gene` trait for `IntGene`. This allows the `IntGene` to be used in a genetic algorithm.
impl<T: Integer<T>> Gene for IntGene<T> {
    type Allele = T;

    fn allele(&self) -> &T {
        &self.allele
    }

    /// Create a new instance of the `IntGene` with a random allele between the min and max values.
    fn new_instance(&self) -> IntGene<T> {
        IntGene {
            allele: random_provider::random_range(self.value_range.clone()),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }

    fn with_allele(&self, allele: &T) -> IntGene<T> {
        IntGene {
            allele: *allele,
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

/// Implement the `Valid` trait for `IntGene`. This allows the `IntGene` to be checked for validity.
/// An `IntGene` is valid if the `allele` is between the `min` and `max` values.
///
/// Note: the bounds are used for crossover and mutation.
impl<T: Integer<T>> Valid for IntGene<T> {
    fn is_valid(&self) -> bool {
        self.allele >= self.value_range.start && self.allele <= self.value_range.end
    }
}

impl<T: Integer<T>> NumericGene for IntGene<T> {
    fn min(&self) -> &T {
        &self.value_range.start
    }

    fn max(&self) -> &T {
        &self.value_range.end
    }

    fn mean(&self, other: &IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: (self.allele + other.allele) / T::from_i32(2),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<T: Integer<T>> RangeBounds<T> for IntGene<T> {
    fn start_bound(&self) -> Bound<&T> {
        self.bounds.start_bound()
    }

    fn end_bound(&self) -> Bound<&T> {
        self.bounds.end_bound()
    }
}

impl<T: Integer<T>> std::fmt::Debug for IntGene<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}

impl<T: Integer<T>> From<T> for IntGene<T> {
    fn from(allele: T) -> Self {
        IntGene {
            allele,
            value_range: T::MIN..T::MAX,
            bounds: T::MIN..T::MAX,
        }
    }
}

impl<T: Integer<T>> From<&T> for IntGene<T> {
    fn from(allele: &T) -> Self {
        IntGene {
            allele: *allele,
            value_range: T::MIN..T::MAX,
            bounds: T::MIN..T::MAX,
        }
    }
}

impl<T: Integer<T>> From<Range<T>> for IntGene<T> {
    fn from(range: Range<T>) -> Self {
        let (min, max) = (range.start, range.end);

        Self {
            allele: random_provider::random_range(range),
            value_range: min..max,
            bounds: min..max,
        }
    }
}

impl<T: Integer<T>> From<(Range<T>, Range<T>)> for IntGene<T> {
    fn from((range, bounds): (Range<T>, Range<T>)) -> Self {
        Self {
            allele: random_provider::random_range(range.clone()),
            value_range: range,
            bounds,
        }
    }
}

impl From<IntGene<i8>> for i8 {
    fn from(gene: IntGene<i8>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i16>> for i16 {
    fn from(gene: IntGene<i16>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i32>> for i32 {
    fn from(gene: IntGene<i32>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i64>> for i64 {
    fn from(gene: IntGene<i64>) -> Self {
        gene.allele
    }
}

impl From<IntGene<i128>> for i128 {
    fn from(gene: IntGene<i128>) -> Self {
        gene.allele
    }
}

impl From<IntGene<u8>> for u8 {
    fn from(gene: IntGene<u8>) -> Self {
        gene.allele
    }
}

impl From<IntGene<u16>> for u16 {
    fn from(gene: IntGene<u16>) -> Self {
        gene.allele
    }
}

impl From<IntGene<u32>> for u32 {
    fn from(gene: IntGene<u32>) -> Self {
        gene.allele
    }
}

impl From<IntGene<u64>> for u64 {
    fn from(gene: IntGene<u64>) -> Self {
        gene.allele
    }
}

impl From<IntGene<u128>> for u128 {
    fn from(gene: IntGene<u128>) -> Self {
        gene.allele
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
/// # Example
/// ``` rust
/// use radiate::*;
///
/// // Create a new IntChromosome with a vector of IntGene instances.
/// let genes = vec![IntGene::from(0..10), IntGene::from(10..20)];
/// let chromosome = IntChromosome::new(genes);
///
/// // Check if the chromosome is valid.
/// assert!(chromosome.is_valid());
#[derive(Clone, PartialEq, Default)]
pub struct IntChromosome<I: Integer<I>> {
    pub genes: Vec<IntGene<I>>,
}

impl<I: Integer<I>> IntChromosome<I> {
    pub fn new(genes: Vec<IntGene<I>>) -> Self {
        IntChromosome { genes }
    }
}

impl<I: Integer<I>> Chromosome for IntChromosome<I> {
    type Gene = IntGene<I>;
}

impl<T: Integer<T>> Valid for IntChromosome<T> {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl<T: Integer<T>> AsRef<[IntGene<T>]> for IntChromosome<T> {
    fn as_ref(&self) -> &[IntGene<T>] {
        &self.genes
    }
}

impl<T: Integer<T>> AsMut<[IntGene<T>]> for IntChromosome<T> {
    fn as_mut(&mut self) -> &mut [IntGene<T>] {
        &mut self.genes
    }
}

impl<T: Integer<T>> From<&[T]> for IntChromosome<T> {
    fn from(alleles: &[T]) -> Self {
        let genes = alleles.iter().map(IntGene::from).collect();
        IntChromosome { genes }
    }
}

impl<T: Integer<T>> From<(usize, Range<T>)> for IntChromosome<T> {
    fn from((size, range): (usize, Range<T>)) -> Self {
        let genes = (0..size).map(|_| IntGene::from(range.clone())).collect();
        IntChromosome { genes }
    }
}

impl<T: Integer<T>> From<(usize, Range<T>, Range<T>)> for IntChromosome<T> {
    fn from((size, range, bounds): (usize, Range<T>, Range<T>)) -> Self {
        let genes = (0..size)
            .map(|_| IntGene::from((range.clone(), bounds.clone())))
            .collect();
        IntChromosome { genes }
    }
}

impl<T: Integer<T>> From<Vec<T>> for IntChromosome<T> {
    fn from(alleles: Vec<T>) -> Self {
        let genes = alleles.iter().map(IntGene::from).collect();
        IntChromosome { genes }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new() {
        let gene = IntGene::from(0..10);
        assert!(gene.allele >= 0 && gene.allele <= 10);
    }

    #[test]
    fn test_new_instance() {
        let gene = IntGene::from(0..10);
        let new_gene = gene.new_instance();
        assert!(new_gene.allele >= 0 && new_gene.allele <= 10);
    }

    #[test]
    fn test_from_allele() {
        let gene = IntGene::from(5);
        let new_gene = gene.with_allele(&5);
        assert_eq!(new_gene.allele, 5);
    }

    #[test]
    fn test_is_valid() {
        let gene = IntGene::from(0..10);
        assert!(gene.is_valid());
    }

    #[test]
    fn test_bounds() {
        let gene_one = IntGene::from((0..10, 0..10));
        let gene_two = IntGene::from((0..10, -100..100));

        assert_eq!(*gene_one.min(), 0);
        assert_eq!(*gene_one.max(), 10);
        assert_eq!(*gene_two.min(), 0);
        assert_eq!(*gene_two.max(), 10);
        assert_eq!(gene_one.start_bound(), Bound::Included(&0));
        assert_eq!(gene_one.end_bound(), Bound::Excluded(&10));
        assert_eq!(gene_two.start_bound(), Bound::Included(&-100));
        assert_eq!(gene_two.end_bound(), Bound::Excluded(&100));
        assert!(gene_one.is_valid());
        assert!(gene_two.is_valid());
    }

    #[test]
    fn test_lower_bound() {
        let gene = IntGene::from((0..10, 0..10));

        assert_eq!(gene.start_bound(), Bound::Included(&0));
        assert_eq!(gene.end_bound(), Bound::Excluded(&10));
    }

    #[test]
    fn test_mean() {
        let gene = IntGene::from(5);
        let other = IntGene::from(5);
        let new_gene = gene.mean(&other);
        assert_eq!(new_gene.allele, 5);
    }

    #[test]
    fn test_into() {
        let gene: IntGene<i32> = 5.into();
        assert_eq!(gene.allele, 5);
    }

    #[test]
    fn test_from() {
        let gene = IntGene::from(5);
        let i: i32 = gene.into();
        assert_eq!(i, 5);
    }

    #[test]
    fn test_chromosome_from_range() {
        let chromosome = IntChromosome::from((10, 0..10));
        assert_eq!(chromosome.genes.len(), 10);
        for gene in &chromosome.genes {
            assert!(gene.allele >= 0 && gene.allele <= 10);
        }
    }

    #[test]
    fn test_chromosome_from_range_with_bounds() {
        let chromosome = IntChromosome::from((10, 0..10, -10..10));

        assert_eq!(chromosome.genes.len(), 10);
        for gene in &chromosome.genes {
            assert!(gene.allele >= 0 && gene.allele <= 10);
            assert!(gene.bounds.start_bound() == Bound::Included(&-10));
            assert!(gene.bounds.end_bound() == Bound::Excluded(&10));
        }
    }

    #[test]
    fn test_chromosome_from_alleles() {
        let alleles = vec![1, 2, 3, 4, 5];
        let chromosome = IntChromosome::from(alleles.clone());
        assert_eq!(chromosome.genes.len(), 5);
        for (i, gene) in chromosome.genes.iter().enumerate() {
            assert_eq!(gene.allele, alleles[i]);
        }
    }
}
