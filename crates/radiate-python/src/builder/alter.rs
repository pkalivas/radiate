use crate::PyEngineParam;
use core::panic;
use radiate::{
    Alter, ArithmeticGene, ArithmeticMutator, BlendCrossover, Chromosome, Crossover, FloatGene,
    GaussianMutator, Gene, GeneticEngineBuilder, IntermediateCrossover, MeanCrossover,
    MultiPointCrossover, Mutate, ScrambleMutator, ShuffleCrossover, SimulatedBinaryCrossover,
    SwapMutator, UniformCrossover, UniformMutator, alters,
};

const BLEND_CROSSOVER: &str = "blend_crossover";
const INTERMEDIATE_CROSSOVER: &str = "intermediate_crossover";
const UNIFORM_CROSSOVER: &str = "uniform_crossover";
const MEAN_CROSSOVER: &str = "mean_crossover";
const SHUFFLE_CROSSOVER: &str = "shuffle_crossover";
const MULTI_POINT_CROSSOVER: &str = "multi_point_crossover";
const SIMULATED_BINARY_CROSSOVER: &str = "simulated_binary_crossover";
const UNIFORM_MUTATOR: &str = "uniform_mutator";
const ARITHMETIC_MUTATOR: &str = "arithmetic_mutator";
const GAUSSIAN_MUTATOR: &str = "gaussian_mutator";
const SCRAMBLE_MUTATOR: &str = "scramble_mutator";
const SWAP_MUTATOR: &str = "swap_mutator";

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
            BLEND_CROSSOVER => {
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
            INTERMEDIATE_CROSSOVER => {
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
            UNIFORM_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformCrossover::new(rate))
            }
            UNIFORM_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformMutator::new(rate))
            }
            ARITHMETIC_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ArithmeticMutator::new(rate))
            }
            MEAN_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(MeanCrossover::new(rate))
            }
            SHUFFLE_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ShuffleCrossover::new(rate))
            }
            SIMULATED_BINARY_CROSSOVER => {
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
            GAUSSIAN_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(GaussianMutator::new(rate))
            }
            SCRAMBLE_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ScrambleMutator::new(rate))
            }
            SWAP_MUTATOR => {
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
            MULTI_POINT_CROSSOVER => {
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
            UNIFORM_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformCrossover::new(rate))
            }
            UNIFORM_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformMutator::new(rate))
            }
            ARITHMETIC_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ArithmeticMutator::new(rate))
            }
            MEAN_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(MeanCrossover::new(rate))
            }
            SHUFFLE_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ShuffleCrossover::new(rate))
            }
            SCRAMBLE_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ScrambleMutator::new(rate))
            }
            SWAP_MUTATOR => {
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
            MULTI_POINT_CROSSOVER => {
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
            UNIFORM_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformCrossover::new(rate))
            }
            UNIFORM_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(UniformMutator::new(rate))
            }
            SHUFFLE_CROSSOVER => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ShuffleCrossover::new(rate))
            }
            SCRAMBLE_MUTATOR => {
                let rate = args
                    .get("rate".into())
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.5);
                alters!(ScrambleMutator::new(rate))
            }
            SWAP_MUTATOR => {
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

// type AlterMapper<C> = Box<dyn ParamMapper<C, Output = Vec<Box<dyn Alter<C> + 'static>>>>;

// pub struct EngineRegistry {
//     pub float_alters: HashMap<String, Vec<AlterMapper<FloatChromosome>>>,
//     pub int_alters: HashMap<String, Vec<AlterMapper<IntChromosome<i32>>>>,
//     pub char_alters: HashMap<String, Vec<AlterMapper<CharChromosome>>>,
// }

// impl EngineRegistry {
//     pub fn new() -> Self {
//         let mut registry = EngineRegistry {
//             float_alters: HashMap::new(),
//             int_alters: HashMap::new(),
//             char_alters: HashMap::new(),
//         };

//         // Register default float alters
//         registry.register_float_alter_mapper(BLEND_CROSSOVER, BlendCrossoverMapper);
//         registry.register_float_alter_mapper(INTERMEDIATE_CROSSOVER, IntermediateCrossoverMapper);
//         registry.register_float_alter_mapper(UNIFORM_CROSSOVER, UniformCrossoverMapper);
//         registry.register_float_alter_mapper(MEAN_CROSSOVER, MeanCrossoverMapper);
//         registry.register_float_alter_mapper(SHUFFLE_CROSSOVER, ShuffleCrossoverMapper);
//         registry.register_float_alter_mapper(MULTI_POINT_CROSSOVER, MultiPointCrossoverMapper);
//         registry.register_float_alter_mapper(UNIFORM_MUTATOR, UniformMutatorMapper);
//         registry.register_float_alter_mapper(ARITHMETIC_MUTATOR, ArithmeticMutatorMapper);
//         registry.register_float_alter_mapper(GAUSSIAN_MUTATOR, GaussianMutatorMapper);
//         registry.register_float_alter_mapper(SCRAMBLE_MUTATOR, ScrambleMutatorMapper);
//         registry.register_float_alter_mapper(SWAP_MUTATOR, SwapMutatorMapper);
//         registry.register_float_alter_mapper(
//             SIMULATED_BINARY_CROSSOVER,
//             SimulatedBinaryCrossoverMapper,
//         );

//         // Register default int alters
//         registry.register_int_alter_mapper(MULTI_POINT_CROSSOVER, MultiPointCrossoverMapper);
//         registry.register_int_alter_mapper(UNIFORM_CROSSOVER, UniformCrossoverMapper);
//         registry.register_int_alter_mapper(MEAN_CROSSOVER, MeanCrossoverMapper);
//         registry.register_int_alter_mapper(SHUFFLE_CROSSOVER, ShuffleCrossoverMapper);
//         registry.register_int_alter_mapper(UNIFORM_MUTATOR, UniformMutatorMapper);
//         registry.register_int_alter_mapper(ARITHMETIC_MUTATOR, ArithmeticMutatorMapper);
//         registry.register_int_alter_mapper(SCRAMBLE_MUTATOR, ScrambleMutatorMapper);
//         registry.register_int_alter_mapper(SWAP_MUTATOR, SwapMutatorMapper);

//         // Register default char alters
//         registry.register_char_alter_mapper(MULTI_POINT_CROSSOVER, MultiPointCrossoverMapper);
//         registry.register_char_alter_mapper(UNIFORM_CROSSOVER, UniformCrossoverMapper);
//         registry.register_char_alter_mapper(SHUFFLE_CROSSOVER, ShuffleCrossoverMapper);
//         registry.register_char_alter_mapper(UNIFORM_MUTATOR, UniformMutatorMapper);
//         registry.register_char_alter_mapper(SCRAMBLE_MUTATOR, ScrambleMutatorMapper);
//         registry.register_char_alter_mapper(SWAP_MUTATOR, SwapMutatorMapper);

//         registry
//     }

//     pub fn get_alters<C: Chromosome>(&self, params: &[PyEngineParam]) -> Vec<Box<dyn Alter<C>>> {
//         let c_type = std::any::type_name::<C>()
//             .split("::")
//             .last()
//             .map(|maybe_name| {
//                 maybe_name
//                     .split("<")
//                     .into_iter()
//                     .map(|val| val.to_string())
//                     .collect::<Vec<String>>()
//             })
//             .map(|val| val.first().cloned())
//             .flatten();

//         println!("{:?}", c_type);

//         match c_type {
//             Some(c_val) => {
//                 if c_val == "IntChromosome" {
//                     let mut alters = Vec::new();
//                     for param in params {
//                         if let Some(mappers) = self.int_alters.get(param.name()) {
//                             for mapper in mappers {
//                                 alters.extend(mapper.map(param));
//                             }
//                         }
//                     }

//                     // return alters;
//                 } else {
//                     panic!("Unknown chromosome type: {}", c_val);
//                 }
//             }
//             _ => panic!(""),
//         }

//         panic!()
//     }

//     pub fn register_float_alter_mapper<
//         M: ParamMapper<FloatChromosome, Output = Vec<Box<dyn Alter<FloatChromosome>>>> + 'static,
//     >(
//         &mut self,
//         name: &str,
//         mapper: M,
//     ) {
//         self.float_alters
//             .entry(name.to_string())
//             .or_insert_with(Vec::new)
//             .push(Box::new(mapper));
//     }

//     pub fn register_int_alter_mapper<
//         M: ParamMapper<IntChromosome<i32>, Output = Vec<Box<dyn Alter<IntChromosome<i32>>>>> + 'static,
//     >(
//         &mut self,
//         name: &str,
//         mapper: M,
//     ) {
//         self.int_alters
//             .entry(name.to_string())
//             .or_insert_with(Vec::new)
//             .push(Box::new(mapper));
//     }

//     pub fn register_char_alter_mapper<
//         M: ParamMapper<CharChromosome, Output = Vec<Box<dyn Alter<CharChromosome>>>> + 'static,
//     >(
//         &mut self,
//         name: &str,
//         mapper: M,
//     ) {
//         self.char_alters
//             .entry(name.to_string())
//             .or_insert_with(Vec::new)
//             .push(Box::new(mapper));
//     }
// }

// pub trait ParamMapper<C: Chromosome + 'static> {
//     type Output;
//     fn map(&self, param: &PyEngineParam) -> Self::Output;
// }

// struct BlendCrossoverMapper;

// impl<C> ParamMapper<C> for BlendCrossoverMapper
// where
//     C: Chromosome<Gene = FloatGene> + 'static,
// {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let alpha = param
//             .get_arg("alpha")
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         let rate = param
//             .get_arg("rate")
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(BlendCrossover::new(rate, alpha))
//     }
// }

// struct IntermediateCrossoverMapper;

// impl ParamMapper<FloatChromosome> for IntermediateCrossoverMapper {
//     type Output = Vec<Box<dyn Alter<FloatChromosome>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate")
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         let alpha = param
//             .get_arg("alpha")
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(IntermediateCrossover::new(rate, alpha))
//     }
// }

// struct UniformCrossoverMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for UniformCrossoverMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(UniformCrossover::new(rate))
//     }
// }

// struct MeanCrossoverMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for MeanCrossoverMapper
// where
//     C::Gene: ArithmeticGene,
// {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(MeanCrossover::new(rate))
//     }
// }

// struct ShuffleCrossoverMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for ShuffleCrossoverMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(ShuffleCrossover::new(rate))
//     }
// }

// struct SimulatedBinaryCrossoverMapper;

// impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper<C> for SimulatedBinaryCrossoverMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         let contiguty = param
//             .get_arg("contiguty".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(SimulatedBinaryCrossover::new(contiguty, rate))
//     }
// }

// struct MultiPointCrossoverMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for MultiPointCrossoverMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         let points = param
//             .get_arg("num_points".into())
//             .map(|s| s.parse::<usize>().unwrap())
//             .unwrap_or(2);
//         alters!(MultiPointCrossover::new(rate, points))
//     }
// }

// struct UniformMutatorMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for UniformMutatorMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(UniformMutator::new(rate))
//     }
// }

// struct ArithmeticMutatorMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for ArithmeticMutatorMapper
// where
//     C::Gene: ArithmeticGene,
// {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(ArithmeticMutator::new(rate))
//     }
// }

// struct GaussianMutatorMapper;

// impl<C: Chromosome<Gene = FloatGene> + 'static> ParamMapper<C> for GaussianMutatorMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(GaussianMutator::new(rate))
//     }
// }

// struct ScrambleMutatorMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for ScrambleMutatorMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(ScrambleMutator::new(rate))
//     }
// }

// struct SwapMutatorMapper;

// impl<C: Chromosome + 'static> ParamMapper<C> for SwapMutatorMapper {
//     type Output = Vec<Box<dyn Alter<C>>>;
//     fn map(&self, param: &PyEngineParam) -> Self::Output {
//         let rate = param
//             .get_arg("rate".into())
//             .map(|s| s.parse::<f32>().unwrap())
//             .unwrap_or(0.5);
//         alters!(SwapMutator::new(rate))
//     }
// }
