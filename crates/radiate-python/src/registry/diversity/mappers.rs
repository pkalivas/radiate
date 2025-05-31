use crate::{ParamMapper, PyEngineParam};
use radiate::{ArithmeticGene, Chromosome, Diversity, EuclideanDistance, Gene, HammingDistance};
use std::sync::Arc;

pub struct HammingDistanceDiversityMapper<G: Gene, C: Chromosome<Gene = G>>
where
    G::Allele: PartialEq,
{
    _marker: std::marker::PhantomData<C>,
}

impl<G, C> ParamMapper for HammingDistanceDiversityMapper<G, C>
where
    C: Chromosome<Gene = G>,
    G: Gene,
    G::Allele: PartialEq,
{
    type Output = Arc<dyn Diversity<C>>;
    fn map(&self, _: &PyEngineParam) -> Self::Output {
        Arc::new(HammingDistance)
    }
}

impl<G: Gene + Default, C: Chromosome<Gene = G>> Default for HammingDistanceDiversityMapper<G, C>
where
    G::Allele: PartialEq,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct EuclideanDistanceDiversityMapper<G: Gene, C: Chromosome<Gene = G>>
where
    G::Allele: Into<f32> + Copy,
{
    _marker: std::marker::PhantomData<C>,
}

impl<G, C> ParamMapper for EuclideanDistanceDiversityMapper<G, C>
where
    C: Chromosome<Gene = G>,
    G: Gene + ArithmeticGene,
    G::Allele: Into<f32> + Copy,
{
    type Output = Arc<dyn Diversity<C>>;
    fn map(&self, _: &PyEngineParam) -> Self::Output {
        Arc::new(EuclideanDistance)
    }
}

impl<G: Gene + Default, C: Chromosome<Gene = G>> Default for EuclideanDistanceDiversityMapper<G, C>
where
    G::Allele: Into<f32> + Copy,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}
