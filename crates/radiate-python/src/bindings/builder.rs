use crate::bindings::codec::PyTreeCodec;
use crate::bindings::{EngineBuilderHandle, EngineHandle};
use crate::events::PyEventHandler;
use crate::{AnyChromosome, PySubscriber};
use crate::{
    FreeThreadPyEvaluator, InputTransform, PyCodec, PyEngine, PyEngineInput, PyEngineInputType,
    PyFitnessFn, PyFitnessInner, PyPermutationCodec, PyPopulation, prelude::*,
};
use pyo3::exceptions::PyTypeError;
use pyo3::{Py, PyAny, pyclass, pymethods, types::PyAnyMethods};
use radiate::{RadiateResult, prelude::*};
use radiate_error::{radiate_bail, radiate_py_bail, radiate_py_err};
use std::collections::HashMap;

macro_rules! apply_to_builder {
    ($builder:expr, $method:ident($($args:expr),*)) => {
        match $builder {
            EngineBuilderHandle::Int(b) => Ok(EngineBuilderHandle::Int(b.$method($($args),*))),
            EngineBuilderHandle::Float(b) => Ok(EngineBuilderHandle::Float(b.$method($($args),*))),
            EngineBuilderHandle::Char(b) => Ok(EngineBuilderHandle::Char(b.$method($($args),*))),
            EngineBuilderHandle::Bit(b) => Ok(EngineBuilderHandle::Bit(b.$method($($args),*))),
            EngineBuilderHandle::Permutation(b) => Ok(EngineBuilderHandle::Permutation(b.$method($($args),*))),
            EngineBuilderHandle::Graph(b) => Ok(EngineBuilderHandle::Graph(b.$method($($args),*))),
            EngineBuilderHandle::Tree(b) => Ok(EngineBuilderHandle::Tree(b.$method($($args),*))),
            EngineBuilderHandle::Any(b) => Ok(EngineBuilderHandle::Any(b.$method($($args),*))),
            EngineBuilderHandle::Empty => Err(radiate_py_err!(
                "Cannot apply method to Empty builder"
            )),
        }
    };
}

macro_rules! dispatch_builder_typed {

    // ------------------------------------------------------------
    // public: single input
    // applier: fn(&PyEngineInput, GeneticEngineBuilder<...>) -> PyResult<GeneticEngineBuilder<...>>
    // ------------------------------------------------------------
    ($builder:expr, $input:expr, $applier:expr) => {{
        dispatch_builder_typed!(@do $builder, |variant_builder| {
            ($applier)(variant_builder, $input)
        })
    }};

    // ------------------------------------------------------------
    // private core: actually match all variants once
    // ------------------------------------------------------------
    (@do $builder:expr, $call:expr) => {{
        match $builder {
            EngineBuilderHandle::Int(b) => {
                $call(b).map(EngineBuilderHandle::Int)
            }
            EngineBuilderHandle::Float(b) => {
                $call(b).map(EngineBuilderHandle::Float)
            }
            EngineBuilderHandle::Char(b) => {
                $call(b).map(EngineBuilderHandle::Char)
            }
            EngineBuilderHandle::Bit(b) => {
                $call(b).map(EngineBuilderHandle::Bit)
            }
            EngineBuilderHandle::Permutation(b) => {
                $call(b).map(EngineBuilderHandle::Permutation)
            }
            EngineBuilderHandle::Any(b) => {
                $call(b).map(EngineBuilderHandle::Any)
            }
            EngineBuilderHandle::Graph(b) => {
                $call(b).map(EngineBuilderHandle::Graph)
            }
            EngineBuilderHandle::Tree(b) => {
                $call(b).map(EngineBuilderHandle::Tree)
            }
            EngineBuilderHandle::Empty => {
                Err(radiate_py_err!("Cannot apply method to Empty builder"))
            }
        }
    }};
}

#[pyclass]
pub struct PyEngineBuilder {
    pub codec: Py<PyAny>,
    pub problem: Py<PyAny>,
    pub inputs: Vec<PyEngineInput>,
}

#[pymethods]
impl PyEngineBuilder {
    #[new]
    pub fn new(codec: Py<PyAny>, problem: Py<PyAny>, inputs: Vec<PyEngineInput>) -> Self {
        PyEngineBuilder {
            codec,
            problem,
            inputs,
        }
    }

    pub fn build<'py>(&mut self, py: Python<'py>) -> PyResult<PyEngine> {
        let mut inner = self.create_builder(py)?;

        let mut accum = HashMap::<PyEngineInputType, Vec<PyEngineInput>>::new();
        let input_groups = self.inputs.iter().fold(&mut accum, |acc, input| {
            acc.entry(input.input_type())
                .or_default()
                .push(input.clone());
            acc
        });

        for (input_type, inputs) in input_groups.iter() {
            inner = Self::process_inputs(inner, *input_type, inputs)?;
        }

        Ok(PyEngine::new(match inner {
            EngineBuilderHandle::Int(builder) => EngineHandle::Int(builder.try_build()?),
            EngineBuilderHandle::Float(builder) => EngineHandle::Float(builder.try_build()?),
            EngineBuilderHandle::Char(builder) => EngineHandle::Char(builder.try_build()?),
            EngineBuilderHandle::Bit(builder) => EngineHandle::Bit(builder.try_build()?),
            EngineBuilderHandle::Any(builder) => EngineHandle::Any(builder.try_build()?),
            EngineBuilderHandle::Permutation(builder) => {
                EngineHandle::Permutation(builder.try_build()?)
            }
            EngineBuilderHandle::Graph(builder) => EngineHandle::Graph(builder.try_build()?),
            EngineBuilderHandle::Tree(builder) => EngineHandle::Tree(builder.try_build()?),
            _ => {
                return Err(radiate_py_err!(
                    "Unsupported builder type for engine creation"
                ));
            }
        }))
    }
}

impl PyEngineBuilder {
    fn create_builder<'py>(&self, py: Python<'py>) -> PyResult<EngineBuilderHandle> {
        let problem = self.problem.bind(py).extract::<PyFitnessFn>()?;
        let codec = self.codec.bind(py);

        let executor = self
            .inputs
            .iter()
            .filter(|i| i.input_type == PyEngineInputType::Executor)
            .filter_map(|input| input.transform())
            .next()
            .unwrap_or(Executor::Serial);

        match problem.inner {
            PyFitnessInner::Custom(fitness_fn) => {
                Self::new_builder_custom(fitness_fn, codec, executor)
            }
            PyFitnessInner::Regression(regression) => {
                Self::new_builder_regression(regression, codec, executor)
            }
            PyFitnessInner::NoveltySearch(novelty_search) => {
                Self::new_builder_novelty(novelty_search, codec, executor)
            }
        }
    }

    fn process_inputs(
        builder: EngineBuilderHandle,
        input_type: PyEngineInputType,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        match input_type {
            PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector => {
                Self::process_selector(builder, inputs)
            }
            PyEngineInputType::Alterer => Self::process_alterers(builder, inputs),
            PyEngineInputType::Objective => Self::process_objective(builder, inputs),
            PyEngineInputType::MaxPhenotypeAge => Self::process_max_phenotype_age(builder, inputs),
            PyEngineInputType::PopulationSize => Self::process_population_size(builder, inputs),
            PyEngineInputType::MaxSpeciesAge => Self::process_max_species_age(builder, inputs),
            PyEngineInputType::SpeciesThreshold => Self::process_species_threshold(builder, inputs),
            PyEngineInputType::OffspringFraction => {
                Self::process_offspring_fraction(builder, inputs)
            }
            PyEngineInputType::FrontRange => Self::process_front_range(builder, inputs),
            PyEngineInputType::Diversity => Self::process_diversity(builder, inputs),
            PyEngineInputType::Population => Self::process_population(builder, inputs),
            PyEngineInputType::Subscriber => Self::process_subscribers(builder, inputs),
            _ => Ok(builder),
        }
    }

    fn process_population(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let population = input.extract::<PyPopulation>("population")?;
                Ok(typed_builder.population(population))
            })
        )
    }

    fn process_subscribers(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_many_typed(|typed_builder, sub_inputs| {
                let mut subs = Vec::with_capacity(sub_inputs.len());
                for input in sub_inputs {
                    subs.push(input.extract::<PySubscriber>("subscriber")?);
                }

                if subs.is_empty() {
                    return Ok(typed_builder);
                }

                let handler = PyEventHandler::new(subs);
                Ok(typed_builder.subscribe(handler))
            })
        )
    }

    fn process_max_species_age(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let max_age = input.get_usize("age").unwrap_or(usize::MAX);
                Ok(typed_builder.max_species_age(max_age))
            })
        )
    }

    fn process_species_threshold(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let threshold = input.get_f32("threshold").unwrap_or(0.3);

                if threshold.is_sign_negative() {
                    return Err(radiate_py_err!("species threshold must be >= 0"));
                }

                Ok(typed_builder.species_threshold(threshold))
            })
        )
    }

    fn process_offspring_fraction(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let fraction = input.get_f32("fraction").unwrap_or(0.8);

                if !(0.0..=1.0).contains(&fraction) {
                    return Err(radiate_py_err!(
                        "offspring fraction must be in the range [0.0, 1.0]"
                    ));
                }

                Ok(typed_builder.offspring_fraction(fraction))
            })
        )
    }

    fn process_population_size(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let size = input.get_usize("size").unwrap_or(100);
                Ok(typed_builder.population_size(size))
            })
        )
    }

    fn process_max_phenotype_age(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let max_age = input.get_usize("age").unwrap_or(usize::MAX);
                Ok(typed_builder.max_age(max_age))
            })
        )
    }

    fn process_selector(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let selector = input.transform();
                Ok(match input.input_type() {
                    PyEngineInputType::SurvivorSelector => {
                        typed_builder.boxed_survivor_selector(selector)
                    }
                    PyEngineInputType::OffspringSelector => {
                        typed_builder.boxed_offspring_selector(selector)
                    }
                    _ => radiate_py_bail!(
                        "parse_selector_typed only implemented for Survivor/Offspring"
                    ),
                })
            })
        )
    }

    fn process_alterers(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_many_typed(|typed_builder, alter_inputs| {
                let alters = alter_inputs.transform();
                Ok(typed_builder.alter(alters))
            })
        )
    }

    fn process_diversity(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let diversity = input.transform();
                Ok(typed_builder.boxed_diversity(diversity))
            })
        )
    }

    fn process_objective(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let objectives = input.get_string("objective").map(|objs| {
                    objs.split('|')
                        .filter_map(|s| match s.trim().to_lowercase().as_str() {
                            "min" => Some(Optimize::Minimize),
                            "max" => Some(Optimize::Maximize),
                            _ => None,
                        })
                        .collect::<Vec<Optimize>>()
                });

                let opt = match objectives {
                    Some(objs) => {
                        if objs.len() == 1 {
                            Objective::Single(objs[0])
                        } else if objs.len() > 1 {
                            Objective::Multi(objs)
                        } else {
                            radiate_py_bail!(
                                "No objectives provided - I'm not even sure this is possible"
                            );
                        }
                    }
                    None => Objective::Single(Optimize::Maximize),
                };

                match opt {
                    Objective::Single(opt) => match opt {
                        Optimize::Minimize => Ok(typed_builder.minimizing()),
                        Optimize::Maximize => Ok(typed_builder.maximizing()),
                    },
                    Objective::Multi(opts) => Ok(typed_builder.multi_objective(opts)),
                }
            })
        )
    }

    fn process_front_range(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let min = input.get_usize("min").unwrap_or(800);
                let max = input.get_usize("max").unwrap_or(1000);

                if min > max {
                    radiate_py_bail!(format!(
                        "Front sizes (min, max) values are invalid: got ({}, {})",
                        min, max
                    ));
                }

                if min == 0 || max == 0 {
                    radiate_py_bail!(format!(
                        "Front sizes (min, max) values must be greater than zero: got ({}, {})",
                        min, max
                    ));
                }

                Ok(typed_builder.front_size(min..max))
            })
        )
    }

    fn new_builder_custom<'py>(
        fitness_fn: PyAnyObject,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        let fitness_fn = fitness_fn.inner;

        let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
            EngineBuilderHandle::Float(Self::new_builder(fitness_fn, float_codec.codec, executor))
        } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
            EngineBuilderHandle::Int(Self::new_builder(fitness_fn, int_codec.codec, executor))
        } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
            EngineBuilderHandle::Char(Self::new_builder(fitness_fn, char_codec.codec, executor))
        } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
            EngineBuilderHandle::Bit(Self::new_builder(fitness_fn, bit_codec.codec, executor))
        } else if let Ok(any_codec) = codec.extract::<PyAnyCodec>() {
            EngineBuilderHandle::Any(Self::new_builder(fitness_fn, any_codec.codec, executor))
        } else if let Ok(perm_codec) = codec.extract::<PyPermutationCodec>() {
            EngineBuilderHandle::Permutation(Self::new_builder(
                fitness_fn,
                perm_codec.codec,
                executor,
            ))
        } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            let py_codec = Self::wrapped_codec(graph_codec.codec);
            EngineBuilderHandle::Graph(
                Self::new_builder(fitness_fn, py_codec, executor)
                    .replace_strategy(GraphReplacement),
            )
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            let py_codec = Self::wrapped_codec(tree_codec.codec);
            EngineBuilderHandle::Tree(Self::new_builder(fitness_fn, py_codec, executor))
        } else {
            radiate_py_bail!("Unsupported codec type for custom fitness function");
        };

        Ok(builder)
    }

    fn new_builder_regression<'py>(
        regression: Regression,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        let builder = if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            EngineBuilderHandle::Graph(
                GeneticEngine::builder()
                    .codec(graph_codec.codec)
                    .fitness_fn(regression)
                    .executor(executor)
                    .bus_executor(Executor::default())
                    .replace_strategy(GraphReplacement),
            )
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            EngineBuilderHandle::Tree(
                GeneticEngine::builder()
                    .codec(tree_codec.codec)
                    .fitness_fn(regression)
                    .executor(executor)
                    .bus_executor(Executor::default()),
            )
        } else {
            radiate_py_bail!("Unsupported codec type for regression problem");
        };

        Ok(builder)
    }

    fn new_builder_novelty<'py>(
        fitness: PyAnyObject,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        let fitness = fitness.inner;
        let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
            EngineBuilderHandle::Float(Self::new_builder(fitness, float_codec.codec, executor))
        } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
            EngineBuilderHandle::Int(Self::new_builder(fitness, int_codec.codec, executor))
        } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
            EngineBuilderHandle::Char(Self::new_builder(fitness, char_codec.codec, executor))
        } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
            EngineBuilderHandle::Bit(Self::new_builder(fitness, bit_codec.codec, executor))
        } else if let Ok(any_codec) = codec.extract::<PyAnyCodec>() {
            EngineBuilderHandle::Any(Self::new_builder(fitness, any_codec.codec, executor))
        } else if let Ok(permutation_codec) = codec.extract::<PyPermutationCodec>() {
            EngineBuilderHandle::Permutation(Self::new_builder(
                fitness,
                permutation_codec.codec,
                executor,
            ))
        } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            let py_codec = Self::wrapped_codec(graph_codec.codec);
            EngineBuilderHandle::Graph(
                Self::new_builder(fitness, py_codec, executor).replace_strategy(GraphReplacement),
            )
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            let py_codec = Self::wrapped_codec(tree_codec.codec);
            EngineBuilderHandle::Tree(Self::new_builder(fitness, py_codec, executor))
        } else {
            radiate_py_bail!("Unsupported codec type for novelty search problem");
        };

        Ok(builder)
    }

    fn new_builder<C, T>(
        fitness_fn: Py<PyAny>,
        codec: PyCodec<C, T>,
        executor: Executor,
    ) -> GeneticEngineBuilder<C, T>
    where
        C: Chromosome + PartialEq + Clone + 'static,
        T: Send + Sync + Clone + IntoPyAnyObject + 'static,
    {
        let problem = PyProblem::new(fitness_fn, codec);
        GeneticEngine::builder()
            .problem(problem)
            .executor(executor.clone())
            .evaluator(FreeThreadPyEvaluator::new(executor))
            .bus_executor(Executor::default())
    }

    fn wrapped_codec<C, T, PC>(original: PC) -> PyCodec<C, T>
    where
        PC: Codec<C, T> + Clone + 'static,
        C: Chromosome + 'static,
        T: Send + Sync + Clone + IntoPyAnyObject + 'static,
    {
        let cloned = original.clone();
        PyCodec::new()
            .with_encoder(move || cloned.encode())
            .with_decoder(move |_, genotype| original.decode(genotype))
    }

    fn process_single_typed<C, T, F>(
        processor: F,
    ) -> impl Fn(GeneticEngineBuilder<C, T>, &[PyEngineInput]) -> PyResult<GeneticEngineBuilder<C, T>>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + 'static,
        F: Fn(GeneticEngineBuilder<C, T>, &PyEngineInput) -> PyResult<GeneticEngineBuilder<C, T>>,
    {
        move |builder: GeneticEngineBuilder<C, T>, inputs: &[PyEngineInput]| match inputs {
            [] => Ok(builder),
            [only] => processor(builder, only),
            many => processor(builder, &many[many.len() - 1]),
        }
    }

    fn process_many_typed<C, T, F>(
        processor: F,
    ) -> impl Fn(GeneticEngineBuilder<C, T>, &[PyEngineInput]) -> PyResult<GeneticEngineBuilder<C, T>>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + 'static,
        F: Fn(GeneticEngineBuilder<C, T>, &[PyEngineInput]) -> PyResult<GeneticEngineBuilder<C, T>>,
    {
        move |builder: GeneticEngineBuilder<C, T>,
              inputs: &[PyEngineInput]|
              -> PyResult<GeneticEngineBuilder<C, T>> { processor(builder, inputs) }
    }
}

// use crate::bindings::codec::PyTreeCodec;
// use crate::bindings::{EngineBuilderHandle, EngineHandle};
// use crate::events::PyEventHandler;
// use crate::{AnyChromosome, PySubscriber};
// use crate::{
//     FreeThreadPyEvaluator, InputTransform, PyCodec, PyEngine, PyEngineInput, PyEngineInputType,
//     PyFitnessFn, PyFitnessInner, PyPermutationCodec, PyPopulation, prelude::*,
// };
// use pyo3::exceptions::PyTypeError;
// use pyo3::{Py, PyAny, pyclass, pymethods, types::PyAnyMethods};
// use radiate::{RadiateResult, prelude::*};
// use radiate_error::{radiate_bail, radiate_py_bail, radiate_py_err};
// use std::collections::HashMap;

// macro_rules! apply_to_builder {
//     ($builder:expr, $method:ident($($args:expr),*)) => {
//         match $builder {
//             EngineBuilderHandle::Int(b) => Ok(EngineBuilderHandle::Int(b.$method($($args),*))),
//             EngineBuilderHandle::Float(b) => Ok(EngineBuilderHandle::Float(b.$method($($args),*))),
//             EngineBuilderHandle::Char(b) => Ok(EngineBuilderHandle::Char(b.$method($($args),*))),
//             EngineBuilderHandle::Bit(b) => Ok(EngineBuilderHandle::Bit(b.$method($($args),*))),
//             EngineBuilderHandle::Permutation(b) => Ok(EngineBuilderHandle::Permutation(b.$method($($args),*))),
//             EngineBuilderHandle::Graph(b) => Ok(EngineBuilderHandle::Graph(b.$method($($args),*))),
//             EngineBuilderHandle::Tree(b) => Ok(EngineBuilderHandle::Tree(b.$method($($args),*))),
//             EngineBuilderHandle::Any(b) => Ok(EngineBuilderHandle::Any(b.$method($($args),*))),
//             EngineBuilderHandle::Empty => Err(radiate_py_err!(
//                 "Cannot apply method to Empty builder"
//             )),
//         }
//     };
// }

// macro_rules! dispatch_builder_typed {
//     ($builder:expr, $input:expr, $applier:path) => {{
//         match $builder {
//             EngineBuilderHandle::Int(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Int(nb)
//             }
//             EngineBuilderHandle::Float(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Float(nb)
//             }
//             EngineBuilderHandle::Char(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Char(nb)
//             }
//             EngineBuilderHandle::Bit(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Bit(nb)
//             }
//             EngineBuilderHandle::Permutation(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Permutation(nb)
//             }
//             EngineBuilderHandle::Any(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Any(nb)
//             }
//             EngineBuilderHandle::Graph(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Graph(nb)
//             }
//             EngineBuilderHandle::Tree(b) => {
//                 let nb = $applier($input, b)?;
//                 EngineBuilderHandle::Tree(nb)
//             }
//             EngineBuilderHandle::Empty => {
//                 return Err(radiate_py_err!("Cannot apply method to Empty builder"));
//             }
//         }
//     }};
// }

// #[pyclass]
// pub struct PyEngineBuilder {
//     pub codec: Py<PyAny>,
//     pub problem: Py<PyAny>,
//     pub inputs: Vec<PyEngineInput>,
// }

// #[pymethods]
// impl PyEngineBuilder {
//     #[new]
//     pub fn new(codec: Py<PyAny>, problem: Py<PyAny>, inputs: Vec<PyEngineInput>) -> Self {
//         PyEngineBuilder {
//             codec,
//             problem,
//             inputs,
//         }
//     }

//     pub fn build<'py>(&mut self, py: Python<'py>) -> PyResult<PyEngine> {
//         let mut inner = self.create_builder(py)?;

//         let mut accum = HashMap::<PyEngineInputType, Vec<PyEngineInput>>::new();
//         let input_groups = self.inputs.iter().fold(&mut accum, |acc, input| {
//             acc.entry(input.input_type())
//                 .or_default()
//                 .push(input.clone());
//             acc
//         });

//         for (input_type, inputs) in input_groups.iter() {
//             inner = Self::process_inputs(inner, *input_type, inputs)?;
//         }

//         Ok(PyEngine::new(match inner {
//             EngineBuilderHandle::Int(builder) => EngineHandle::Int(builder.try_build()?),
//             EngineBuilderHandle::Float(builder) => EngineHandle::Float(builder.try_build()?),
//             EngineBuilderHandle::Char(builder) => EngineHandle::Char(builder.try_build()?),
//             EngineBuilderHandle::Bit(builder) => EngineHandle::Bit(builder.try_build()?),
//             EngineBuilderHandle::Any(builder) => EngineHandle::Any(builder.try_build()?),
//             EngineBuilderHandle::Permutation(builder) => {
//                 EngineHandle::Permutation(builder.try_build()?)
//             }
//             EngineBuilderHandle::Graph(builder) => EngineHandle::Graph(builder.try_build()?),
//             EngineBuilderHandle::Tree(builder) => EngineHandle::Tree(builder.try_build()?),
//             _ => {
//                 return Err(radiate_py_err!(
//                     "Unsupported builder type for engine creation"
//                 ));
//             }
//         }))
//     }
// }

// impl PyEngineBuilder {
//     fn create_builder<'py>(&self, py: Python<'py>) -> PyResult<EngineBuilderHandle> {
//         let problem = self.problem.bind(py).extract::<PyFitnessFn>()?;
//         let codec = self.codec.bind(py);

//         let executor = self
//             .inputs
//             .iter()
//             .filter(|i| i.input_type == PyEngineInputType::Executor)
//             .filter_map(|input| input.transform())
//             .next()
//             .unwrap_or(Executor::Serial);

//         match problem.inner {
//             PyFitnessInner::Custom(fitness_fn) => {
//                 Self::new_builder_custom(fitness_fn, codec, executor)
//             }
//             PyFitnessInner::Regression(regression) => {
//                 Self::new_builder_regression(regression, codec, executor)
//             }
//             PyFitnessInner::NoveltySearch(novelty_search) => {
//                 Self::new_builder_novelty(novelty_search, codec, executor)
//             }
//         }
//     }

//     fn process_inputs(
//         builder: EngineBuilderHandle,
//         input_type: PyEngineInputType,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         match input_type {
//             PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector => {
//                 Self::process_selector(builder, inputs)
//             }
//             PyEngineInputType::Alterer => Self::process_alterers(builder, inputs),
//             PyEngineInputType::Objective => Self::process_objective(builder, inputs),
//             PyEngineInputType::MaxPhenotypeAge => Self::process_max_phenotype_age(builder, inputs),
//             PyEngineInputType::PopulationSize => Self::process_population_size(builder, inputs),
//             PyEngineInputType::MaxSpeciesAge => Self::process_max_species_age(builder, inputs),
//             PyEngineInputType::SpeciesThreshold => Self::process_species_threshold(builder, inputs),
//             PyEngineInputType::OffspringFraction => {
//                 Self::process_offspring_fraction(builder, inputs)
//             }
//             PyEngineInputType::FrontRange => Self::process_front_range(builder, inputs),
//             PyEngineInputType::Diversity => Self::process_diversity(builder, inputs),
//             PyEngineInputType::Population => Self::process_population(builder, inputs),
//             PyEngineInputType::Subscriber => Self::process_subscribers(builder, inputs),
//             _ => Ok(builder),
//         }
//     }

//     fn process_single_value<F>(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//         processor: F,
//     ) -> PyResult<EngineBuilderHandle>
//     where
//         F: Fn(EngineBuilderHandle, &PyEngineInput) -> PyResult<EngineBuilderHandle>,
//     {
//         match inputs {
//             [] => Ok(builder),
//             [only] => processor(builder, only),
//             many => processor(builder, &many[many.len() - 1]),
//         }
//     }

//     fn process_population(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let population = input.extract::<PyPopulation>("population")?;
//             apply_to_builder!(builder, population(population))
//         })
//     }

//     fn process_subscribers(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         let mut subs = Vec::with_capacity(inputs.len());
//         for input in inputs {
//             subs.push(input.extract::<PySubscriber>("subscriber")?);
//         }

//         if subs.is_empty() {
//             return Ok(builder);
//         }

//         let handler = PyEventHandler::new(subs);
//         apply_to_builder!(builder, subscribe(handler))
//     }

//     fn process_max_species_age(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let max_age = input.get_usize("age").unwrap_or(usize::MAX);
//             apply_to_builder!(builder, max_species_age(max_age))
//         })
//     }

//     fn process_species_threshold(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let threshold = input.get_f32("threshold").unwrap_or(0.3);

//             if threshold.is_sign_negative() {
//                 return Err(radiate_py_err!("species threshold must be >= 0"));
//             }

//             apply_to_builder!(builder, species_threshold(threshold))
//         })
//     }

//     fn process_offspring_fraction(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let fraction = input.get_f32("fraction").unwrap_or(0.8);

//             if !(0.0..=1.0).contains(&fraction) {
//                 return Err(radiate_py_err!(
//                     "offspring fraction must be in the range [0.0, 1.0]"
//                 ));
//             }

//             apply_to_builder!(builder, offspring_fraction(fraction))
//         })
//     }

//     fn process_population_size(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let size = input.get_usize("size").unwrap_or(5);
//             apply_to_builder!(builder, population_size(size))
//         })
//     }

//     fn process_max_phenotype_age(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let max_age = input.get_usize("age").unwrap_or(usize::MAX);
//             apply_to_builder!(builder, max_age(max_age))
//         })
//     }

//     fn parse_selector_typed<C, T>(
//         input: &PyEngineInput,
//         builder: GeneticEngineBuilder<C, T>,
//     ) -> RadiateResult<GeneticEngineBuilder<C, T>>
//     where
//         C: Chromosome + Clone + PartialEq,
//         T: Clone + Send,
//     {
//         let selector: Box<dyn Select<C>> = input.transform();
//         Ok(match input.input_type() {
//             PyEngineInputType::SurvivorSelector => builder.boxed_survivor_selector(selector),
//             PyEngineInputType::OffspringSelector => builder.boxed_offspring_selector(selector),
//             _ => radiate_bail!("parse_selector_typed only implemented for Survivor/Offspring"),
//         })
//     }

//     fn process_selector(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let temp = dispatch_builder_typed!(builder, input, Self::parse_selector_typed);
//             Ok(temp)
//         })

//         // Self::process_single_value(builder, inputs, |builder, input| match builder {
//         //     EngineBuilderHandle::Int(builder) => Ok(EngineBuilderHandle::Int(
//         //         Self::parse_selector_typed(input, builder)?,
//         //     )),
//         //     EngineBuilderHandle::Float(builder) => {
//         //         let selector: Box<dyn Select<FloatChromosome>> = input.transform();
//         //         match input.input_type() {
//         //             PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Float(
//         //                 builder.boxed_survivor_selector(selector),
//         //             )),
//         //             PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Float(
//         //                 builder.boxed_offspring_selector(selector),
//         //             )),
//         //             _ => Err(PyTypeError::new_err(
//         //                 "process_selector only implemented for Survivor and Offspring selectors",
//         //             )),
//         //         }
//         //     }
//         //     EngineBuilderHandle::Char(builder) => {
//         //         let selector: Box<dyn Select<CharChromosome>> = input.transform();
//         //         match input.input_type() {
//         //             PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Char(
//         //                 builder.boxed_survivor_selector(selector),
//         //             )),
//         //             PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Char(
//         //                 builder.boxed_offspring_selector(selector),
//         //             )),
//         //             _ => Err(PyTypeError::new_err(
//         //                 "process_selector only implemented for Survivor and Offspring selectors",
//         //             )),
//         //         }
//         //     }
//         //     EngineBuilderHandle::Bit(builder) => {
//         //         let selector: Box<dyn Select<BitChromosome>> = input.transform();
//         //         match input.input_type() {
//         //             PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Bit(
//         //                 builder.boxed_survivor_selector(selector),
//         //             )),
//         //             PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Bit(
//         //                 builder.boxed_offspring_selector(selector),
//         //             )),
//         //             _ => Err(PyTypeError::new_err(
//         //                 "process_selector only implemented for Survivor and Offspring selectors",
//         //             )),
//         //         }
//         //     }
//         //     EngineBuilderHandle::Permutation(builder) => {
//         //         let selector: Box<dyn Select<PermutationChromosome<usize>>> = input.transform();
//         //         match input.input_type() {
//         //             PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Permutation(
//         //                 builder.boxed_survivor_selector(selector),
//         //             )),
//         //             PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Permutation(
//         //                 builder.boxed_offspring_selector(selector),
//         //             )),
//         //             _ => Err(PyTypeError::new_err(
//         //                 "process_selector only implemented for Survivor and Offspring selectors",
//         //             )),
//         //         }
//         //     }
//         //     EngineBuilderHandle::Any(builder) => {
//         //         let selector: Box<dyn Select<AnyChromosome<'static>>> = input.transform();
//         //         match input.input_type() {
//         //             PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Any(
//         //                 builder.boxed_survivor_selector(selector),
//         //             )),
//         //             PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Any(
//         //                 builder.boxed_offspring_selector(selector),
//         //             )),
//         //             _ => Err(PyTypeError::new_err(
//         //                 "process_selector only implemented for Survivor and Offspring selectors",
//         //             )),
//         //         }
//         //     }
//         //     EngineBuilderHandle::Graph(builder) => {
//         //         let selector: Box<dyn Select<GraphChromosome<Op<f32>>>> = input.transform();
//         //         match input.input_type() {
//         //             PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Graph(
//         //                 builder.boxed_survivor_selector(selector),
//         //             )),
//         //             PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Graph(
//         //                 builder.boxed_offspring_selector(selector),
//         //             )),
//         //             _ => Err(PyTypeError::new_err(
//         //                 "process_selector only implemented for Survivor and Offspring selectors",
//         //             )),
//         //         }
//         //     }
//         //     EngineBuilderHandle::Tree(builder) => {
//         //         let selector: Box<dyn Select<TreeChromosome<Op<f32>>>> = input.transform();
//         //         match input.input_type() {
//         //             PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Tree(
//         //                 builder.boxed_survivor_selector(selector),
//         //             )),
//         //             PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Tree(
//         //                 builder.boxed_offspring_selector(selector),
//         //             )),
//         //             _ => Err(PyTypeError::new_err(
//         //                 "process_selector only implemented for Survivor and Offspring selectors",
//         //             )),
//         //         }
//         //     }
//         //     _ => Err(radiate_py_err!(
//         //         "process_selector only implemented for Int, Float, Char, Bit, Any, Graph, and Tree builder types"
//         //     )),
//         // })
//     }

//     fn process_alterers(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         match builder {
//             EngineBuilderHandle::Int(builder) => {
//                 let alters: Vec<Box<dyn Alter<IntChromosome<i32>>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Int(builder.alter(alters)))
//             }
//             EngineBuilderHandle::Float(builder) => {
//                 let alters: Vec<Box<dyn Alter<FloatChromosome>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Float(builder.alter(alters)))
//             }
//             EngineBuilderHandle::Char(builder) => {
//                 let alters: Vec<Box<dyn Alter<CharChromosome>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Char(builder.alter(alters)))
//             }
//             EngineBuilderHandle::Bit(builder) => {
//                 let alters: Vec<Box<dyn Alter<BitChromosome>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Bit(builder.alter(alters)))
//             }
//             EngineBuilderHandle::Any(builder) => {
//                 let alters: Vec<Box<dyn Alter<AnyChromosome<'static>>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Any(builder.alter(alters)))
//             }
//             EngineBuilderHandle::Graph(builder) => {
//                 let alters: Vec<Box<dyn Alter<GraphChromosome<Op<f32>>>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Graph(builder.alter(alters)))
//             }
//             EngineBuilderHandle::Tree(builder) => {
//                 let alters: Vec<Box<dyn Alter<TreeChromosome<Op<f32>>>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Tree(builder.alter(alters)))
//             }
//             EngineBuilderHandle::Permutation(builder) => {
//                 let alters: Vec<Box<dyn Alter<PermutationChromosome<usize>>>> = inputs.transform();
//                 Ok(EngineBuilderHandle::Permutation(builder.alter(alters)))
//             }
//             _ => Err(radiate_py_err!(
//                 "process_alterer only implemented for Int, Float, Char, Bit, Any, Graph, Tree, and Permutation builder types"
//             )),
//         }
//     }

//     fn process_diversity(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| match builder {
//             EngineBuilderHandle::Int(b) => {
//                 let diversity: Option<Box<dyn Diversity<IntChromosome<i32>>>> = input.transform();
//                 Ok(EngineBuilderHandle::Int(b.boxed_diversity(diversity)))
//             }
//             EngineBuilderHandle::Float(b) => {
//                 let diversity: Option<Box<dyn Diversity<FloatChromosome>>> = input.transform();
//                 Ok(EngineBuilderHandle::Float(b.boxed_diversity(diversity)))
//             }
//             EngineBuilderHandle::Char(b) => {
//                 let diversity: Option<Box<dyn Diversity<CharChromosome>>> = input.transform();
//                 Ok(EngineBuilderHandle::Char(b.boxed_diversity(diversity)))
//             }
//             EngineBuilderHandle::Bit(b) => {
//                 let diversity: Option<Box<dyn Diversity<BitChromosome>>> = input.transform();
//                 Ok(EngineBuilderHandle::Bit(b.boxed_diversity(diversity)))
//             }
//             EngineBuilderHandle::Graph(b) => {
//                 let diversity: Option<Box<dyn Diversity<GraphChromosome<Op<f32>>>>> =
//                     input.transform();
//                 Ok(EngineBuilderHandle::Graph(b.boxed_diversity(diversity)))
//             }
//             EngineBuilderHandle::Permutation(b) => {
//                 let diversity: Option<Box<dyn Diversity<PermutationChromosome<usize>>>> =
//                     input.transform();
//                 Ok(EngineBuilderHandle::Permutation(
//                     b.boxed_diversity(diversity),
//                 ))
//             }
//             _ => Err(radiate_py_err!(
//                 "Unable to process diversity for this builder type"
//             )),
//         })
//     }

//     fn process_objective(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let objectives = input.get_string("objective").map(|objs| {
//                 objs.split('|')
//                     .filter_map(|s| match s.trim().to_lowercase().as_str() {
//                         "min" => Some(Optimize::Minimize),
//                         "max" => Some(Optimize::Maximize),
//                         _ => None,
//                     })
//                     .collect::<Vec<Optimize>>()
//             });

//             let opt = match objectives {
//                 Some(objs) => {
//                     if objs.len() == 1 {
//                         Objective::Single(objs[0])
//                     } else if objs.len() > 1 {
//                         Objective::Multi(objs)
//                     } else {
//                         radiate_py_bail!(
//                             "No objectives provided - I'm not even sure this is possible"
//                         );
//                     }
//                 }
//                 None => Objective::Single(Optimize::Maximize),
//             };

//             match opt {
//                 Objective::Single(opt) => match opt {
//                     Optimize::Minimize => apply_to_builder!(builder, minimizing()),
//                     Optimize::Maximize => apply_to_builder!(builder, maximizing()),
//                 },
//                 Objective::Multi(opts) => apply_to_builder!(builder, multi_objective(opts)),
//             }
//         })
//     }

//     fn process_front_range(
//         builder: EngineBuilderHandle,
//         inputs: &[PyEngineInput],
//     ) -> PyResult<EngineBuilderHandle> {
//         Self::process_single_value(builder, inputs, |builder, input| {
//             let min = input.get_usize("min").unwrap_or(800);
//             let max = input.get_usize("max").unwrap_or(1000);

//             if min > max {
//                 radiate_py_bail!(format!(
//                     "Front sizes (min, max) values are invalid: got ({}, {})",
//                     min, max
//                 ));
//             }

//             if min == 0 || max == 0 {
//                 radiate_py_bail!(format!(
//                     "Front sizes (min, max) values must be greater than zero: got ({}, {})",
//                     min, max
//                 ));
//             }

//             apply_to_builder!(builder, front_size(min..max))
//         })
//     }

//     fn new_builder_custom<'py>(
//         fitness_fn: PyAnyObject,
//         codec: &Bound<'py, PyAny>,
//         executor: Executor,
//     ) -> PyResult<EngineBuilderHandle> {
//         let fitness_fn = fitness_fn.inner;

//         let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
//             EngineBuilderHandle::Float(Self::new_builder(fitness_fn, float_codec.codec, executor))
//         } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
//             EngineBuilderHandle::Int(Self::new_builder(fitness_fn, int_codec.codec, executor))
//         } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
//             EngineBuilderHandle::Char(Self::new_builder(fitness_fn, char_codec.codec, executor))
//         } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
//             EngineBuilderHandle::Bit(Self::new_builder(fitness_fn, bit_codec.codec, executor))
//         } else if let Ok(any_codec) = codec.extract::<PyAnyCodec>() {
//             EngineBuilderHandle::Any(Self::new_builder(fitness_fn, any_codec.codec, executor))
//         } else if let Ok(perm_codec) = codec.extract::<PyPermutationCodec>() {
//             EngineBuilderHandle::Permutation(Self::new_builder(
//                 fitness_fn,
//                 perm_codec.codec,
//                 executor,
//             ))
//         } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
//             let py_codec = Self::wrapped_codec(graph_codec.codec);
//             EngineBuilderHandle::Graph(
//                 Self::new_builder(fitness_fn, py_codec, executor)
//                     .replace_strategy(GraphReplacement),
//             )
//         } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
//             let py_codec = Self::wrapped_codec(tree_codec.codec);
//             EngineBuilderHandle::Tree(Self::new_builder(fitness_fn, py_codec, executor))
//         } else {
//             radiate_py_bail!("Unsupported codec type for custom fitness function");
//         };

//         Ok(builder)
//     }

//     fn new_builder_regression<'py>(
//         regression: Regression,
//         codec: &Bound<'py, PyAny>,
//         executor: Executor,
//     ) -> PyResult<EngineBuilderHandle> {
//         let builder = if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
//             EngineBuilderHandle::Graph(
//                 GeneticEngine::builder()
//                     .codec(graph_codec.codec)
//                     .fitness_fn(regression)
//                     .executor(executor)
//                     .bus_executor(Executor::default())
//                     .replace_strategy(GraphReplacement),
//             )
//         } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
//             EngineBuilderHandle::Tree(
//                 GeneticEngine::builder()
//                     .codec(tree_codec.codec)
//                     .fitness_fn(regression)
//                     .executor(executor)
//                     .bus_executor(Executor::default()),
//             )
//         } else {
//             radiate_py_bail!("Unsupported codec type for regression problem");
//         };

//         Ok(builder)
//     }

//     fn new_builder_novelty<'py>(
//         fitness: PyAnyObject,
//         codec: &Bound<'py, PyAny>,
//         executor: Executor,
//     ) -> PyResult<EngineBuilderHandle> {
//         let fitness = fitness.inner;
//         let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
//             EngineBuilderHandle::Float(Self::new_builder(fitness, float_codec.codec, executor))
//         } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
//             EngineBuilderHandle::Int(Self::new_builder(fitness, int_codec.codec, executor))
//         } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
//             EngineBuilderHandle::Char(Self::new_builder(fitness, char_codec.codec, executor))
//         } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
//             EngineBuilderHandle::Bit(Self::new_builder(fitness, bit_codec.codec, executor))
//         } else if let Ok(any_codec) = codec.extract::<PyAnyCodec>() {
//             EngineBuilderHandle::Any(Self::new_builder(fitness, any_codec.codec, executor))
//         } else if let Ok(permutation_codec) = codec.extract::<PyPermutationCodec>() {
//             EngineBuilderHandle::Permutation(Self::new_builder(
//                 fitness,
//                 permutation_codec.codec,
//                 executor,
//             ))
//         } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
//             let py_codec = Self::wrapped_codec(graph_codec.codec);
//             EngineBuilderHandle::Graph(
//                 Self::new_builder(fitness, py_codec, executor).replace_strategy(GraphReplacement),
//             )
//         } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
//             let py_codec = Self::wrapped_codec(tree_codec.codec);
//             EngineBuilderHandle::Tree(Self::new_builder(fitness, py_codec, executor))
//         } else {
//             radiate_py_bail!("Unsupported codec type for novelty search problem");
//         };

//         Ok(builder)
//     }

//     fn new_builder<C, T>(
//         fitness_fn: Py<PyAny>,
//         codec: PyCodec<C, T>,
//         executor: Executor,
//     ) -> GeneticEngineBuilder<C, T>
//     where
//         C: Chromosome + PartialEq + Clone + 'static,
//         T: Send + Sync + Clone + IntoPyAnyObject + 'static,
//     {
//         let problem = PyProblem::new(fitness_fn, codec);
//         GeneticEngine::builder()
//             .problem(problem)
//             .executor(executor.clone())
//             .evaluator(FreeThreadPyEvaluator::new(executor))
//             .bus_executor(Executor::default())
//     }

//     fn wrapped_codec<C, T, PC>(original: PC) -> PyCodec<C, T>
//     where
//         PC: Codec<C, T> + Clone + 'static,
//         C: Chromosome + 'static,
//         T: Send + Sync + Clone + IntoPyAnyObject + 'static,
//     {
//         let cloned = original.clone();
//         PyCodec::new()
//             .with_encoder(move || cloned.encode())
//             .with_decoder(move |_, genotype| original.decode(genotype))
//     }
// }
