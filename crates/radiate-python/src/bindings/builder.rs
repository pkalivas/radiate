use crate::PySubscriber;
use crate::bindings::codec::PyTreeCodec;
use crate::bindings::{EngineBuilderHandle, EngineHandle};
use crate::events::PyEventHandler;
use crate::{
    FreeThreadPyEvaluator, InputTransform, PyCodec, PyEngine, PyEngineInput, PyEngineInputType,
    PyFitnessFn, PyFitnessInner, PyPermutationCodec, PyPopulation, prelude::*,
};
use pyo3::{Py, PyAny, pyclass, pymethods, types::PyAnyMethods};
use radiate::prelude::*;
use radiate_error::{radiate_py_bail, radiate_py_err};
use std::collections::HashMap;

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
        use EngineBuilderHandle::*;
        match $builder {
            Int(b) => $call(b).map(Int),
            Float(b) => $call(b).map(Float),
            Char(b) => $call(b).map(Char),
            Bit(b) => $call(b).map(Bit),
            Permutation(b) => $call(b).map(Permutation),
            Any(b) => $call(b).map(Any),
            Graph(b) => $call(b).map(Graph),
            Tree(b) => $call(b).map(Tree),
            Empty => Err(radiate_py_err!("Cannot apply method to Empty builder"))
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
        use EngineBuilderHandle::*;
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
            Int(builder) => EngineHandle::Int(builder.try_build()?),
            Float(builder) => EngineHandle::Float(builder.try_build()?),
            Char(builder) => EngineHandle::Char(builder.try_build()?),
            Bit(builder) => EngineHandle::Bit(builder.try_build()?),
            Any(builder) => EngineHandle::Any(builder.try_build()?),
            Permutation(builder) => EngineHandle::Permutation(builder.try_build()?),
            Graph(builder) => EngineHandle::Graph(builder.try_build()?),
            Tree(builder) => EngineHandle::Tree(builder.try_build()?),
            _ => {
                radiate_py_bail!("Unsupported builder type for engine creation");
            }
        }))
    }
}

impl PyEngineBuilder {
    fn create_builder<'py>(&self, py: Python<'py>) -> PyResult<EngineBuilderHandle> {
        use PyFitnessInner::*;

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
            custom @ Custom(..) => Self::init_custom_handle(codec, custom, executor),
            reg @ Regression(..) => Self::init_regression_builder(reg, codec, executor),
            novelty @ NoveltySearch(..) => Self::init_novelty_builder(novelty, codec, executor),
        }
    }

    fn process_inputs(
        builder: EngineBuilderHandle,
        input_type: PyEngineInputType,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        use PyEngineInputType::*;
        match input_type {
            SurvivorSelector | OffspringSelector => Self::process_selector(builder, inputs),
            Alterer => Self::process_alterers(builder, inputs),
            Objective => Self::process_objective(builder, inputs),
            MaxPhenotypeAge => Self::process_max_phenotype_age(builder, inputs),
            PopulationSize => Self::process_population_size(builder, inputs),
            MaxSpeciesAge => Self::process_max_species_age(builder, inputs),
            SpeciesThreshold => Self::process_species_threshold(builder, inputs),
            OffspringFraction => Self::process_offspring_fraction(builder, inputs),
            FrontRange => Self::process_front_range(builder, inputs),
            Diversity => Self::process_diversity(builder, inputs),
            Population => Self::process_population(builder, inputs),
            Subscriber => Self::process_subscribers(builder, inputs),
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
        use PyEngineInputType::*;
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let selector = input.transform();
                Ok(match input.input_type() {
                    SurvivorSelector => typed_builder.boxed_survivor_selector(selector),
                    OffspringSelector => typed_builder.boxed_offspring_selector(selector),
                    _ => {
                        radiate_py_bail!("process_selector only implemented for Survivor/Offspring")
                    }
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

    fn init_custom_handle<'py>(
        codec: &Bound<'py, PyAny>,
        fitness: PyFitnessInner,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        use EngineBuilderHandle::*;
        use PyFitnessInner::Custom;

        if !matches!(fitness, Custom(_, _)) {
            error::radiate_py_bail!("init_custom_handle only supports Custom fitness functions")
        }

        let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
            Float(Self::new_builder(fitness, float_codec.codec, executor))
        } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
            Int(Self::new_builder(fitness, int_codec.codec, executor))
        } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
            Char(Self::new_builder(fitness, char_codec.codec, executor))
        } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
            Bit(Self::new_builder(fitness, bit_codec.codec, executor))
        } else if let Ok(any_codec) = codec.extract::<PyAnyCodec>() {
            Any(Self::new_builder(fitness, any_codec.codec, executor))
        } else if let Ok(perm_codec) = codec.extract::<PyPermutationCodec>() {
            Permutation(Self::new_builder(fitness, perm_codec.codec, executor))
        } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            let py_codec = Self::wrapped_codec(graph_codec.codec);
            Graph(Self::new_builder(fitness, py_codec, executor).replace_strategy(GraphReplacement))
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            let py_codec = Self::wrapped_codec(tree_codec.codec);
            Tree(Self::new_builder(fitness, py_codec, executor))
        } else {
            radiate_py_bail!("Unsupported codec type for custom fitness function");
        };

        Ok(builder)
    }

    fn init_regression_builder<'py>(
        regression: PyFitnessInner,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        use EngineBuilderHandle::*;
        use PyFitnessInner::Regression;

        let Regression(regression, is_batch) = regression else {
            error::radiate_py_bail!(
                "init_regression_builder only supports Regression fitness functions"
            )
        };

        let builder = if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            let base_engine = GeneticEngine::builder()
                .codec(graph_codec.codec)
                .executor(executor)
                .bus_executor(Executor::default())
                .replace_strategy(GraphReplacement);

            if is_batch {
                Graph(base_engine.batch_fitness_fn(regression))
            } else {
                Graph(base_engine.fitness_fn(regression))
            }
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            let base_engine = GeneticEngine::builder()
                .codec(tree_codec.codec)
                .executor(executor)
                .bus_executor(Executor::default());

            if is_batch {
                Tree(base_engine.batch_fitness_fn(regression))
            } else {
                Tree(base_engine.fitness_fn(regression))
            }
        } else {
            radiate_py_bail!("Unsupported codec type for regression problem");
        };

        Ok(builder)
    }

    fn init_novelty_builder<'py>(
        fitness: PyFitnessInner,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        use EngineBuilderHandle::*;
        use PyFitnessInner::NoveltySearch;

        if !matches!(fitness, NoveltySearch(_, _)) {
            error::radiate_py_bail!(
                "init_novelty_builder only supports Novelty Search fitness functions"
            )
        }

        let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
            Float(Self::new_builder(fitness, float_codec.codec, executor))
        } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
            Int(Self::new_builder(fitness, int_codec.codec, executor))
        } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
            Char(Self::new_builder(fitness, char_codec.codec, executor))
        } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
            Bit(Self::new_builder(fitness, bit_codec.codec, executor))
        } else if let Ok(any_codec) = codec.extract::<PyAnyCodec>() {
            Any(Self::new_builder(fitness, any_codec.codec, executor))
        } else if let Ok(permutation_codec) = codec.extract::<PyPermutationCodec>() {
            Permutation(Self::new_builder(
                fitness,
                permutation_codec.codec,
                executor,
            ))
        } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            let py_codec = Self::wrapped_codec(graph_codec.codec);
            Graph(Self::new_builder(fitness, py_codec, executor).replace_strategy(GraphReplacement))
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            let py_codec = Self::wrapped_codec(tree_codec.codec);
            Tree(Self::new_builder(fitness, py_codec, executor))
        } else {
            radiate_py_bail!("Unsupported codec type for novelty search problem");
        };

        Ok(builder)
    }

    fn new_builder<C, T>(
        fitness_fn: PyFitnessInner,
        codec: PyCodec<C, T>,
        executor: Executor,
    ) -> GeneticEngineBuilder<C, T>
    where
        C: Chromosome + PartialEq + Clone + 'static,
        T: Send + Sync + Clone + IntoPyAnyObject + 'static,
    {
        let problem = PyProblem::from((fitness_fn, codec));

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
