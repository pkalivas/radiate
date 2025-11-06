use crate::{AnyChromosome, InputTransform, PyEngineInput};
use radiate::{
    BitChromosome, CharChromosome, CosineDistance, Diversity, EuclideanDistance, FloatChromosome,
    GraphChromosome, HammingDistance, IntChromosome, NeatDistance, Op, PermutationChromosome,
    TreeChromosome,
};

impl InputTransform<Option<Box<dyn Diversity<IntChromosome<i32>>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<IntChromosome<i32>>>> {
        match self.component.as_str() {
            crate::names::HAMMING_DISTANCE => Some(Box::new(HammingDistance)),
            _ => None,
        }
    }
}

impl InputTransform<Option<Box<dyn Diversity<FloatChromosome>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<FloatChromosome>>> {
        match self.component.as_str() {
            crate::names::EUCLIDEAN_DISTANCE => Some(Box::new(EuclideanDistance)),
            crate::names::HAMMING_DISTANCE => Some(Box::new(HammingDistance)),
            crate::names::COSINE_DISTANCE => Some(Box::new(CosineDistance)),
            _ => None,
        }
    }
}

impl InputTransform<Option<Box<dyn Diversity<BitChromosome>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<BitChromosome>>> {
        match self.component.as_str() {
            crate::names::HAMMING_DISTANCE => Some(Box::new(HammingDistance)),
            _ => None,
        }
    }
}

impl InputTransform<Option<Box<dyn Diversity<CharChromosome>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<CharChromosome>>> {
        match self.component.as_str() {
            crate::names::HAMMING_DISTANCE => Some(Box::new(HammingDistance)),
            _ => None,
        }
    }
}

impl InputTransform<Option<Box<dyn Diversity<PermutationChromosome<usize>>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<PermutationChromosome<usize>>>> {
        match self.component.as_str() {
            crate::names::HAMMING_DISTANCE => Some(Box::new(HammingDistance)),
            _ => None,
        }
    }
}

impl InputTransform<Option<Box<dyn Diversity<AnyChromosome<'static>>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<AnyChromosome<'static>>>> {
        match self.component.as_str() {
            crate::names::HAMMING_DISTANCE => Some(Box::new(HammingDistance)),
            _ => None,
        }
    }
}

impl InputTransform<Option<Box<dyn Diversity<TreeChromosome<Op<f32>>>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<TreeChromosome<Op<f32>>>>> {
        None
    }
}

impl InputTransform<Option<Box<dyn Diversity<GraphChromosome<Op<f32>>>>>> for PyEngineInput {
    fn transform(&self) -> Option<Box<dyn Diversity<GraphChromosome<Op<f32>>>>> {
        match self.component.as_str() {
            crate::names::NEAT_DISTANCE => {
                let excess = self.get_f32("excess").unwrap_or(1.0);
                let disjoint = self.get_f32("disjoint").unwrap_or(1.0);
                let weight_diff = self.get_f32("weight_diff").unwrap_or(0.4);
                Some(Box::new(NeatDistance::new(excess, disjoint, weight_diff)))
            }
            crate::names::HAMMING_DISTANCE => Some(Box::new(HammingDistance)),
            _ => None,
        }
    }
}
