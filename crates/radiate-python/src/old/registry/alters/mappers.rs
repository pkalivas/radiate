use crate::{ParamMapper, PyEngineParam};
use radiate::prelude::*;

#[derive(Default)]
pub(crate) struct BlendCrossoverMapper<C: Chromosome<Gene = FloatGene>> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper for BlendCrossoverMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let alpha = param
            .get_arg("alpha")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let rate = param
            .get_arg("rate")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(BlendCrossover::new(rate, alpha))
    }
}

#[derive(Default)]
pub(crate) struct IntermediateCrossoverMapper<C: Chromosome<Gene = FloatGene>> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper for IntermediateCrossoverMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let alpha = param
            .get_arg("alpha")
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(IntermediateCrossover::new(rate, alpha))
    }
}

#[derive(Default)]
pub(crate) struct UniformCrossoverMapper<C: Chromosome> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome + 'static> ParamMapper for UniformCrossoverMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(UniformCrossover::new(rate))
    }
}

#[derive(Default)]
pub(crate) struct MeanCrossoverMapper<G: ArithmeticGene, C: Chromosome<Gene = G>> {
    _marker: std::marker::PhantomData<C>,
}

impl<G: ArithmeticGene, C: Chromosome<Gene = G> + 'static> ParamMapper
    for MeanCrossoverMapper<G, C>
{
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(MeanCrossover::new(rate))
    }
}

#[derive(Default)]
pub(crate) struct ShuffleCrossoverMapper<C: Chromosome> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome + Clone + 'static> ParamMapper for ShuffleCrossoverMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ShuffleCrossover::new(rate))
    }
}

#[derive(Default)]
pub(crate) struct SimulatedBinaryCrossoverMapper<C: Chromosome<Gene = FloatGene>> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper for SimulatedBinaryCrossoverMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let contiguty = param
            .get_arg("contiguty".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(SimulatedBinaryCrossover::new(contiguty, rate))
    }
}

#[derive(Default)]
pub(crate) struct MultiPointCrossoverMapper<C: Chromosome> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome + 'static> ParamMapper for MultiPointCrossoverMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        let points = param
            .get_arg("num_points".into())
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(2);
        alters!(MultiPointCrossover::new(rate, points))
    }
}

#[derive(Default)]
pub(crate) struct UniformMutatorMapper<C: Chromosome> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome + 'static> ParamMapper for UniformMutatorMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(UniformMutator::new(rate))
    }
}

#[derive(Default)]
pub(crate) struct ArithmeticMutatorMapper<G: ArithmeticGene, C: Chromosome<Gene = G>> {
    _marker: std::marker::PhantomData<C>,
}

impl<G: ArithmeticGene, C: Chromosome<Gene = G> + 'static> ParamMapper
    for ArithmeticMutatorMapper<G, C>
{
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ArithmeticMutator::new(rate))
    }
}

#[derive(Default)]
pub(crate) struct GaussianMutatorMapper<C: Chromosome<Gene = FloatGene>> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper for GaussianMutatorMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(GaussianMutator::new(rate))
    }
}

#[derive(Default)]
pub(crate) struct ScrambleMutatorMapper<C: Chromosome> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome + 'static> ParamMapper for ScrambleMutatorMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ScrambleMutator::new(rate))
    }
}

#[derive(Default)]
pub(crate) struct SwapMutatorMapper<C: Chromosome> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome + 'static> ParamMapper for SwapMutatorMapper<C> {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(SwapMutator::new(rate))
    }
}
