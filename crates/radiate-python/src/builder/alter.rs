use crate::PyEngineParam;
use radiate::{
    Alter, ArithmeticGene, ArithmeticMutator, BlendCrossover, CharChromosome, Chromosome,
    Crossover, FloatChromosome, FloatGene, GaussianMutator, Gene, GeneticEngineBuilder,
    IntChromosome, IntermediateCrossover, MeanCrossover, MultiPointCrossover, Mutate,
    ScrambleMutator, ShuffleCrossover, SimulatedBinaryCrossover, SwapMutator, UniformCrossover,
    UniformMutator, alters,
};
use std::collections::HashMap;

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
            "mean_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(MeanCrossover::new(rate))
            }
            "shuffle_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ShuffleCrossover::new(rate))
            }
            "simulated_binary_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                let contiguty = args
                    .get("contiguty".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(SimulatedBinaryCrossover::new(contiguty, rate))
            }
            "gaussian_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(GaussianMutator::new(rate))
            }
            "scramble_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ScrambleMutator::new(rate))
            }
            "swap_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(SwapMutator::new(rate))
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
            "mean_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(MeanCrossover::new(rate))
            }
            "shuffle_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ShuffleCrossover::new(rate))
            }
            "scramble_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ScrambleMutator::new(rate))
            }
            "swap_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(SwapMutator::new(rate))
            }
            _ => panic!("Unknown alter type"),
        });
    }

    let alters = alters_vec.into_iter().flatten().collect::<Vec<_>>();
    builder.alter(alters)
}

pub fn get_alters_with_char_gene<C, G, T>(
    builder: GeneticEngineBuilder<C, T>,
    alters: &Vec<PyEngineParam>,
) -> GeneticEngineBuilder<C, T>
where
    C: Chromosome<Gene = G> + 'static,
    T: Clone + Send + Sync,
    G: Gene,
    G::Allele: Clone,
{
    let mut alters_vec = Vec::new();

    for alter in alters {
        let args = alter.get_args();

        alters_vec.push(match alter.name() {
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
            "shuffle_crossover" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ShuffleCrossover::new(rate))
            }
            "scramble_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ScrambleMutator::new(rate))
            }
            "swap_mutator" => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(SwapMutator::new(rate))
            }
            _ => panic!("Unknown alter type"),
        });
    }

    let alters = alters_vec.into_iter().flatten().collect::<Vec<_>>();
    builder.alter(alters)
}

type AlterMapper<C> = Box<dyn ParamMapper<C, Output = Vec<Box<dyn Alter<C> + 'static>>>>;

pub struct EngineRegistry {
    pub float_alters: HashMap<String, Vec<AlterMapper<FloatChromosome>>>,
    pub int_alters: HashMap<String, Vec<AlterMapper<IntChromosome<i32>>>>,
    pub char_alters: HashMap<String, Vec<AlterMapper<CharChromosome>>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        let mut registry = EngineRegistry {
            float_alters: HashMap::new(),
            int_alters: HashMap::new(),
            char_alters: HashMap::new(),
        };

        // Register default float alters
        registry.register_float_alter_mapper("blend_crossover", BlendCrossoverMapper);
        registry.register_float_alter_mapper("intermediate_crossover", IntermediateCrossoverMapper);
        registry.register_float_alter_mapper("uniform_crossover", UniformCrossoverMapper);
        registry.register_float_alter_mapper("mean_crossover", MeanCrossoverMapper);
        registry.register_float_alter_mapper("shuffle_crossover", ShuffleCrossoverMapper);
        registry.register_float_alter_mapper("multi_point_crossover", MultiPointCrossoverMapper);
        registry.register_float_alter_mapper(
            "simulated_binary_crossover",
            SimulatedBinaryCrossoverMapper,
        );
        registry.register_float_alter_mapper("uniform_mutator", UniformMutatorMapper);
        registry.register_float_alter_mapper("arithmetic_mutator", ArithmeticMutatorMapper);
        registry.register_float_alter_mapper("gaussian_mutator", GaussianMutatorMapper);
        registry.register_float_alter_mapper("scramble_mutator", ScrambleMutatorMapper);
        registry.register_float_alter_mapper("swap_mutator", SwapMutatorMapper);

        // Register default int alters
        registry.register_int_alter_mapper("multi_point_crossover", MultiPointCrossoverMapper);
        registry.register_int_alter_mapper("uniform_crossover", UniformCrossoverMapper);
        registry.register_int_alter_mapper("mean_crossover", MeanCrossoverMapper);
        registry.register_int_alter_mapper("shuffle_crossover", ShuffleCrossoverMapper);
        registry.register_int_alter_mapper("uniform_mutator", UniformMutatorMapper);
        registry.register_int_alter_mapper("arithmetic_mutator", ArithmeticMutatorMapper);
        registry.register_int_alter_mapper("scramble_mutator", ScrambleMutatorMapper);
        registry.register_int_alter_mapper("swap_mutator", SwapMutatorMapper);

        // Register default char alters
        registry.register_char_alter_mapper("multi_point_crossover", MultiPointCrossoverMapper);
        registry.register_char_alter_mapper("uniform_crossover", UniformCrossoverMapper);
        registry.register_char_alter_mapper("shuffle_crossover", ShuffleCrossoverMapper);
        registry.register_char_alter_mapper("uniform_mutator", UniformMutatorMapper);
        registry.register_char_alter_mapper("scramble_mutator", ScrambleMutatorMapper);
        registry.register_char_alter_mapper("swap_mutator", SwapMutatorMapper);

        registry
    }

    pub fn get_float_alters(
        &self,
        params: &[PyEngineParam],
    ) -> Vec<Box<dyn Alter<FloatChromosome>>> {
        let mut alters = Vec::new();
        for param in params {
            if let Some(mappers) = self.float_alters.get(param.name()) {
                for mapper in mappers {
                    alters.extend(mapper.map(param));
                }
            }
        }

        alters
    }

    pub fn register_float_alter_mapper<
        M: ParamMapper<FloatChromosome, Output = Vec<Box<dyn Alter<FloatChromosome>>>> + 'static,
    >(
        &mut self,
        name: &str,
        mapper: M,
    ) {
        self.float_alters
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(mapper));
    }

    pub fn register_int_alter_mapper<
        M: ParamMapper<IntChromosome<i32>, Output = Vec<Box<dyn Alter<IntChromosome<i32>>>>> + 'static,
    >(
        &mut self,
        name: &str,
        mapper: M,
    ) {
        self.int_alters
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(mapper));
    }

    pub fn register_char_alter_mapper<
        M: ParamMapper<CharChromosome, Output = Vec<Box<dyn Alter<CharChromosome>>>> + 'static,
    >(
        &mut self,
        name: &str,
        mapper: M,
    ) {
        self.char_alters
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(mapper));
    }
}

pub trait ParamMapper<C: Chromosome + 'static> {
    type Output;
    fn map(&self, param: &PyEngineParam) -> Self::Output;
}

struct BlendCrossoverMapper;

impl ParamMapper<FloatChromosome> for BlendCrossoverMapper {
    type Output = Vec<Box<dyn Alter<FloatChromosome>>>;
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

struct IntermediateCrossoverMapper;

impl ParamMapper<FloatChromosome> for IntermediateCrossoverMapper {
    type Output = Vec<Box<dyn Alter<FloatChromosome>>>;
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

struct UniformCrossoverMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for UniformCrossoverMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(UniformCrossover::new(rate))
    }
}

struct MeanCrossoverMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for MeanCrossoverMapper
where
    C::Gene: ArithmeticGene,
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

struct ShuffleCrossoverMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for ShuffleCrossoverMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ShuffleCrossover::new(rate))
    }
}

struct SimulatedBinaryCrossoverMapper;

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper<C> for SimulatedBinaryCrossoverMapper {
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

struct MultiPointCrossoverMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for MultiPointCrossoverMapper {
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

struct UniformMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for UniformMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(UniformMutator::new(rate))
    }
}

struct ArithmeticMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for ArithmeticMutatorMapper
where
    C::Gene: ArithmeticGene,
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

struct GaussianMutatorMapper;

impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper<C> for GaussianMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(GaussianMutator::new(rate))
    }
}

struct ScrambleMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for ScrambleMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(ScrambleMutator::new(rate))
    }
}

struct SwapMutatorMapper;

impl<C: Chromosome + 'static> ParamMapper<C> for SwapMutatorMapper {
    type Output = Vec<Box<dyn Alter<C>>>;
    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let rate = param
            .get_arg("rate".into())
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(0.5);
        alters!(SwapMutator::new(rate))
    }
}
