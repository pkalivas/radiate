use crate::bindings::codec::{PyGraphCodecInner, PyTreeCodecInner};
use crate::events::PyEventHandler;
use crate::{
    EngineBuilderHandle, FreeThreadPyEvaluator, InputTransform, PyCodec, PyEngine, PyEngineInput,
    PyEngineInputType, PyExpr, PyFitnessFn, PyFitnessInner, PyPermutationCodec, PyPopulation,
    prelude::*, radiate,
};
use crate::{
    PyCheckpointReader,
    bindings::codec::{PyTreeCodec, TypedNumericCodec},
};
use crate::{PyGeneration, PySubscriber};
use core::panic;
use pyo3::{Py, PyAny, pyclass, pymethods, types::PyAnyMethods};
use radiate::prelude::*;
use radiate_error::{ResultExt, radiate_py_bail, radiate_py_err};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
        use crate::EngineBuilderHandle::*;
        match $builder {
            UInt8(b) => $call(b).map(UInt8),
            UInt16(b) => $call(b).map(UInt16),
            UInt32(b) => $call(b).map(UInt32),
            UInt64(b) => $call(b).map(UInt64),
            Int8(b) => $call(b).map(Int8),
            Int16(b) => $call(b).map(Int16),
            Int32(b) => $call(b).map(Int32),
            Int64(b) => $call(b).map(Int64),
            Float32(b) => $call(b).map(Float32),
            Float64(b) => $call(b).map(Float64),
            Char(b) => $call(b).map(Char),
            Bit(b) => $call(b).map(Bit),
            Permutation(b) => $call(b).map(Permutation),
            Graph32(b) => $call(b).map(Graph32),
            Graph64(b) => $call(b).map(Graph64),
            Tree32(b) => $call(b).map(Tree32),
            Tree64(b) => $call(b).map(Tree64),
            Empty => Err(radiate_py_err!("Cannot apply method to Empty builder"))
        }
    }};
}

#[pyclass]
pub struct PyEngineBuilder {
    pub inputs: Vec<PyEngineInput>,
}

#[pymethods]
impl PyEngineBuilder {
    #[new]
    pub fn new(inputs: Vec<PyEngineInput>) -> PyResult<Self> {
        Ok(PyEngineBuilder { inputs })
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

        let limits = input_groups
            .get(&PyEngineInputType::Limit)
            .map(|inputs| inputs.transform())
            .unwrap_or_default();

        Ok(PyEngine::new(limits, inner.try_build()?))
    }
}

impl PyEngineBuilder {
    fn create_builder<'py>(&self, py: Python<'py>) -> PyResult<EngineBuilderHandle> {
        use PyFitnessInner::*;

        let (fitness, codec, executor) = Self::get_and_check_base_inputs(py, &self.inputs)?;

        let codec = codec.bind(py);
        match fitness {
            custom @ Custom(..) => Self::init_custom_builder(custom, codec, executor),
            reg32 @ Regression32(..) => Self::init_regression_builder32(reg32, codec, executor),
            reg64 @ Regression64(..) => Self::init_regression_builder64(reg64, codec, executor),
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
            Generation => Self::process_generation(builder, inputs),
            Checkpoint => Self::process_checkpoint(builder, inputs),
            Metric => Self::process_metrics(builder, inputs),
            Filter => Self::process_filters(builder, inputs),
            TargetSpecies => Self::process_target_species(builder, inputs),
            _ => Ok(builder),
        }
    }

    fn process_metrics(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_many_typed(|typed_builder, metric_inputs| {
                let mut metrics = Vec::with_capacity(metric_inputs.len());
                for input in metric_inputs {
                    let name = input.extract::<String>("name")?;
                    let expr = input.extract::<PyExpr>("expr")?;

                    metrics.push(expr.inner().clone().alias(name));
                }

                if metrics.is_empty() {
                    return Ok(typed_builder);
                }

                Ok(typed_builder.metrics(metrics))
            })
        )
    }

    fn process_filters(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_many_typed(|typed_builder, inputs| {
                let filters = InputTransform::<
                    RadiateResult<Vec<Arc<Mutex<dyn EcosystemFilter<_>>>>>,
                >::transform(&inputs)
                .context("Failed to transform filter input")?;
                Ok(typed_builder.filters(filters))
            })
        )
    }

    fn process_generation(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let generation = input.extract::<PyGeneration>("generation")?;
                let ecosystem = Ecosystem::from(generation.ecosystem());
                Ok(typed_builder.ecosystem(ecosystem))
            })
        )
    }

    fn process_checkpoint(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let ignore_not_found = input.extract::<bool>("ignore_not_found").unwrap_or(false);
                let path = input.extract::<String>("path")?;
                let file_type = input.extract::<String>("file_type")?;

                if ignore_not_found
                    && let Err(e) = std::fs::metadata(&path)
                    && e.kind() == std::io::ErrorKind::NotFound
                {
                    return Ok(typed_builder);
                }

                Ok(typed_builder.load_checkpoint(path, PyCheckpointReader(file_type)))
            })
        )
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
                let max_age = input.extract::<i64>("age")? as usize;
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
                let threshold = if let Ok(expr) = input.extract::<PyExpr>("threshold") {
                    expr.inner().clone().compile()
                } else {
                    let val = input.extract::<f64>("threshold").unwrap_or(0.5);
                    if val <= 0.0 {
                        return Err(radiate_py_err!("Species threshold must be greater than 0."));
                    }

                    Expr::lit(val)
                };

                Ok(typed_builder.species_threshold(threshold))
            })
        )
    }

    fn process_target_species(
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        dispatch_builder_typed!(
            builder,
            inputs,
            Self::process_single_typed(|typed_builder, input| {
                let target_species = input.extract::<i64>("target_species")? as usize;
                Ok(typed_builder.target_species(target_species))
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
                let fraction = input.extract::<f64>("fraction").map_err(|e| {
                    radiate_py_err!(format!("Invalid offspring fraction value: {}", e))
                })?;

                if !(0.0..=1.0).contains(&fraction) {
                    return Err(radiate_py_err!(
                        "offspring fraction must be in the range [0.0, 1.0]"
                    ));
                }

                Ok(typed_builder.offspring_fraction(fraction as f32))
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
                let size = input.extract::<i64>("size").map_err(|e| {
                    radiate_py_err!(format!("Failed to extract population size value: {}", e))
                })?;
                Ok(typed_builder.population_size(size as usize))
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
                let max_age = input.extract::<i64>("age").map_err(|e| {
                    radiate_py_err!(format!("Failed to extract max phenotype age value: {}", e))
                })?;

                Ok(typed_builder.max_age(max_age as usize))
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
                let selector =
                    InputTransform::<RadiateResult<Box<dyn Select<_>>>>::transform(input)
                        .context("Failed to transform selector input")?;

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
                let alters =
                    InputTransform::<RadiateResult<Vec<Alterer<_>>>>::transform(&alter_inputs)
                        .context("Failed to transform alterers input")?;
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
                let diversity =
                    InputTransform::<RadiateResult<Box<dyn Diversity<_>>>>::transform(input)
                        .context("Failed to transform diversity input")?;

                Ok(typed_builder.boxed_diversity(Some(diversity)))
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
                let objectives = input.extract::<Vec<String>>("objective").map(|objs| {
                    objs.iter()
                        .map(|val| match val.trim().to_lowercase().as_str() {
                            "min" => Optimize::Minimize,
                            "max" => Optimize::Maximize,
                            _ => panic!("Objective {} not recognized", val),
                        })
                        .collect::<Vec<Optimize>>()
                })?;

                let opt = if objectives.len() == 1 {
                    Objective::Single(objectives[0])
                } else if objectives.len() > 1 {
                    Objective::Multi(objectives)
                } else {
                    radiate_py_bail!("No objectives provided - I'm not even sure this is possible");
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
                let min = input.extract::<i64>("min")? as usize;
                let max = input.extract::<i64>("max")? as usize;

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

    fn init_custom_builder<'py>(
        fitness: PyFitnessInner,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        use EngineBuilderHandle::*;
        use PyFitnessInner::Custom;

        if !matches!(fitness, Custom(_, _)) {
            radiate_py_bail!("init_custom_handle only supports Custom fitness functions")
        }

        let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
            match float_codec.codec {
                TypedNumericCodec::F32(c) => Float32(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::F64(c) => Float64(Self::new_builder(fitness, c, executor)),
                _ => {
                    radiate_py_bail!("Unsupported float codec type for custom fitness function");
                }
            }
        } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
            match int_codec.codec {
                TypedNumericCodec::U8(c) => UInt8(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::U16(c) => UInt16(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::U32(c) => UInt32(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::U64(c) => UInt64(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I8(c) => Int8(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I16(c) => Int16(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I32(c) => Int32(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I64(c) => Int64(Self::new_builder(fitness, c, executor)),
                _ => {
                    radiate_py_bail!("Unsupported integer codec type for custom fitness function");
                }
            }
        } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
            Char(Self::new_builder(fitness, char_codec.codec, executor))
        } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
            Bit(Self::new_builder(fitness, bit_codec.codec, executor))
        } else if let Ok(perm_codec) = codec.extract::<PyPermutationCodec>() {
            Permutation(Self::new_builder(fitness, perm_codec.codec, executor))
        } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            match graph_codec.codec {
                PyGraphCodecInner::Float32(c) => {
                    let py_codec = Self::wrapped_codec(c);
                    Graph32(
                        Self::new_builder(fitness, py_codec, executor)
                            .replace_strategy(GraphReplacement),
                    )
                }
                PyGraphCodecInner::Float64(c) => {
                    let py_codec = Self::wrapped_codec(c);
                    Graph64(
                        Self::new_builder(fitness, py_codec, executor)
                            .replace_strategy(GraphReplacement),
                    )
                }
            }
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            match tree_codec.codec {
                PyTreeCodecInner::Float32(c) => {
                    let py_codec = Self::wrapped_codec(c);
                    Tree32(Self::new_builder(fitness, py_codec, executor))
                }
                PyTreeCodecInner::Float64(c) => {
                    let py_codec = Self::wrapped_codec(c);
                    Tree64(Self::new_builder(fitness, py_codec, executor))
                }
            }
        } else {
            radiate_py_bail!("Unsupported codec type for custom fitness function");
        };

        Ok(builder)
    }

    fn init_regression_builder32<'py>(
        regression: PyFitnessInner,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        use EngineBuilderHandle::*;
        use PyFitnessInner::Regression32;

        let Regression32(regression, is_batch) = regression else {
            radiate_py_bail!("init_regression_builder only supports Regression fitness functions")
        };

        if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            if let PyGraphCodecInner::Float32(c) = graph_codec.codec {
                let base_engine = GeneticEngine::builder()
                    .codec(c)
                    .executor(executor)
                    .bus_executor(Executor::default())
                    .replace_strategy(GraphReplacement);

                return if is_batch {
                    Ok(Graph32(base_engine.raw_batch_fitness_fn(regression)))
                } else {
                    Ok(Graph32(base_engine.raw_fitness_fn(regression)))
                };
            } else {
                radiate_py_bail!("F64 GraphCodec not supported for F32 Regression Fitness.");
            }
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            if let PyTreeCodecInner::Float32(c) = tree_codec.codec {
                let base_engine = GeneticEngine::builder()
                    .codec(c)
                    .executor(executor)
                    .bus_executor(Executor::default());

                return if is_batch {
                    Ok(Tree32(base_engine.raw_batch_fitness_fn(regression)))
                } else {
                    Ok(Tree32(base_engine.raw_fitness_fn(regression)))
                };
            } else {
                radiate_py_bail!("F64 TreeCodec not supported for F32 Regression Fitness.");
            }
        } else {
            radiate_py_bail!("Only Graph or Tree codecs are supported for regression problems");
        };
    }

    fn init_regression_builder64<'py>(
        regression: PyFitnessInner,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        use EngineBuilderHandle::*;
        use PyFitnessInner::Regression64;

        let Regression64(regression, is_batch) = regression else {
            radiate_py_bail!("init_regression_builder64 only supports Regression fitness functions")
        };

        if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            if let PyGraphCodecInner::Float64(c) = graph_codec.codec {
                let base_engine = GeneticEngine::builder()
                    .codec(c)
                    .executor(executor)
                    .bus_executor(Executor::default())
                    .replace_strategy(GraphReplacement);

                return if is_batch {
                    Ok(Graph64(base_engine.raw_batch_fitness_fn(regression)))
                } else {
                    Ok(Graph64(base_engine.raw_fitness_fn(regression)))
                };
            } else {
                radiate_py_bail!("F32 GraphCodec not supported for F64 Regression Fitness.");
            }
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            if let PyTreeCodecInner::Float64(c) = tree_codec.codec {
                let base_engine = GeneticEngine::builder()
                    .codec(c)
                    .executor(executor)
                    .bus_executor(Executor::default());

                return if is_batch {
                    Ok(Tree64(base_engine.raw_batch_fitness_fn(regression)))
                } else {
                    Ok(Tree64(base_engine.raw_fitness_fn(regression)))
                };
            } else {
                radiate_py_bail!("F32 TreeCodec not supported for F64 Regression Fitness.");
            }
        } else {
            radiate_py_bail!("Only Graph or Tree codecs are supported for regression problems");
        };
    }

    fn init_novelty_builder<'py>(
        fitness: PyFitnessInner,
        codec: &Bound<'py, PyAny>,
        executor: Executor,
    ) -> PyResult<EngineBuilderHandle> {
        use EngineBuilderHandle::*;
        use PyFitnessInner::NoveltySearch;

        if !matches!(fitness, NoveltySearch(_, _)) {
            radiate_py_bail!("init_novelty_builder only supports Novelty Search fitness functions")
        }

        let builder = if let Ok(float_codec) = codec.extract::<PyFloatCodec>() {
            match float_codec.codec {
                TypedNumericCodec::F32(c) => Float32(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::F64(c) => Float64(Self::new_builder(fitness, c, executor)),
                _ => {
                    radiate_py_bail!("Unsupported float codec type for novelty search problem");
                }
            }
        } else if let Ok(int_codec) = codec.extract::<PyIntCodec>() {
            match int_codec.codec {
                TypedNumericCodec::U8(c) => UInt8(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::U16(c) => UInt16(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::U32(c) => UInt32(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::U64(c) => UInt64(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I8(c) => Int8(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I16(c) => Int16(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I32(c) => Int32(Self::new_builder(fitness, c, executor)),
                TypedNumericCodec::I64(c) => Int64(Self::new_builder(fitness, c, executor)),
                _ => {
                    radiate_py_bail!("Unsupported integer codec type for novelty search problem");
                }
            }
        } else if let Ok(char_codec) = codec.extract::<PyCharCodec>() {
            Char(Self::new_builder(fitness, char_codec.codec, executor))
        } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>() {
            Bit(Self::new_builder(fitness, bit_codec.codec, executor))
        } else if let Ok(permutation_codec) = codec.extract::<PyPermutationCodec>() {
            Permutation(Self::new_builder(
                fitness,
                permutation_codec.codec,
                executor,
            ))
        } else if let Ok(graph_codec) = codec.extract::<PyGraphCodec>() {
            match graph_codec.codec {
                PyGraphCodecInner::Float32(c) => Graph32(
                    Self::new_builder(fitness, Self::wrapped_codec(c), executor)
                        .replace_strategy(GraphReplacement),
                ),
                PyGraphCodecInner::Float64(c) => Graph64(
                    Self::new_builder(fitness, Self::wrapped_codec(c), executor)
                        .replace_strategy(GraphReplacement),
                ),
            }
        } else if let Ok(tree_codec) = codec.extract::<PyTreeCodec>() {
            match tree_codec.codec {
                PyTreeCodecInner::Float32(c) => {
                    let py_codec = Self::wrapped_codec(c);
                    Tree32(Self::new_builder(fitness, py_codec, executor))
                }
                PyTreeCodecInner::Float64(c) => {
                    let py_codec = Self::wrapped_codec(c);
                    Tree64(Self::new_builder(fitness, py_codec, executor))
                }
            }
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
        PC: Codec<C, T> + Clone + Send + Sync + 'static,
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

    fn get_and_check_base_inputs<'py>(
        py: Python<'py>,
        inputs: &[PyEngineInput],
    ) -> PyResult<(PyFitnessInner, Py<PyAny>, Executor)> {
        use PyFitnessInner::*;

        let gil_enabled = radiate(py)
            .getattr(py, "_GIL_ENABLED")?
            .extract::<bool>(py)?;

        let codec = Self::get_input_of_type(inputs, PyEngineInputType::Codec)
            .map(|codec| codec.extract::<Py<PyAny>>("codec"))
            .unwrap_or(Err(radiate_py_err!(
                "EngineBuilder requires a Codec input."
            )))?;

        let problem = Self::get_input_of_type(inputs, PyEngineInputType::FitnessFunction)
            .map(|fitness| fitness.extract::<PyFitnessFn>("fitness"))
            .unwrap_or(Err(radiate_py_err!(
                "EngineBuilder requires a FitnessFunction input."
            )))?;

        let executor = Self::get_input_of_type(inputs, PyEngineInputType::Executor)
            .map(InputTransform::<RadiateResult<Executor>>::transform)
            .unwrap_or(Ok(Executor::Serial))?;

        if executor.is_parallel()
            && gil_enabled
            && !matches!(problem.inner, Regression32(_, _) | Regression64(_, _))
        {
            radiate_py_bail!(
                "Parallel execution is not supported for non-regression fitness functions
                 when the GIL is enabled. Please disable the GIL or use the 'Executor.Serial()' executor."
            );
        }

        Ok((problem.inner, codec, executor))
    }

    fn get_input_of_type(
        inputs: &[PyEngineInput],
        input_type: PyEngineInputType,
    ) -> Option<&PyEngineInput> {
        inputs
            .iter()
            .rev()
            .find(|inp| inp.input_type() == input_type)
    }
}
