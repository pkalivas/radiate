use crate::{InputTransform, PyEngineInput};
use radiate::{
    BitChromosome, CharChromosome, CosineDistance, EuclideanDistance, FloatChromosome,
    GraphChromosome, HammingDistance, IntChromosome, NeatDistance, Op, PackedBitChromosome,
    PackedBitHammingDistance, PermutationChromosome, Phenotype, RadiateResult, TreeChromosome,
    chromosomes::NumericAllele, distance::Distance, ops::OpFloat,
};
use radiate_error::radiate_bail;
use radiate_utils::{Float, Integer};

impl<I: Integer + NumericAllele>
    InputTransform<RadiateResult<Box<dyn Distance<Phenotype<IntChromosome<I>>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Distance<Phenotype<IntChromosome<I>>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            crate::constants::components::EUCLIDEAN_DISTANCE => Ok(Box::new(EuclideanDistance)),
            crate::constants::components::COSINE_DISTANCE => Ok(Box::new(CosineDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl<F: Float + NumericAllele>
    InputTransform<RadiateResult<Box<dyn Distance<Phenotype<FloatChromosome<F>>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Distance<Phenotype<FloatChromosome<F>>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            crate::constants::components::COSINE_DISTANCE => Ok(Box::new(CosineDistance)),
            crate::constants::components::EUCLIDEAN_DISTANCE => Ok(Box::new(EuclideanDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl InputTransform<RadiateResult<Box<dyn Distance<Phenotype<BitChromosome>>>>> for PyEngineInput {
    fn transform(&self) -> RadiateResult<Box<dyn Distance<Phenotype<BitChromosome>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl InputTransform<RadiateResult<Box<dyn Distance<Phenotype<PackedBitChromosome>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Distance<Phenotype<PackedBitChromosome>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => {
                Ok(Box::new(PackedBitHammingDistance))
            }
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl InputTransform<RadiateResult<Box<dyn Distance<Phenotype<CharChromosome>>>>> for PyEngineInput {
    fn transform(&self) -> RadiateResult<Box<dyn Distance<Phenotype<CharChromosome>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl InputTransform<RadiateResult<Box<dyn Distance<Phenotype<PermutationChromosome<usize>>>>>>
    for PyEngineInput
{
    fn transform(
        &self,
    ) -> RadiateResult<Box<dyn Distance<Phenotype<PermutationChromosome<usize>>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl<F: OpFloat> InputTransform<RadiateResult<Box<dyn Distance<Phenotype<TreeChromosome<Op<F>>>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Distance<Phenotype<TreeChromosome<Op<F>>>>>> {
        // There are currently no diversity measures implemented for tree chromosomes
        radiate_bail!(Builder: "No diversity measures implemented for tree chromosomes")
    }
}

impl<F: OpFloat> InputTransform<RadiateResult<Box<dyn Distance<Phenotype<GraphChromosome<Op<F>>>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Distance<Phenotype<GraphChromosome<Op<F>>>>>> {
        match self.component() {
            crate::constants::components::NEAT_DISTANCE => {
                let excess = self.extract::<f64>("excess")?;
                let disjoint = self.extract::<f64>("disjoint")?;
                let weight_diff = self.extract::<f64>("weight_diff")?;

                Ok(Box::new(NeatDistance::new(
                    excess as f32,
                    disjoint as f32,
                    weight_diff as f32,
                )))
            }
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}
