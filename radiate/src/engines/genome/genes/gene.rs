pub trait Valid {
    fn is_valid(&self) -> bool {
        true
    }
}

/// A gene is a single unit of information in a chromosome.
/// This is the most basic building block of this entire library.
pub trait Gene<G, A>: Clone + PartialEq + Valid
where
    G: Gene<G, A>,
{
    /// Get the allele of the gene. This is the value that the gene represents.
    fn allele(&self) -> &A;

    /// Create a new instance of the gene.
    fn new_instance(&self) -> G;

    /// Create a new gene with the given allele.
    fn from_allele(&self, allele: &A) -> G;
}

/// A gene that has bounds. This is useful for genes that represent numbers or other values that have
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
