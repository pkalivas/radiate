use crate::PyEngineParam;
use radiate::{
    Alter, ArithmeticGene, ArithmeticMutator, BlendCrossover, Chromosome, Crossover, FloatGene,
    GeneticEngineBuilder, IntermediateCrossover, MultiPointCrossover, Mutate, UniformCrossover,
    UniformMutator, alters,
};

pub fn get_alters_with_float_gene<C: Chromosome<Gene = FloatGene>, T>(
    builder: GeneticEngineBuilder<C, T>,
    alters: &Vec<PyEngineParam>,
) -> GeneticEngineBuilder<C, T>
where
    C: 'static,
    T: Clone + Send + Sync,
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
            "uniform_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformCrossover::new(rate))
            }
            "uniform_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformMutator::new(rate))
            }
            "arithmetic_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ArithmeticMutator::new(rate))
            }
            _ => panic!("Unknown alter type"),
        });
    }

    let alters = alters_vec.into_iter().flatten().collect::<Vec<_>>();
    builder.alter(alters)
}

pub fn get_alters_with_int_gene<C, G, T>(
    builder: GeneticEngineBuilder<C, T>,
    alters: &Vec<PyEngineParam>,
) -> GeneticEngineBuilder<C, T>
where
    C: Chromosome<Gene = G> + 'static,
    T: Clone + Send + Sync,
    G: ArithmeticGene + Clone,
    G::Allele: Clone,
{
    let mut alters_vec = Vec::new();

    for alter in alters {
        let args = alter.get_args();

        alters_vec.push(match alter.name() {
            // "blend_crossover" => {
            //     let alpha = args
            //         .get("alpha".into())
            //         .map(|s| s.parse::<f32>().unwrap())
            //         .unwrap_or(0.5);
            //     let rate = args
            //         .get("rate".into())
            //         .map(|s| s.parse::<f32>().unwrap())
            //         .unwrap_or(0.5);

            //     alters!(BlendCrossover::new(rate, alpha))
            // }
            "multi_point_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                let points = args
                    .get("num_points".into())
                    .map(|s| s.parse::<usize>().unwrap())
                    .unwrap_or(2);
                alters!(MultiPointCrossover::new(rate, points))
            }
            "uniform_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformCrossover::new(rate))
            }
            "uniform_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformMutator::new(rate))
            }
            "arithmetic_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ArithmeticMutator::new(rate))
            }
            _ => panic!("Unknown alter type"),
        });
    }

    let alters = alters_vec.into_iter().flatten().collect::<Vec<_>>();
    builder.alter(alters)
}
