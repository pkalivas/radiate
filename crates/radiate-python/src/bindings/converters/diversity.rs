use crate::{InputTransform, PyEngineInput};
use radiate::{
    BitChromosome, CharChromosome, CosineDistance, Diversity, EuclideanDistance, FloatChromosome,
    GraphChromosome, HammingDistance, IntChromosome, NeatDistance, Op, PermutationChromosome,
    RadiateResult, TreeChromosome, chromosomes::NumericAllele, ops::OpFloat,
};
use radiate_error::radiate_bail;
use radiate_utils::{Float, Integer};

impl<I: Integer + NumericAllele> InputTransform<RadiateResult<Box<dyn Diversity<IntChromosome<I>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Diversity<IntChromosome<I>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            crate::constants::components::EUCLIDEAN_DISTANCE => Ok(Box::new(EuclideanDistance)),
            crate::constants::components::COSINE_DISTANCE => Ok(Box::new(CosineDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl<F: Float + NumericAllele> InputTransform<RadiateResult<Box<dyn Diversity<FloatChromosome<F>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Diversity<FloatChromosome<F>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            crate::constants::components::COSINE_DISTANCE => Ok(Box::new(CosineDistance)),
            crate::constants::components::EUCLIDEAN_DISTANCE => Ok(Box::new(EuclideanDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl InputTransform<RadiateResult<Box<dyn Diversity<BitChromosome>>>> for PyEngineInput {
    fn transform(&self) -> RadiateResult<Box<dyn Diversity<BitChromosome>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl InputTransform<RadiateResult<Box<dyn Diversity<CharChromosome>>>> for PyEngineInput {
    fn transform(&self) -> RadiateResult<Box<dyn Diversity<CharChromosome>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl InputTransform<RadiateResult<Box<dyn Diversity<PermutationChromosome<usize>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Diversity<PermutationChromosome<usize>>>> {
        match self.component() {
            crate::constants::components::HAMMING_DISTANCE => Ok(Box::new(HammingDistance)),
            _ => radiate_bail!(Builder: "Unknown diversity measure: {}", self.component()),
        }
    }
}

impl<F: OpFloat> InputTransform<RadiateResult<Box<dyn Diversity<TreeChromosome<Op<F>>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Diversity<TreeChromosome<Op<F>>>>> {
        // There are currently no diversity measures implemented for tree chromosomes
        radiate_bail!(Builder: "No diversity measures implemented for tree chromosomes")
    }
}

impl<F: OpFloat> InputTransform<RadiateResult<Box<dyn Diversity<GraphChromosome<Op<F>>>>>>
    for PyEngineInput
{
    fn transform(&self) -> RadiateResult<Box<dyn Diversity<GraphChromosome<Op<F>>>>> {
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
