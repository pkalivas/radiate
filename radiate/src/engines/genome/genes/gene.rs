pub trait Valid {
    fn is_valid(&self) -> bool {
        true
    }
}

pub trait Gene<G, A>: Clone + PartialEq + Valid
where
    G: Gene<G, A>,
{
    fn allele(&self) -> &A;
    fn new_instance(&self) -> G;
    fn from_allele(&self, allele: &A) -> G;
}

pub trait BoundGene<G, A>: Gene<G, A>
where
    G: BoundGene<G, A>,
{
    fn upper_bound(&self) -> &A;
    fn lower_bound(&self) -> &A;
    fn with_bounds(self, upper_bound: A, lower_bound: A) -> G;
}

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
