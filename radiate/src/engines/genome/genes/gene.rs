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
pub trait Gene<G, A>: Clone + PartialEq + Valid
where
    G: Gene<G, A>,
{
    /// Get the `allele` of the `Gene`. This is the value that the `Gene` represents or "expresses".
    fn allele(&self) -> &A;

    /// Create a new instance of the `Gene`.
    fn new_instance(&self) -> G;

    /// Create a new `Gene` with the given `allele`.
    fn from_allele(&self, allele: &A) -> G;
}

/// A `Gene` that has bounds. This is useful for genes that represent numbers or other values that have
/// a range of valid values. For example, a gene that represents a number between 0 and 1.
pub trait BoundGene<G, A>: Gene<G, A>
where
    G: BoundGene<G, A>,
{
    fn upper_bound(&self) -> &A;
    fn lower_bound(&self) -> &A;
    fn with_bounds(self, upper_bound: A, lower_bound: A) -> G;
}

/// A gene that represents a number. This gene can be used to represent any type of number, including
/// integers, floats, etc. Usefull for using numeric mutations or crossover operations on numeric genes.
pub trait NumericGene<G, A>: BoundGene<G, A>
where
    G: NumericGene<G, A>,
{
    fn add(&self, other: &G) -> G;
    fn sub(&self, other: &G) -> G;
    fn mul(&self, other: &G) -> G;
    fn div(&self, other: &G) -> G;
    fn mean(&self, other: &G) -> G;
}
