use crate::PyEngineParam;
use radiate::{
    Alter, ArithmeticMutator, BlendCrossover, Chromosome, Crossover, FloatGene,
    IntermediateCrossover, Mutate, alters,
};

pub fn get_alters_with_arithmetic_gene<C: Chromosome<Gene = FloatGene>>(
    alters: Vec<PyEngineParam>,
) -> Vec<Box<dyn Alter<C>>>
where
    C: 'static,
{
    let mut alters_vec = Vec::new();

    for alter in alters {
        let args = alter.get_args();

        alters_vec.push(match alter.name() {
            "blend_crossover" => {
                let alpha = args
                    .get("alpha".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);

                alters!(BlendCrossover::new(rate, alpha))
            }
            "arithmetic_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ArithmeticMutator::new(rate))
            }
            "intermediate_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                let alpha = args
                    .get("alpha".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(IntermediateCrossover::new(rate, alpha))
            }
            _ => panic!("Unknown alter type"),
        });
    }

    alters_vec.into_iter().flatten().collect::<Vec<_>>()
}
