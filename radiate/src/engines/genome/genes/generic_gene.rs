use std::sync::Arc;

use super::{Gene, Valid};

pub struct GenericGene<A: Clone + PartialEq> {
    pub allele: A,
    pub supplier: Arc<dyn Fn() -> A>,
}

impl<A: Clone + PartialEq> GenericGene<A> {
    pub fn new(allele: A, supplier: Arc<dyn Fn() -> A>) -> Self {
        Self { allele, supplier }
    }
}

impl<A: Clone + PartialEq> Gene<GenericGene<A>, A> for GenericGene<A> {
    fn allele(&self) -> &A {
        &self.allele
    }

    fn new_instance(&self) -> GenericGene<A> {
        GenericGene {
            allele: (self.supplier)(),
            supplier: Arc::clone(&self.supplier),
        }
    }

    fn from_allele(&self, allele: &A) -> GenericGene<A> {
        GenericGene {
            allele: allele.clone(),
            supplier: Arc::clone(&self.supplier),
        }
    }
}

impl<A: Clone + PartialEq> Clone for GenericGene<A> {
    fn clone(&self) -> Self {
        Self {
            allele: self.allele.clone(),
            supplier: Arc::clone(&self.supplier),
        }
    }
}

impl<A: Clone + PartialEq> Valid for GenericGene<A> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<A: Clone + PartialEq> PartialEq for GenericGene<A> {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}
