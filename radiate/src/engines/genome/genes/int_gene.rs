use super::{
    gene::{BoundGene, Gene, NumericGene, Valid},
    Integer,
};
use crate::random_provider;
use rand::distributions::Standard;
use std::ops::{Add, Div, Mul, Sub};

/// A `Gene` that represents an integer value. This gene just wraps an integer value and provides
/// functionality for it to be used in a genetic algorithm. In this `Gene` implementation, the
/// `allele` is the integer value itself, the min and max values are the minimum and maximum values
/// that the integer can be generated from, and the upper and lower bounds are the upper and lower bounds the gene will
/// be subject to during crossover and mutation. If the `allele` exceedes the bounds, the `Gene` will be considered invalid.
///
/// `IntGene` is generic over `T` - the type of integer. The `Integer` trait is implemented for `i8`, `i16`, `i32`, `i64`, and `i128`.
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
/// let gene = IntGene::new(5);
///
/// // Create a gene, but with a min value of 0 and a max value of 10. In this case,
/// // the allele will be a random value between 0 and 10. The min and max values will
/// // be set to 0 and 10, and the upper and lower bounds will be set to i32::MAX and i32::MIN.
/// let gene = IntGene::from_min_max(0, 10);
///
/// // Create a gene with a min value of 0 and a max value of 10, but with upper and lower bounds of 10 and 0.
/// // In this case, the allele will be a random value between 0 and 10, but the upper and lower bounds will be 10 and 0.
/// let gene = IntGene::from_min_max(0, 10).with_bounds(10, 0);
/// ```
///
/// # Type Parameters
/// - `T`: The type of integer used in the gene.
///
#[derive(Clone, PartialEq)]
pub struct IntGene<T: Integer<T>>
where
    Standard: rand::distributions::Distribution<T>,
{
    pub allele: T,
    pub min: T,
    pub max: T,
    pub upper_bound: T,
    pub lower_bound: T,
}
impl<T: Integer<T>> IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    /// Create a new instance of the `IntGene` with the given allele. The min and max values will be set
    /// to the minimum and maximum values of the integer type `T`, and the upper and lower
    /// bounds will be set to the maximum and minimum values of the integer type `T`.
    pub fn new(allele: T) -> Self {
        IntGene {
            allele,
            min: T::MIN,
            max: T::MAX,
            upper_bound: T::MAX,
            lower_bound: T::MIN,
        }
    }

    /// Create a new instance of the `IntGene` with a random allele between the given min and max values.
    /// The upper and lower bounds will be set to the maximum and minimum values of the integer type `T`.
    pub fn from_min_max(min: T, max: T) -> Self {
        let (min, max) = if min > max { (max, min) } else { (min, max) };

        Self {
            allele: random_provider::gen_range(min..max),
            min,
            max,
            upper_bound: T::MAX,
            lower_bound: T::MIN,
        }
    }
}

/// Implement the `Gene` trait for `IntGene`. This allows the `IntGene` to be used in a genetic algorithm.
impl<T: Integer<T>> Gene for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    type Allele = T;

    fn allele(&self) -> &T {
        &self.allele
    }

    /// Create a new instance of the `IntGene` with a random allele between the min and max values.
    fn new_instance(&self) -> IntGene<T> {
        IntGene {
            allele: random_provider::gen_range(self.min..self.max),
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn from_allele(&self, allele: &T) -> IntGene<T> {
        IntGene {
            allele: *allele,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }
}

/// Implement the `Valid` trait for `IntGene`. This allows the `IntGene` to be checked for validity.
/// An `IntGene` is valid if the `allele` is between the `min` and `max` values.
///
/// Note: the bounds are used for crossover and mutation.
impl<T: Integer<T>> Valid for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn is_valid(&self) -> bool {
        self.allele >= self.min && self.allele <= self.max
    }
}

impl<T: Integer<T>> BoundGene for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn upper_bound(&self) -> &T {
        &self.upper_bound
    }

    fn lower_bound(&self) -> &T {
        &self.lower_bound
    }

    fn with_bounds(self, upper_bound: T, lower_bound: T) -> IntGene<T> {
        IntGene {
            upper_bound,
            lower_bound,
            ..self
        }
    }
}

impl<T: Integer<T>> NumericGene for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn min(&self) -> &T {
        &self.min
    }

    fn max(&self) -> &T {
        &self.max
    }

    fn mean(&self, other: &IntGene<T>) -> IntGene<T> {
        IntGene {
            allele: (self.allele + other.allele) / T::from_i32(2),
            ..*self
        }
    }
}

impl<T: Integer<T>> std::fmt::Debug for IntGene<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
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

impl From<i8> for IntGene<i8> {
    fn from(allele: i8) -> Self {
        IntGene::new(allele)
    }
}

impl From<i16> for IntGene<i16> {
    fn from(allele: i16) -> Self {
        IntGene::new(allele)
    }
}

impl From<i32> for IntGene<i32> {
    fn from(allele: i32) -> Self {
        IntGene::new(allele)
    }
}

impl From<i64> for IntGene<i64> {
    fn from(allele: i64) -> Self {
        IntGene::new(allele)
    }
}

impl From<i128> for IntGene<i128> {
    fn from(allele: i128) -> Self {
        IntGene::new(allele)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gene = IntGene::from_min_max(0, 10);
        assert!(gene.allele >= 0 && gene.allele <= 10);
    }

    #[test]
    fn test_new_instance() {
        let gene = IntGene::from_min_max(0, 10);
        let new_gene = gene.new_instance();
        assert!(new_gene.allele >= 0 && new_gene.allele <= 10);
    }

    #[test]
    fn test_from_allele() {
        let gene = IntGene::new(5);
        let new_gene = gene.from_allele(&5);
        assert_eq!(new_gene.allele, 5);
    }

    #[test]
    fn test_is_valid() {
        let gene = IntGene::from_min_max(0, 10);
        assert!(gene.is_valid());
    }

    #[test]
    fn test_upper_bound() {
        let gene = IntGene::from_min_max(0, 10).with_bounds(10, 0);
        assert_eq!(*gene.upper_bound(), 10);
    }

    #[test]
    fn test_lower_bound() {
        let gene = IntGene::from_min_max(0, 10).with_bounds(10, 0);
        assert_eq!(*gene.lower_bound(), 0);
    }

    #[test]
    fn test_mean() {
        let gene = IntGene::new(5);
        let other = IntGene::new(5);
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
        let gene = IntGene::new(5);
        let i: i32 = gene.into();
        assert_eq!(i, 5);
    }
}
