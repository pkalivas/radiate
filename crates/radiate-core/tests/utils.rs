use radiate_core::*;
use radiate_error::Result;

pub fn float_ecosystem() -> Ecosystem<FloatChromosome<f32>> {
    Ecosystem::new(Population::from(vec![
        Phenotype::from((vec![FloatChromosome::from(vec![1.0])], 0)),
        Phenotype::from((vec![FloatChromosome::from(vec![2.0])], 0)),
        Phenotype::from((vec![FloatChromosome::from(vec![3.0])], 0)),
    ]))
}

pub struct FloatEvalProblem;

impl Problem<FloatChromosome<f32>, f32> for FloatEvalProblem {
    fn encode(&self) -> Genotype<FloatChromosome<f32>> {
        unreachable!()
    }

    fn decode(&self, _: &Genotype<FloatChromosome<f32>>) -> f32 {
        unreachable!()
    }

    fn eval(&self, individual: &Genotype<FloatChromosome<f32>>) -> Result<Score> {
        Ok(Score::from(*individual[0].get(0).allele()))
    }
}
