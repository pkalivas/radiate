use super::{
    Chromosome,
    gene::{ArithmeticGene, BoundedGene, Gene, Valid},
};
use crate::random_provider;
use rand::distr::uniform::SampleUniform;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Range, Sub},
};

pub trait Float:
    Copy
    + Clone
    + PartialOrd
    + Debug
    + PartialEq
    + SampleUniform
    + Display
    + Default
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const EPS: Self;
    const HALF: Self;

    fn is_finite(self) -> bool;
    fn sanitize(self) -> Self;
    fn from_f64(value: f64) -> Self;
    fn from_f32(value: f32) -> Self;

    fn abs(self) -> Self {
        if self < Self::ZERO { -self } else { self }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }
    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

#[macro_export]
macro_rules! impl_float_scalar {
    ($t:ty, $min:expr, $max:expr, $eps:expr) => {
        impl Float for $t {
            const MIN: Self = $min;
            const MAX: Self = $max;
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const TWO: Self = 2.0;
            const HALF: Self = 0.5;
            const EPS: Self = $eps;

            fn from_f64(value: f64) -> Self {
                value as Self
            }

            fn from_f32(value: f32) -> Self {
                value as Self
            }

            fn is_finite(self) -> bool {
                <$t>::is_finite(self)
            }

            fn sanitize(self) -> Self {
                if self.is_finite() { self } else { Self::EPS }
            }
        }
    };
}

impl_float_scalar!(f32, -1e18_f32, 1e18_f32, 1e-6_f32);
impl_float_scalar!(f64, -1e100_f64, 1e100_f64, 1e-12_f64);

#[inline]
pub fn safe_div<T: Float>(num: T, den: T) -> T {
    let den = if den.sanitize() == T::ZERO {
        return num.sanitize();
    } else {
        den.sanitize()
    };

    (num.sanitize() / den).sanitize()
}

#[inline]
pub fn safe_add<T: Float>(a: T, b: T) -> T {
    (a.sanitize() + b.sanitize()).sanitize()
}

#[inline]
pub fn safe_sub<T: Float>(a: T, b: T) -> T {
    (a.sanitize() - b.sanitize()).sanitize()
}

#[inline]
pub fn safe_mul<T: Float>(a: T, b: T) -> T {
    (a.sanitize() * b.sanitize()).sanitize()
}

/// Minimum and maximum values for the [FloatGene] allele.
/// This should be large enough to cover most practical use cases
/// but small enough to avoid overflow or underflow issues in calculations.
/// 1e18 = 1 quintillion

/// A [`Gene`] that represents a floating point number.
/// The `allele` is the in the case of the [`FloatGene`] a f32. The `min` and `max` values
/// default to [MIN] and [MAX] respectively. The `min` and `max` values are used to
/// generate a random number between the `min` and `max` values, which is the `allele` of the [`FloatGene`].
/// The `upper_bound` and `lower_bound` are used to set the bounds of the [`FloatGene`] when it is used
/// in a `BoundGene` context (crossover or mutation). The `upper_bound` and `lower_bound`
/// default to [MAX] and [MIN] respectively.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatGene with a min value of 0 and a max value of 1 meaning the
/// // allele will be a random number between 0 and 1.
/// // The upper_bound and lower_bound are set to 0 and 1 respectively.
/// let gene = FloatGene::from(0_f32..1_f32);
///
/// // Create a new FloatGene with a min of 0 and a max of 1 and set the upper_bound
/// // and lower_bound to 0 and 100 respectively.
/// let gene = FloatGene::from((0_f32..1_f32, 0_f32..100_f32));
/// ```
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatGene<F: Float> {
    allele: F,
    value_range: Range<F>,
    bounds: Range<F>,
}

impl<F: Float> FloatGene<F> {
    /// Creates a new [`FloatGene`] with the given `allele`, `value_range`, and `bounds`.
    pub fn new(allele: F, value_range: Range<F>, bounds: Range<F>) -> Self {
        FloatGene {
            allele,
            value_range: value_range.start.max(F::MIN)..value_range.end.min(F::MAX),
            bounds: bounds.start.max(F::MIN)..bounds.end.min(F::MAX),
        }
    }
}

/// Implement the [`Valid`] trait for the [`FloatGene`].
///
/// The `is_valid` method checks if the `allele` of the [`FloatGene`] is between the `min` and `max` values.
/// The `GeneticEngine` will check the validity of the [`Chromosome`] and `Phenotype` and remove any
/// invalid individuals from the population, replacing them with new individuals at the given generation.
impl<F: Float> Valid for FloatGene<F> {
    fn is_valid(&self) -> bool {
        self.allele >= self.bounds.start && self.allele <= self.bounds.end
    }
}

impl<F: Float> Gene for FloatGene<F> {
    type Allele = F;

    fn allele(&self) -> &F {
        &self.allele
    }

    fn allele_mut(&mut self) -> &mut F {
        &mut self.allele
    }

    fn new_instance(&self) -> FloatGene<F> {
        FloatGene {
            allele: random_provider::range(self.value_range.clone()),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }

    fn with_allele(&self, allele: &F) -> FloatGene<F> {
        FloatGene {
            allele: *allele,
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<F: Float> BoundedGene for FloatGene<F> {
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

impl<F: Float> ArithmeticGene for FloatGene<F> {
    fn mean(&self, other: &FloatGene<F>) -> FloatGene<F> {
        FloatGene {
            allele: safe_mul(safe_add(self.allele, other.allele), F::HALF)
                .clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<F: Float> Add for FloatGene<F> {
    type Output = FloatGene<F>;

    fn add(self, other: FloatGene<F>) -> FloatGene<F> {
        FloatGene {
            allele: safe_add(self.allele, other.allele).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<F: Float> Sub for FloatGene<F> {
    type Output = FloatGene<F>;

    fn sub(self, other: FloatGene<F>) -> FloatGene<F> {
        FloatGene {
            allele: safe_sub(self.allele, other.allele).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<F: Float> Mul for FloatGene<F> {
    type Output = FloatGene<F>;

    fn mul(self, other: FloatGene<F>) -> FloatGene<F> {
        FloatGene {
            allele: safe_mul(self.allele, other.allele).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<F: Float> Div for FloatGene<F> {
    type Output = FloatGene<F>;

    fn div(self, other: FloatGene<F>) -> FloatGene<F> {
        FloatGene {
            allele: safe_div(self.allele, other.allele).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl<F: Float> Default for FloatGene<F> {
    fn default() -> Self {
        FloatGene {
            allele: F::ZERO,
            value_range: F::MIN..F::MAX,
            bounds: F::MIN..F::MAX,
        }
    }
}

impl<F: Float> From<F> for FloatGene<F> {
    fn from(allele: F) -> Self {
        FloatGene {
            allele,
            value_range: F::MIN..F::MAX,
            bounds: F::MIN..F::MAX,
        }
    }
}

impl<F: Float> From<Range<F>> for FloatGene<F> {
    fn from(range: Range<F>) -> Self {
        let (min, max) = (range.start.max(F::MIN), range.end.min(F::MAX));

        FloatGene {
            allele: random_provider::range(range),
            value_range: min..max,
            bounds: min..max,
        }
    }
}

impl<F: Float> From<(Range<F>, Range<F>)> for FloatGene<F> {
    fn from((value_range, bounds): (Range<F>, Range<F>)) -> Self {
        let value_range = value_range.start.max(F::MIN)..value_range.end.min(F::MAX);
        let bounds = bounds.start.max(F::MIN)..bounds.end.min(F::MAX);
        let allele = random_provider::range(value_range.clone());

        FloatGene {
            allele,
            value_range,
            bounds,
        }
    }
}

impl<F: Float> Display for FloatGene<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}

/// Represents a chromosome composed of floating-point genes.
///
/// A [`FloatChromosome`] contains a vector of [`FloatGene`] instances, each representing
/// a single floating-point value. This structure is typically used in problems where
/// solutions are encoded as real numbers.
///
/// # Fields
///
/// * `genes` - A vector of [`FloatGene`] representing the individual's genetic information.
///
/// # Example
/// ```rust
/// use radiate_core::*;
///
/// // Create a chromosome with 3 genes with alleles 0.0, 1.0, and 2.0 respectively
/// let chromosome = FloatChromosome::from(vec![0.0, 1.0, 2.0]);
/// let chromosome_alleles = chromosome
///     .iter()
///     .map(|gene| *gene.allele())
///     .collect::<Vec<f32>>();
///
/// assert!(chromosome.is_valid());
/// assert_eq!(chromosome_alleles.len(), 3);
/// assert_eq!(chromosome_alleles, vec![0.0, 1.0, 2.0]);
///
/// // Create a chromosome with 3 genes all with alleles in the range 0.0 to 10.0
/// let ranged_chromo = FloatChromosome::from((3, 0.0..10.0));
/// let ranged_chromo_alleles = ranged_chromo
///    .iter()
///    .map(|gene| *gene.allele())
///    .collect::<Vec<f32>>();
///
/// assert!(ranged_chromo.is_valid());
/// assert_eq!(ranged_chromo_alleles.len(), 3);
/// for allele in ranged_chromo_alleles {
///    assert!(allele >= 0.0 && allele <= 10.0);
/// }
///```
#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatChromosome<F: Float> {
    genes: Vec<FloatGene<F>>,
}

impl<F: Float> FloatChromosome<F> {
    pub fn new(genes: Vec<FloatGene<F>>) -> Self {
        FloatChromosome { genes }
    }
}

impl<F: Float> Chromosome for FloatChromosome<F> {
    type Gene = FloatGene<F>;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

impl<F: Float> Valid for FloatChromosome<F> {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl<F: Float> From<FloatGene<F>> for FloatChromosome<F> {
    fn from(gene: FloatGene<F>) -> Self {
        FloatChromosome { genes: vec![gene] }
    }
}

impl<F: Float> From<Vec<FloatGene<F>>> for FloatChromosome<F> {
    fn from(genes: Vec<FloatGene<F>>) -> Self {
        FloatChromosome { genes }
    }
}

impl<F: Float> From<Vec<F>> for FloatChromosome<F> {
    fn from(alleles: Vec<F>) -> Self {
        FloatChromosome {
            genes: alleles.into_iter().map(FloatGene::from).collect(),
        }
    }
}

impl<F: Float> From<(usize, Range<F>)> for FloatChromosome<F> {
    fn from((size, range): (usize, Range<F>)) -> Self {
        FloatChromosome {
            genes: (0..size).map(|_| FloatGene::from(range.clone())).collect(),
        }
    }
}

impl<F: Float> From<(usize, Range<F>, Range<F>)> for FloatChromosome<F> {
    fn from((size, range, bounds): (usize, Range<F>, Range<F>)) -> Self {
        FloatChromosome {
            genes: (0..size)
                .map(|_| FloatGene::from((range.clone(), bounds.clone())))
                .collect(),
        }
    }
}

impl<F: Float> FromIterator<FloatGene<F>> for FloatChromosome<F> {
    fn from_iter<I: IntoIterator<Item = FloatGene<F>>>(iter: I) -> Self {
        FloatChromosome {
            genes: iter.into_iter().collect(),
        }
    }
}

impl<F: Float> IntoIterator for FloatChromosome<F> {
    type Item = FloatGene<F>;
    type IntoIter = std::vec::IntoIter<FloatGene<F>>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

impl<F: Float> Debug for FloatChromosome<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.genes)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const MIN: f32 = -1e18;
    const MAX: f32 = 1e18;

    #[test]
    fn test_new() {
        let gene_one = FloatGene::from(0_f32..1_f32);
        let gene_two = FloatGene::from((-1.0..1.0, -100.0..100.0));
        let gene_three = FloatGene::new(10.0, (MIN * 10.0)..(MAX * 10.0), -1000.0..1000.0);

        // assert_eq!(*gene_one.min(), 0_f32);
        assert_eq!(*gene_one.max(), 1_f32);
        assert_eq!(gene_one.bounds().0, &0_f32);
        assert_eq!(gene_one.bounds().1, &1_f32);
        assert!(gene_one.is_valid());

        // assert_eq!(*gene_two.min(), -1.0);
        assert_eq!(*gene_two.max(), 1.0);
        assert_eq!(gene_two.bounds().0, &-100.0);
        assert_eq!(gene_two.bounds().1, &100.0);
        assert!(gene_two.is_valid());

        assert_eq!(*gene_three.allele(), 10.0);
        // assert_eq!(*gene_three.min(), MIN);
        assert_eq!(*gene_three.max(), MAX);
        assert_eq!(gene_three.bounds().0, &-1000.0);
        assert_eq!(gene_three.bounds().1, &1000.0);
    }

    #[test]
    fn test_from() {
        let gene = FloatGene::from(0_f32..1_f32);
        let copy = gene.clone();
        assert_eq!(gene, copy);
    }

    #[test]
    fn test_is_valid() {
        let gene = FloatGene::from(0_f32..1_f32);
        assert!(gene.is_valid());
        assert!(gene.allele >= 0_f32 && gene.allele <= 1_f32);
    }

    #[test]
    fn test_gene_clamping() {
        let one = FloatGene::new(5.0, 0.0..10.0, 0.0..10.0);
        let two = FloatGene::new(5.0, 0.0..10.0, 0.0..10.0);
        let really_big = FloatGene::new(100000.0, 0.0..10.0, 0.0..10.0);

        let add = one.clone() + two.clone();
        let sub = one.clone() - two.clone();
        let mul = one.clone() * two.clone();
        let div = one.clone() / two.clone();

        assert_eq!(add.allele, 10.0);
        assert_eq!(sub.allele, 0.0);
        assert_eq!(mul.allele, 10.0);
        assert_eq!(div.allele, 1.0);

        let big_add = one.clone() + really_big.clone();
        let big_sub = one.clone() - really_big.clone();
        let big_mul = one.clone() * really_big.clone();
        let big_div = really_big.clone() / one.clone();

        assert_eq!(big_add.allele, 10.0);
        assert_eq!(big_sub.allele, 0.0);
        assert_eq!(big_mul.allele, 10.0);
        assert_eq!(big_div.allele, 10.0);
    }

    #[test]
    fn test_chromosome() {
        let chromosome = FloatChromosome::from((10, -1.0..1.0));

        assert_eq!(chromosome.len(), 10);
        assert!(chromosome.is_valid());
        for gene in chromosome.iter() {
            assert!(gene.is_valid());
            assert!(gene.allele >= -1.0 && gene.allele <= 1.0);
        }
    }

    #[test]
    fn test_chromosome_from_vec() {
        let chromosome = FloatChromosome::from(vec![0.0, 1.0, 2.0]);

        assert_eq!(chromosome.len(), 3);
        assert!(chromosome.is_valid());
        for (gene, allele) in chromosome.iter().zip(vec![0.0, 1.0, 2.0]) {
            assert!(gene.is_valid());
            assert_eq!(gene.allele, allele);
        }
    }

    #[test]
    fn test_chromosome_from_range_with_bounds() {
        let chromosome = FloatChromosome::from((3, 0.0..10.0, -10.0..10.0));

        assert_eq!(chromosome.len(), 3);
        assert!(chromosome.is_valid());
        for gene in chromosome.iter() {
            assert!(gene.is_valid());
            assert!(gene.allele >= 0.0 && gene.allele <= 10.0);
            assert!(gene.bounds.start >= -10.0 && gene.bounds.end <= 10.0);
        }
    }

    #[test]
    fn test_gene_arithmetic() {
        let gene_one = FloatGene::from(5_f32);
        let gene_two = FloatGene::from(10_f32);
        let zero_gene = FloatGene::from(0_f32);

        let add = gene_one.clone() + gene_two.clone();
        let sub = gene_one.clone() - gene_two.clone();
        let mul = gene_one.clone() * gene_two.clone();
        let div = gene_one.clone() / gene_two.clone();
        let mean = gene_one.clone().mean(&gene_two.clone());
        let div_zero = gene_one.clone() / zero_gene.clone();

        assert_eq!(add.allele, 15_f32);
        assert_eq!(sub.allele, -5_f32);
        assert_eq!(mul.allele, 50_f32);
        assert_eq!(div.allele, 0.5_f32);
        assert_eq!(mean.allele, 7.5_f32);
        assert_eq!(div_zero.allele, 5_f32);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_float_gene_serialization() {
        let gene = FloatGene::from(0.5_f32..1.5_f32);

        assert!(gene.is_valid());

        let serialized = serde_json::to_string(&gene).expect("Failed to serialize FloatGene");
        let deserialized: FloatGene<f32> =
            serde_json::from_str(&serialized).expect("Failed to deserialize FloatGene");

        let chromosome = FloatChromosome::from((10, 0.0..1.0, -1.0..1.0));
        let serialized_chromosome =
            serde_json::to_string(&chromosome).expect("Failed to serialize FloatChromosome");
        let deserialized_chromosome: FloatChromosome<f32> =
            serde_json::from_str(&serialized_chromosome)
                .expect("Failed to deserialize FloatChromosome");

        assert_eq!(gene, deserialized);
        assert_eq!(chromosome, deserialized_chromosome);
    }
}
