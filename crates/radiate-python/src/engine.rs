use crate::codec::PyCodec;
use crate::{
    EngineRegistry, Limit, PyBitCodec, PyCharCodec, PyEngineParam, PyGeneType, PyGeneration,
    PyProblem,
};
use crate::{PyEngineBuilder, PyFloatCodec, PyIntCodec, conversion::ObjectValue};
use pyo3::PyResult;
use pyo3::{PyObject, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, Epoch, Generation, GeneticEngine,
    GeneticEngineBuilder, Objective, Optimize, log_ctx,
};
use radiate::{FloatChromosome, IntChromosome};
use std::fmt::Debug;

pub enum EngineInner {
    Int(GeneticEngine<IntChromosome<i32>, ObjectValue>),
    Float(GeneticEngine<FloatChromosome, ObjectValue>),
    Char(GeneticEngine<CharChromosome, ObjectValue>),
    Bit(GeneticEngine<BitChromosome, ObjectValue>),
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
        let inner = if let Ok(int_codec) = codec.extract::<PyIntCodec>(py) {
            EngineInner::Int(
                build_single_objective_engine(gene_type, int_codec.codec, fitness_func, &builder)
                    .build(),
            )
        } else if let Ok(float_codec) = codec.extract::<PyFloatCodec>(py) {
            EngineInner::Float(
                build_single_objective_engine(gene_type, float_codec.codec, fitness_func, &builder)
                    .build(),
            )
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
            })
            .unwrap_or_else(|| {
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Engine is not initialized",
                ))
            })
    }
}

pub(crate) fn build_single_objective_engine<C>(
    gene_type: PyGeneType,
    codec: PyCodec<C>,
    fitness_func: PyObject,
    builder: &PyEngineBuilder,
) -> GeneticEngineBuilder<C, ObjectValue>
where
    C: Chromosome + PartialEq + Clone,
{
    let registry = EngineRegistry::new();
    let mut engine = GeneticEngine::builder()
        .problem(PyProblem::new(fitness_func, codec))
        .population_size(builder.population_size)
        .alter(registry.get_alters::<C>(gene_type, &builder.alters));

    engine = crate::set_evaluator(engine, &builder.num_threads);
    engine = crate::set_selector(engine, &builder.offspring_selector, true);
    engine = crate::set_selector(engine, &builder.survivor_selector, false);
    engine = crate::set_single_objective(engine, &builder.objectives);

    engine
}

pub(crate) fn run_single_objective_engine<C, T>(
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
