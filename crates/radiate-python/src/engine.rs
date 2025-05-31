use crate::codec::PyCodec;
use crate::{
    ComponentRegistry, EngineRegistry, Limit, PyBitCodec, PyCharCodec, PyEngineParam, PyGeneType,
    PyGeneration, PyProblem,
};
use crate::{ObjectValue, PyEngineBuilder, PyFloatCodec, PyIntCodec};
use pyo3::{PyObject, PyResult, Python, pyclass, pymethods};
use radiate::prelude::*;
use std::fmt::Debug;

type SingleEngine<C> = GeneticEngine<C, ObjectValue, Generation<C, ObjectValue>>;
type MultiEngine<C> = GeneticEngine<C, ObjectValue, MultiObjectiveGeneration<C>>;

pub enum EngineInner {
    Int(SingleEngine<IntChromosome<i32>>),
    IntMulti(MultiEngine<IntChromosome<i32>>),
    Float(SingleEngine<FloatChromosome>),
    FloatMulti(MultiEngine<FloatChromosome>),
    Char(SingleEngine<CharChromosome>),
    Bit(SingleEngine<BitChromosome>),
}

#[pyclass]
pub struct PyEngine {
    pub engine: Option<EngineInner>,
}

#[pymethods]
impl PyEngine {
    #[new]
    #[pyo3(signature = (gene_type, codec, fitness_func, builder))]
    pub fn new<'py>(
        py: Python<'py>,
        gene_type: PyGeneType,
        codec: PyObject,
        fitness_func: PyObject,
        builder: PyEngineBuilder,
    ) -> Self {
        let is_multi_objective = builder.objectives.len() > 1;
        let inner = if let Ok(int_codec) = codec.extract::<PyIntCodec>(py) {
            if is_multi_objective {
                EngineInner::IntMulti(
                    build_multi_objective_engine(
                        gene_type,
                        int_codec.codec,
                        fitness_func,
                        &builder,
                    )
                    .build(),
                )
            } else {
                EngineInner::Int(
                    build_single_objective_engine(
                        gene_type,
                        int_codec.codec,
                        fitness_func,
                        &builder,
                    )
                    .build(),
                )
            }
        } else if let Ok(float_codec) = codec.extract::<PyFloatCodec>(py) {
            if is_multi_objective {
                EngineInner::FloatMulti(
                    build_multi_objective_engine(
                        gene_type,
                        float_codec.codec,
                        fitness_func,
                        &builder,
                    )
                    .build(),
                )
            } else {
                EngineInner::Float(
                    build_single_objective_engine(
                        gene_type,
                        float_codec.codec,
                        fitness_func,
                        &builder,
                    )
                    .build(),
                )
            }
        } else if let Ok(char_codec) = codec.extract::<PyCharCodec>(py) {
            EngineInner::Char(
                build_single_objective_engine(gene_type, char_codec.codec, fitness_func, &builder)
                    .build(),
            )
        } else if let Ok(bit_codec) = codec.extract::<PyBitCodec>(py) {
            EngineInner::Bit(
                build_single_objective_engine(gene_type, bit_codec.codec, fitness_func, &builder)
                    .build(),
            )
        } else {
            panic!("Unsupported codec type");
        };

        PyEngine {
            engine: Some(inner),
        }
    }

    pub fn run(&mut self, limits: Vec<PyEngineParam>, log: bool) -> PyResult<PyGeneration> {
        self.engine
            .take()
            .map(|inner| match inner {
                EngineInner::Int(engine) => {
                    run_single_objective_engine(&mut Some(engine), limits, log)
                }
                EngineInner::Float(engine) => {
                    run_single_objective_engine(&mut Some(engine), limits, log)
                }
                EngineInner::Char(engine) => {
                    run_single_objective_engine(&mut Some(engine), limits, log)
                }
                EngineInner::Bit(engine) => {
                    run_single_objective_engine(&mut Some(engine), limits, log)
                }
                EngineInner::IntMulti(engine) => {
                    run_multi_objective_engine(&mut Some(engine), limits, log)
                }
                EngineInner::FloatMulti(engine) => {
                    run_multi_objective_engine(&mut Some(engine), limits, log)
                }
            })
            .unwrap_or_else(|| {
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Engine is not initialized",
                ))
            })
    }
}

pub(crate) fn build_multi_objective_engine<C>(
    gene_type: PyGeneType,
    codec: PyCodec<C>,
    fitness_func: PyObject,
    builder: &PyEngineBuilder,
) -> GeneticEngineBuilder<C, ObjectValue, MultiObjectiveGeneration<C>>
where
    C: Chromosome + PartialEq + Clone,
{
    let engine = build_single_objective_engine(gene_type, codec, fitness_func, builder);
    crate::set_multi_objective(engine, &builder)
}

pub(crate) fn build_single_objective_engine<C>(
    gene_type: PyGeneType,
    codec: PyCodec<C>,
    fitness_func: PyObject,
    py_builder: &PyEngineBuilder,
) -> GeneticEngineBuilder<C, ObjectValue, Generation<C, ObjectValue>>
where
    C: Chromosome + PartialEq + Clone,
{
    let registry = EngineRegistry::new();
    let mut builder = GeneticEngine::builder()
        .problem(PyProblem::new(fitness_func, codec))
        .population_size(py_builder.population_size)
        .offspring_fraction(py_builder.offspring_fraction)
        .max_age(py_builder.max_phenotype_age);

    builder = registry.apply(builder, py_builder, gene_type);

    builder = crate::set_evaluator(builder, &py_builder.num_threads);
    builder = crate::set_single_objective(builder, &py_builder.objectives);

    builder
}

fn run_multi_objective_engine<C, T>(
    engine: &mut Option<GeneticEngine<C, T, MultiObjectiveGeneration<C>>>,
    limits: Vec<PyEngineParam>,
    _: bool,
) -> PyResult<PyGeneration>
where
    C: Chromosome + Clone,
    T: Debug + Clone + Send + Sync,
    MultiObjectiveGeneration<C>: Into<PyGeneration>,
{
    let lims = limits.into_iter().map(Limit::from).collect::<Vec<_>>();

    engine
        .take()
        .map(|engine| {
            engine
                .iter()
                .skip_while(|epoch| {
                    lims.iter().all(|limit| match limit {
                        Limit::Generations(lim) => epoch.index() < *lim,
                        Limit::Score(_) => false,
                        Limit::Seconds(val) => return epoch.seconds() < *val,
                    })
                })
                .take(1)
                .last()
                .map(|epoch| epoch.into())
        })
        .flatten()
        .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
            "No generation found that meets the limits",
        ))
}

fn run_single_objective_engine<C, T>(
    engine: &mut Option<GeneticEngine<C, T, Generation<C, T>>>,
    limits: Vec<PyEngineParam>,
    log: bool,
) -> PyResult<PyGeneration>
where
    C: Chromosome + Clone,
    T: Debug + Clone + Send + Sync,
    Generation<C, T>: Into<PyGeneration>,
{
    let lims = limits.into_iter().map(Limit::from).collect::<Vec<_>>();

    engine
        .take()
        .map(|engine| {
            engine
                .iter()
                .inspect(|epoch| {
                    if log {
                        log_ctx!(epoch);
                    }
                })
                .skip_while(|epoch| {
                    lims.iter().all(|limit| match limit {
                        Limit::Generations(lim) => epoch.index() < *lim,
                        Limit::Score(lim) => match epoch.objective() {
                            Objective::Single(opt) => match opt {
                                Optimize::Minimize => epoch.score().as_f32() > *lim,
                                Optimize::Maximize => epoch.score().as_f32() < *lim,
                            },
                            Objective::Multi(_) => false,
                        },
                        Limit::Seconds(val) => return epoch.seconds() < *val,
                    })
                })
                .take(1)
                .last()
                .inspect(|epoch| {
                    if log {
                        println!("{:?}", epoch);
                    }
                })
                .map(|epoch| epoch.into())
        })
        .flatten()
        .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
            "No generation found that meets the limits",
        ))
}
