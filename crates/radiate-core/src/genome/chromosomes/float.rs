use super::{
    Chromosome,
    gene::{ArithmeticGene, BoundedGene, Gene, Valid},
};
use crate::random_provider;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Range, Sub},
};

/// Minimum and maximum values for the `FloatGene` allele.
/// This should be large enough to cover most practical use cases
/// but small enough to avoid overflow or underflow issues in calculations.
const MIN: f32 = -1e10;
const MAX: f32 = 1e10;

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
pub struct FloatGene {
    allele: f32,
    value_range: Range<f32>,
    bounds: Range<f32>,
}

impl FloatGene {
    /// Creates a new [`FloatGene`] with the given `allele`, `value_range`, and `bounds`.
    pub fn new(allele: f32, value_range: Range<f32>, bounds: Range<f32>) -> Self {
        FloatGene {
            allele,
            value_range: MIN.max(value_range.start)..MAX.min(value_range.end),
            bounds: MIN.max(bounds.start)..MAX.min(bounds.end),
        }
    }
}

/// Implement the [`Valid`] trait for the [`FloatGene`].
///
/// The `is_valid` method checks if the `allele` of the [`FloatGene`] is between the `min` and `max` values.
/// The `GeneticEngine` will check the validity of the [`Chromosome`] and `Phenotype` and remove any
/// invalid individuals from the population, replacing them with new individuals at the given generation.
impl Valid for FloatGene {
    fn is_valid(&self) -> bool {
        self.allele >= self.bounds.start && self.allele <= self.bounds.end
    }
}

impl Gene for FloatGene {
    type Allele = f32;

    fn allele(&self) -> &f32 {
        &self.allele
    }

    fn allele_mut(&mut self) -> &mut f32 {
        &mut self.allele
    }

    fn new_instance(&self) -> FloatGene {
        FloatGene {
            allele: random_provider::range(self.value_range.clone()),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }

    fn with_allele(&self, allele: &f32) -> FloatGene {
        FloatGene {
            allele: *allele,
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl BoundedGene for FloatGene {
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

impl ArithmeticGene for FloatGene {
    fn mean(&self, other: &FloatGene) -> FloatGene {
        FloatGene {
            allele: ((self.allele + other.allele) * 0.5).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl Add for FloatGene {
    type Output = FloatGene;

    fn add(self, other: FloatGene) -> FloatGene {
        FloatGene {
            allele: (self.allele + other.allele).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl Sub for FloatGene {
    type Output = FloatGene;

    fn sub(self, other: FloatGene) -> FloatGene {
        FloatGene {
            allele: (self.allele - other.allele).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl Mul for FloatGene {
    type Output = FloatGene;

    fn mul(self, other: FloatGene) -> FloatGene {
        FloatGene {
            allele: (self.allele * other.allele).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl Div for FloatGene {
    type Output = FloatGene;

    fn div(self, other: FloatGene) -> FloatGene {
        let denominator = if other.allele == 0.0 {
            1.0
        } else {
            other.allele
        };

        FloatGene {
            allele: (self.allele / denominator).clamp(self.bounds.start, self.bounds.end),
            value_range: self.value_range.clone(),
            bounds: self.bounds.clone(),
        }
    }
}

impl Default for FloatGene {
    fn default() -> Self {
        FloatGene {
            allele: 0.0,
            value_range: MIN..MAX,
            bounds: MIN..MAX,
        }
    }
}

impl From<FloatGene> for f32 {
    fn from(gene: FloatGene) -> f32 {
        gene.allele
    }
}

impl From<f32> for FloatGene {
    fn from(allele: f32) -> Self {
        FloatGene {
            allele,
            value_range: MIN..MAX,
            bounds: MIN..MAX,
        }
    }
}

impl From<Range<f32>> for FloatGene {
    fn from(range: Range<f32>) -> Self {
        let (min, max) = (range.start.max(MIN), range.end.min(MAX));

        FloatGene {
            allele: random_provider::range(range),
            value_range: min..max,
            bounds: min..max,
        }
    }
}

impl From<(Range<f32>, Range<f32>)> for FloatGene {
    fn from((value_range, bounds): (Range<f32>, Range<f32>)) -> Self {
        let value_range = value_range.start.max(MIN)..value_range.end.min(MAX);
        let bounds = bounds.start.max(MIN)..bounds.end.min(MAX);
        let allele = random_provider::range(value_range.clone());

        FloatGene {
            allele,
            value_range,
            bounds,
        }
    }
}

impl Display for FloatGene {
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
/// let chromosome_allels = chromosome
///     .iter()
///     .map(|gene| *gene.allele())
///     .collect::<Vec<f32>>();
///
/// assert!(chromosome.is_valid());
/// assert_eq!(chromosome_allels.len(), 3);
/// assert_eq!(chromosome_allels, vec![0.0, 1.0, 2.0]);
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
pub struct FloatChromosome {
    genes: Vec<FloatGene>,
}

impl FloatChromosome {
    pub fn new(genes: Vec<FloatGene>) -> Self {
        FloatChromosome { genes }
    }
}

impl Chromosome for FloatChromosome {
    type Gene = FloatGene;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

impl Valid for FloatChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl From<FloatGene> for FloatChromosome {
    fn from(gene: FloatGene) -> Self {
        FloatChromosome { genes: vec![gene] }
    }
}

impl From<Vec<FloatGene>> for FloatChromosome {
    fn from(genes: Vec<FloatGene>) -> Self {
        FloatChromosome { genes }
    }
}

impl From<Vec<f32>> for FloatChromosome {
    fn from(alleles: Vec<f32>) -> Self {
        FloatChromosome {
            genes: alleles.into_iter().map(FloatGene::from).collect(),
        }
    }
}

impl From<(usize, Range<f32>)> for FloatChromosome {
    fn from((size, range): (usize, Range<f32>)) -> Self {
        FloatChromosome {
            genes: (0..size).map(|_| FloatGene::from(range.clone())).collect(),
        }
    }
}

impl From<(usize, Range<f32>, Range<f32>)> for FloatChromosome {
    fn from((size, range, bounds): (usize, Range<f32>, Range<f32>)) -> Self {
        FloatChromosome {
            genes: (0..size)
                .map(|_| FloatGene::from((range.clone(), bounds.clone())))
                .collect(),
        }
    }
}

impl FromIterator<FloatGene> for FloatChromosome {
    fn from_iter<I: IntoIterator<Item = FloatGene>>(iter: I) -> Self {
        FloatChromosome {
            genes: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for FloatChromosome {
    type Item = FloatGene;
    type IntoIter = std::vec::IntoIter<FloatGene>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

impl Debug for FloatChromosome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.genes)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
    fn test_into() {
        let gene = FloatGene::from(0_f32..1_f32);
        let copy = gene.clone();
        let allele: f32 = gene.into();
        assert_eq!(allele, copy.allele);
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
        let deserialized: FloatGene =
            serde_json::from_str(&serialized).expect("Failed to deserialize FloatGene");

        let chromosome = FloatChromosome::from((10, 0.0..1.0, -1.0..1.0));
        let serialized_chromosome =
            serde_json::to_string(&chromosome).expect("Failed to serialize FloatChromosome");
        let deserialized_chromosome: FloatChromosome = serde_json::from_str(&serialized_chromosome)
            .expect("Failed to deserialize FloatChromosome");

        assert_eq!(gene, deserialized);
        assert_eq!(chromosome, deserialized_chromosome);
    }
}
