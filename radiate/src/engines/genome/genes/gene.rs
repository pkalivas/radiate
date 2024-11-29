/// A `Valid` type is a type that can be checked for validity. This is used for checking if a gene
/// or a chromosome is valid. For example, a gene that represents a number between 0 and 1 can be checked
/// for validity by ensuring that the allele is between 0 and 1.
///
/// The `GeneticEngine` will check the validity of the `Chromosome` and `Phenotype` and remove any
/// invalid individuals from the population, replacing them with new individuals at the given generation.
pub trait Valid {
    fn is_valid(&self) -> bool {
        true
    }
}

/// A `Gene` is a single unit of information in a `Chromosome`.
/// This is the most basic building block of this entire library.
///
/// Any type that implements this trait can be used as a gene in a chromosome, as such
/// it can be used in any genetic algorithm that uses this library.
///
/// # Example
/// ```
/// use radiate::*;
///
/// // A simple gene that represents a number.
/// #[derive(Clone, Debug, PartialEq)]
/// struct MyGene {
///    allele: f32,
/// }
///
/// // Implement the Gene trait for the NumberGene.
/// impl Gene for MyGene {
///     type Allele = f32;
///
///     fn allele(&self) -> &Self::Allele {
///         &self.allele
///     }
///
///     fn new_instance(&self) -> Self {
///        MyGene { allele: 0.0 }
///     }
///
///    fn with_allele(&self, allele: &Self::Allele) -> Self {
///       MyGene { allele: *allele }
///     }
/// }
///
/// // You must also implement the `Valid` trait for the gene.
/// // This is used to check if the gene is valid. For example, a gene that represents a number between 0 and 1.
/// // The default implementation of the `Valid` trait is to return true.
/// impl Valid for MyGene {
///    fn is_valid(&self) -> bool {
///      self.allele >= 0.0 && self.allele <= 1.0
///   }
/// }
/// ```
///
pub trait Gene: Clone + PartialEq + Valid {
    type Allele;

    /// Get the `allele` of the `Gene`. This is the value that the `Gene` represents or "expresses".
    fn allele(&self) -> &Self::Allele;

    /// Create a new instance of the `Gene`.
    fn new_instance(&self) -> Self;

    /// Create a new `Gene` with the given `allele`.
    fn with_allele(&self, allele: &Self::Allele) -> Self;
}

/// A `Gene` that has bounds. This is useful for genes that represent numbers or other values that have
/// a range of valid values. For example, a gene that represents a number between 0 and 1.
pub trait BoundGene: Gene {
    fn upper_bound(&self) -> &Self::Allele;
    fn lower_bound(&self) -> &Self::Allele;
    fn with_bounds(self, upper_bound: Self::Allele, lower_bound: Self::Allele) -> Self;
}

/// A gene that represents a number. This gene can be used to represent any type of number, including
/// integers, floats, etc. Useful for using numeric mutations or crossover operations on numeric genes.
pub trait NumericGene: BoundGene {
    fn min(&self) -> &Self::Allele;
    fn max(&self) -> &Self::Allele;
    fn mean(&self, other: &Self) -> Self;
}
