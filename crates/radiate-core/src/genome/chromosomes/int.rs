use super::{
    Chromosome, Integer,
    gene::{ArithmeticGene, Gene, Valid},
};
use crate::{chromosomes::BoundedGene, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Range, Sub},
};

#[macro_export]
macro_rules! impl_integer {
    ($($t:ty),*) => {
        $(
            impl Integer<$t> for $t {
                const MIN: $t = <$t>::MIN;
                const MAX: $t = <$t>::MAX;
                const ZERO: $t = 0;
                const ONE: $t = 1;
                const TWO: $t = 2;

                fn sat_add(self, rhs: $t) -> $t {
                    self.saturating_add(rhs)
                }

                fn sat_sub(self, rhs: $t) -> $t {
                    self.saturating_sub(rhs)
                }

                fn sat_mul(self, rhs: $t) -> $t {
                    self.saturating_mul(rhs)
                }

                fn sat_div(self, rhs: $t) -> $t {
                    if rhs == Self::ZERO {
                        self.saturating_div(Self::ONE)
                    } else {
                        self.saturating_div(rhs)
                    }
                }

                fn clamp(self, min: $t, max: $t) -> $t {
                    if self < min {
                        min
                    } else if self > max {
                        max
                    } else {
                        self
                    }
                }
            }
        )*
    };
}

/// A [`Gene`] that represents an integer value. This gene just wraps an integer value and provides
/// functionality for it to be used in a genetic algorithm. In this [`Gene`] implementation, the
/// `allele` is the integer value itself, the min and max values are the minimum and maximum values
/// that the integer can be generated from, and the upper and lower bounds are the upper and lower bounds the gene will
/// be subject to during crossover and mutation. If the `allele` exceedes the bounds, the [`Gene`] will be considered invalid.
///
/// [`IntGene`] is generic over `T` - the type of integer. The `Integer` trait is implemented
/// for `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, and `u128`.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
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
/// // be set to 0 and 10, and the upper and lower bounds will be set to 0 and 10.
/// let gene = IntGene::from(0..10);
///
/// // Create a gene with a min value of 0 and a max value of 10, but with upper and lower bounds of 10 and 0.
/// // In this case, the allele will be a random value between 0 and 10, but the lower and upper bounds will be -10 and 10.
/// let gene = IntGene::from((0..10, -10..10));
/// ```
///
/// # Type Parameters
/// - `T`: The type of integer used in the gene.
#[derive(Clone, PartialEq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IntGene<T: Integer<T>> {
    allele: T,
    value_range: Range<T>,
    bounds: Range<T>,
}

impl<T: Integer<T>> IntGene<T> {
    /// Create a new [`IntGene`] with the given allele, value range and bounds.
    pub fn new(allele: T, value_range: Range<T>, bounds: Range<T>) -> Self {
        IntGene {
            allele,
            value_range,
            bounds,
        }
    }
}

/// Implement the [`Gene`] trait for [`IntGene`]. This allows the [`IntGene`] to be used in a genetic algorithm.
impl<T: Integer<T>> Gene for IntGene<T> {
    type Allele = T;

    fn allele(&self) -> &T {
        &self.allele
    }

    fn allele_mut(&mut self) -> &mut T {
        &mut self.allele
    }

    /// Create a new instance of the [`IntGene`] with a random allele between the min and max values.
    fn new_instance(&self) -> IntGene<T> {
        IntGene {
            allele: random_provider::range(self.value_range.clone()),
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

/// Implement the `Valid` trait for [`IntGene`]. This allows the [`IntGene`] to be checked for validity.
/// An [`IntGene`] is valid if the `allele` is between the `min` and `max` values.
///
/// Note: the bounds are used for crossover and mutation.
impl<T: Integer<T>> Valid for IntGene<T> {
    fn is_valid(&self) -> bool {
        self.allele >= self.bounds.start && self.allele <= self.bounds.end
    }
}

/// Implement the `BoundedGene` trait for [`IntGene`]. This allows parts of radiate to
/// access the 'min', 'max', and bounds values of the [`IntGene`].
impl<T: Integer<T>> BoundedGene for IntGene<T> {
    fn min(&self) -> &Self::Allele {
        &self.value_range.start
    }

    fn max(&self) -> &Self::Allele {
        &self.value_range.end
    }

    fn bounds(&self) -> (&Self::Allele, &Self::Allele) {
        (&self.bounds.start, &self.bounds.end)
    }
}

/// Implement the `ArithmeticGene` trait for [`IntGene`]. This allows the [`IntGene`] to be used in numeric
/// operations. The `ArithmeticGene` trait is a superset of the [`Gene`] trait, and adds functionality
/// for numeric operations such as addition, subtraction, multiplication, division and mean.
impl<T: Integer<T>> ArithmeticGene for IntGene<T> {
    fn mean(&self, other: &IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: (self.allele.sat_add(other.allele)).sat_div(T::TWO),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<T: Integer<T>> Add for IntGene<T> {
    type Output = IntGene<T>;

    fn add(self, other: IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: self
                .allele
                .sat_add(other.allele)
                .clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<T: Integer<T>> Sub for IntGene<T> {
    type Output = IntGene<T>;

    fn sub(self, other: IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: self
                .allele
                .sat_sub(other.allele)
                .clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<T: Integer<T>> Mul for IntGene<T> {
    type Output = IntGene<T>;

    fn mul(self, other: IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: self
                .allele
                .sat_mul(other.allele)
                .clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<T: Integer<T>> Div for IntGene<T> {
    type Output = IntGene<T>;

    fn div(self, other: IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: self
                .allele
                .sat_div(other.allele)
                .clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
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

impl<T: Integer<T>> From<Range<T>> for IntGene<T> {
    fn from(range: Range<T>) -> Self {
        let (min, max) = (range.start, range.end);

        IntGene {
            allele: random_provider::range(range),
            value_range: min..max,
            bounds: min..max,
        }
    }
}

impl<T: Integer<T>> From<(Range<T>, Range<T>)> for IntGene<T> {
    fn from((range, bounds): (Range<T>, Range<T>)) -> Self {
        IntGene {
            allele: random_provider::range(range.clone()),
            value_range: range,
            bounds,
        }
    }
}

impl<T: Integer<T>> std::fmt::Display for IntGene<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}

/// Represents a chromosome composed of integer genes.
///
/// An [`IntChromosome`] is generic over the integer type `T` and contains a vector of [`IntGene<T>`]
/// instances. This structure is suitable for optimization problems where solutions are encoded
/// as integers.
///
/// # Type Parameters
///
/// * `T` - The integer type used for genes (e.g., `i32`, `u32`).
///
/// # Fields
///
/// * `genes` - A vector of [`IntGene<T>`] representing the individual's genetic informationn.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new IntChromosome with a vector of IntGene instances.
/// let genes = vec![IntGene::from(0..10), IntGene::from(10..20)];
/// let chromosome = IntChromosome::new(genes);
///
/// // Check if the chromosome is valid.
/// assert!(chromosome.is_valid());
///
#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IntChromosome<I: Integer<I>> {
    genes: Vec<IntGene<I>>,
}

impl<I: Integer<I>> IntChromosome<I> {
    /// Given a vec of [IntGene]'s, create a new [IntChromosome].
    pub fn new(genes: Vec<IntGene<I>>) -> Self {
        IntChromosome { genes }
    }
}

impl<I: Integer<I>> Chromosome for IntChromosome<I> {
    type Gene = IntGene<I>;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

impl<T: Integer<T>> Valid for IntChromosome<T> {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl<T: Integer<T>> From<IntGene<T>> for IntChromosome<T> {
    fn from(gene: IntGene<T>) -> Self {
        IntChromosome { genes: vec![gene] }
    }
}

impl<T: Integer<T>> From<Vec<IntGene<T>>> for IntChromosome<T> {
    fn from(genes: Vec<IntGene<T>>) -> Self {
        IntChromosome { genes }
    }
}

impl<T: Integer<T>> From<(usize, Range<T>)> for IntChromosome<T> {
    fn from((size, range): (usize, Range<T>)) -> Self {
        IntChromosome {
            genes: (0..size).map(|_| IntGene::from(range.clone())).collect(),
        }
    }
}

impl<T: Integer<T>> From<(usize, Range<T>, Range<T>)> for IntChromosome<T> {
    fn from((size, range, bounds): (usize, Range<T>, Range<T>)) -> Self {
        IntChromosome {
            genes: (0..size)
                .map(|_| IntGene::from((range.clone(), bounds.clone())))
                .collect(),
        }
    }
}

impl<T: Integer<T>> From<Vec<T>> for IntChromosome<T> {
    fn from(alleles: Vec<T>) -> Self {
        IntChromosome {
            genes: alleles.into_iter().map(IntGene::from).collect(),
        }
    }
}

impl<T: Integer<T>> FromIterator<IntGene<T>> for IntChromosome<T> {
    fn from_iter<I: IntoIterator<Item = IntGene<T>>>(iter: I) -> Self {
        IntChromosome {
            genes: iter.into_iter().collect(),
        }
    }
}

impl<T: Integer<T>> IntoIterator for IntChromosome<T> {
    type Item = IntGene<T>;
    type IntoIter = std::vec::IntoIter<IntGene<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

impl<T: Integer<T>> Debug for IntChromosome<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.genes)
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

        let (one_min, one_max) = gene_one.bounds();
        let (two_min, two_max) = gene_two.bounds();

        assert_eq!(*one_min, 0);
        assert_eq!(*one_max, 10);
        assert_eq!(*two_min, -100);
        assert_eq!(*two_max, 100);
        assert_eq!(gene_one.min(), &0);
        assert_eq!(gene_one.max(), &10);
        assert_eq!(gene_two.min(), &0);
        assert_eq!(gene_two.max(), &10);
        assert!(gene_one.is_valid());
        assert!(gene_two.is_valid());
    }

    #[test]
    fn test_mean() {
        let gene = IntGene::from(5);
        let other = IntGene::from(5);
        let new_gene = gene.mean(&other);
        assert_eq!(new_gene.allele, 5);
    }

    #[test]
    fn test_int_arithmetic_doesnt_overflow() {
        let gene = IntGene::<u8>::from(8_u8);
        let other = IntGene::<u8>::from(8_u8);
        let sixteen = IntGene::<u8>::from(16_u8);

        assert_eq!((gene.clone() + other.clone()).allele, 16);
        assert_eq!((gene.clone() - sixteen.clone()).allele, 0);
        assert_eq!((gene.clone() * other.clone()).allele, 64);

        let zero = IntGene::<u8>::from(0_u8);
        assert_eq!((gene.clone() / zero.clone()).allele, 8);
        assert_eq!((gene.clone() / other.clone()).allele, 1);

        let max = IntGene::<u8>::from(u8::MAX);
        assert_eq!((max.clone() + other.clone()).allele, u8::MAX);
        assert_eq!((zero.clone() - other.clone()).allele, 0);

        let i_eight = IntGene::<i8>::from(8_i8);
        let i_other = IntGene::<i8>::from(8_i8);
        let i_sixteen = IntGene::<i8>::from(16_i8);

        assert_eq!((i_eight.clone() + i_other.clone()).allele, 16);
        assert_eq!((i_eight.clone() - i_sixteen.clone()).allele, -8);
        assert_eq!((i_eight.clone() * i_other.clone()).allele, 64);
    }

    #[test]
    fn test_int_clamp_arithmetic_clamping() {
        let gene = IntGene::new(5, 5..10, 0..10);
        let other = IntGene::new(5, 8..10, 0..10);
        let really_big = IntGene::new(100000, 0..10, 0..10);

        let add = gene.clone() + other.clone();
        let sub = gene.clone() - other.clone();
        let mul = gene.clone() * other.clone();
        let div = gene.clone() / other.clone();

        let really_big_add = gene.clone() + really_big.clone();
        let really_big_sub = gene.clone() - really_big.clone();
        let really_big_mul = gene.clone() * really_big.clone();
        let really_big_div = gene.clone() / really_big.clone();

        assert_eq!(add.allele, 10);
        assert_eq!(sub.allele, 0);
        assert_eq!(mul.allele, 10);
        assert_eq!(div.allele, 1);

        assert_eq!(really_big_add.allele, 10);
        assert_eq!(really_big_sub.allele, 0);
        assert_eq!(really_big_mul.allele, 10);
        assert_eq!(really_big_div.allele, 0);
    }

    #[test]
    fn test_into() {
        let gene: IntGene<i32> = 5.into();
        assert_eq!(gene.allele, 5);
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
            assert_eq!(*gene.bounds().0, -10);
            assert_eq!(*gene.bounds().1, 10);
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

    #[test]
    fn test_gene_arithmetic() {
        let gene_one = IntGene::from(5);
        let gene_two = IntGene::from(5);
        let zero_gene = IntGene::from(0);

        let add = gene_one.clone() + gene_two.clone();
        let sub = gene_one.clone() - gene_two.clone();
        let mul = gene_one.clone() * gene_two.clone();
        let div = gene_one.clone() / gene_two.clone();
        let div_zero = gene_one.clone() / zero_gene.clone();
        let mean = gene_one.mean(&gene_two);

        assert_eq!(add.allele, 10);
        assert_eq!(sub.allele, 0);
        assert_eq!(mul.allele, 25);
        assert_eq!(div.allele, 1);
        assert_eq!(div_zero.allele, 5);
        assert_eq!(mean.allele, 5);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_int_gene_serialization() {
        let gene = IntGene::from(-5_i32..5_i32);

        assert!(gene.is_valid());

        let serialized = serde_json::to_string(&gene).expect("Failed to serialize IntGene");
        let deserialized: IntGene<i32> =
            serde_json::from_str(&serialized).expect("Failed to deserialize IntGene");

        let chromosome = IntChromosome::from((10, 0..10, -10..10));
        let serialized_chromosome =
            serde_json::to_string(&chromosome).expect("Failed to serialize IntChromosome");
        let deserialized_chromosome: IntChromosome<i32> =
            serde_json::from_str(&serialized_chromosome)
                .expect("Failed to deserialize IntChromosome");

        assert_eq!(gene, deserialized);
        assert_eq!(chromosome, deserialized_chromosome);
    }
}
