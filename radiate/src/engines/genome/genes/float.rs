use super::gene::{BoundGene, Gene, NumericGene, Valid};
use crate::random_provider;
use std::fmt::Debug;

/// A `Gene` that represents a floating point number.
/// The `allele` is the in the case of the `FloatGene` a f32. The `min` and `max` values
/// default to f32::MIN and f32::MAX respectively. The `min` and `max` values are used to
/// generate a random number between the `min` and `max` values, which is the `allele` of the `FloatGene`.
/// The `upper_bound` and `lower_bound` are used to set the bounds of the `FloatGene` when it is used
/// in a `BoundGene` context (crossover or mutation). The `upper_bound` and `lower_bound`
/// default to f32::MAX and f32::MIN respectively.
///
/// # Example
/// ``` rust
/// use radiate::*;
///
/// // Create a new FloatGene with a min value of 0 and a max value of 1 meaning the
/// // allele will be a random number between 0 and 1.
/// // The upper_bound and lower_bound are set to f32::MAX and f32::MIN respectively.
/// let gene = FloatGene::new(0_f32, 1_f32);
///
/// // Create a new FloatGene with a min of 0 and a max of 1 and set the upper_bound
/// // and lower_bound to 0 and 100 respectively.
/// let gene = FloatGene::new(0_f32, 1_f32).with_bounds(100_f32, 0_f32);
/// ```
///
#[derive(Clone, PartialEq)]
pub struct FloatGene {
    pub allele: f32,
    pub min: f32,
    pub max: f32,
    pub upper_bound: f32,
    pub lower_bound: f32,
}

impl FloatGene {
    pub fn new(min: f32, max: f32) -> Self {
        FloatGene {
            allele: random_provider::random::<f32>() * (max - min) + min,
            min,
            max,
            upper_bound: f32::MAX,
            lower_bound: f32::MIN,
        }
    }
}

/// Implement the `Valid` trait for the `FloatGene`.
///
/// The `is_valid` method checks if the `allele` of the `FloatGene` is between the `min` and `max` values.
/// The `GeneticEngine` will check the validity of the `Chromosome` and `Phenotype` and remove any
/// invalid individuals from the population, replacing them with new individuals at the given generation.
impl Valid for FloatGene {
    fn is_valid(&self) -> bool {
        self.allele >= self.lower_bound && self.allele <= self.upper_bound
    }
}

impl Gene for FloatGene {
    type Allele = f32;

    fn allele(&self) -> &f32 {
        &self.allele
    }

    fn new_instance(&self) -> FloatGene {
        FloatGene {
            allele: random_provider::random::<f32>() * (self.max - self.min) + self.min,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn with_allele(&self, allele: &f32) -> FloatGene {
        FloatGene {
            allele: *allele,
            min: self.min,
            max: self.max,
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }
}

impl BoundGene for FloatGene {
    fn upper_bound(&self) -> &f32 {
        &self.upper_bound
    }

    fn lower_bound(&self) -> &f32 {
        &self.lower_bound
    }

    fn with_bounds(self, lower_bound: f32, upper_bound: f32) -> FloatGene {
        FloatGene {
            upper_bound,
            lower_bound,
            ..self
        }
    }
}

impl NumericGene for FloatGene {
    fn min(&self) -> &Self::Allele {
        &self.min
    }

    fn max(&self) -> &Self::Allele {
        &self.max
    }

    fn mean(&self, other: &FloatGene) -> FloatGene {
        FloatGene {
            allele: (self.allele + other.allele) / 2_f32,
            ..*self
        }
    }
}

impl Debug for FloatGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
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
            min: f32::MIN,
            max: f32::MAX,
            upper_bound: f32::MAX,
            lower_bound: f32::MIN,
        }
    }
}

impl From<&f32> for FloatGene {
    fn from(allele: &f32) -> Self {
        FloatGene {
            allele: *allele,
            min: f32::MIN,
            max: f32::MAX,
            upper_bound: f32::MAX,
            lower_bound: f32::MIN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gene = FloatGene::new(0_f32, 1_f32);
        assert!(gene.is_valid());
    }

    #[test]
    fn test_into() {
        let gene = FloatGene::new(0_f32, 1_f32);
        let copy = gene.clone();
        let allele: f32 = gene.into();
        assert_eq!(allele, copy.allele);
    }

    #[test]
    fn test_from() {
        let gene = FloatGene::new(0_f32, 1_f32);
        let copy = gene.clone();
        assert_eq!(gene, copy);
    }

    #[test]
    fn test_is_valid() {
        let gene = FloatGene::new(0_f32, 1_f32).with_bounds(0.0, 1_f32);
        assert!(gene.is_valid());
        assert!(gene.allele >= 0_f32 && gene.allele <= 1_f32);
    }
}
