use std::ops::{Add, Div, Mul, Sub};

/// A [`Valid`] type is a type that can be checked for validity. This is used for checking if a gene
/// or a chromosome is valid. For example, a gene that represents a number between 0 and 1 can be checked
/// for validity by ensuring that the allele is between 0 and 1.
///
/// The `GeneticEngine` will check the validity of the [Chromosome](super::chromosome::Chromosome) and `Phenotype` and remove any
/// invalid individuals from the population, replacing them with new individuals at the given generation.
pub trait Valid {
    fn is_valid(&self) -> bool {
        true
    }
}

/// A [`Gene`] is a single unit of information in a [Chromosome](super::chromosome::Chromosome).
/// This is the most basic building block of this entire library.
///
/// Any type that implements this trait can be used as a gene in a chromosome, as such
/// it can be used in any genetic algorithm that uses this library.
///
/// # Example
/// ```
/// use radiate_core::*;
///
/// // A simple gene that represents a point.
/// #[derive(Clone, Debug, PartialEq)]
/// struct PointGene {
///    allele: (f32, f32),
/// }
///
/// // Implement the Gene trait for the PointGene.
/// impl Gene for PointGene {
///     type Allele = (f32, f32);
///
///     fn allele(&self) -> &Self::Allele {
///         &self.allele
///     }
///
///     fn allele_mut(&mut self) -> &mut Self::Allele {
///         &mut self.allele
///     }
///
///     fn new_instance(&self) -> Self {
///        PointGene { allele: (0.0, 0.0) }
///     }
///
///     fn with_allele(&self, allele: &Self::Allele) -> Self {
///       PointGene { allele: *allele }
///     }
/// }
///
/// // You must also implement the [`Valid`] trait for the gene.
/// // The default implementation of the [`Valid`] trait is to return true.
/// impl Valid for PointGene {
///    fn is_valid(&self) -> bool {
///      let (x, y) = self.allele;
///     // Check if the x and y values are between 0 and 1.
///     x >= 0.0 && x <= 1.0 && y >= 0.0 && y <= 1.0
///   }
/// }
/// ```
pub trait Gene: Clone + Valid {
    type Allele;

    /// Get the `allele` of the [Gene]. This is the value that the [Gene] represents or "expresses".
    fn allele(&self) -> &Self::Allele;

    /// Get a mutable reference to the `allele` of the [Gene].
    fn allele_mut(&mut self) -> &mut Self::Allele;

    /// Create a new instance of the [Gene].
    fn new_instance(&self) -> Self;

    /// Create a new [Gene] with the given `allele`.
    fn with_allele(&self, allele: &Self::Allele) -> Self;
}

pub trait BoundedGene: Gene {
    fn min(&self) -> &Self::Allele;
    fn max(&self) -> &Self::Allele;
    fn bounds(&self) -> (&Self::Allele, &Self::Allele);
}

/// A [Gene] that represents a number. This gene can be used to represent any type of number,
/// including integers, floats, etc. Essentially, any gene that can `Add`, `Sub`, `Mul`, and `Div`
/// can be used as a [ArithmeticGene].
pub trait ArithmeticGene:
    Gene + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self>
{
    fn mean(&self, other: &Self) -> Self;
}
