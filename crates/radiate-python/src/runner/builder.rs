use super::{PyAlterer, PyObjective, PySelector, objective};
use crate::{
    FreeThreadPyEvaluator, ObjectValue, PyBitCodec, PyCharCodec, PyFloatCodec, PyGeneType,
    PyGeneration, PyIntCodec, PyLimit, PyProblem,
    codec::PyCodec,
    conversion::{Wrap, py_object_to_any_value},
};
use core::panic;
use pyo3::{
    Bound, FromPyObject, IntoPyObjectExt, Py, PyAny, PyErr, PyObject, PyResult, Python, pyclass,
    pymethods,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods, PyString, PyTuple},
};
use radiate::{
    Alter, BitChromosome, CharChromosome, Chromosome, Epoch, Executor, FloatChromosome, Generation,
    GeneticEngine, GeneticEngineBuilder, IntChromosome, MultiObjectiveGeneration, Objective,
    Optimize, Select, log_ctx, ops,
};
use std::{fmt::Debug, sync::Arc, vec};

#[pyclass]
pub struct EngineBuilderTemp {
    fitness_func: PyObject,
    codec: PyObject,
    params: Py<PyDict>,
}

#[pymethods]
impl EngineBuilderTemp {
    #[new]
    #[pyo3(signature = (fitness_func, codec, **kwds))]
    pub fn new<'py>(
        py: Python<'py>,
        fitness_func: PyObject,
        codec: PyObject,
        kwds: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let params = kwds.map(|d| d.to_owned()).unwrap_or(PyDict::new(py));

        Ok(Self {
            fitness_func,
            codec,
            params: params.into(),
        })
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let repr = format!(
            "EngineBuilderTemp(
                population_size={},
                offspring_fraction={},
                alters={},
                survivor_selector={},
                offspring_selector={},
                objective={},
                front_range={}
                )",
            self.get_population_size(py)?,
            self.get_offspring_fraction(py)?,
            self.get_alters(py)?
                .iter()
                .map(|a| a.name())
                .collect::<Vec<_>>()
                .join(", "),
            self.get_survivor_selector(py)?.name(),
            self.get_offspring_selector(py)?.name(),
            self.get_objective(py)?.optimize().join(", "),
            self.get_front_range(py)?
                .bind_borrowed(py)
                .try_iter()
                .iter()
                .take(2)
                .map(|v| v.extract::<usize>().unwrap_or(0))
                .collect::<Vec<_>>()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        PyString::new(py, &repr).into_bound_py_any(py)
    }

    pub fn run<'py>(&self, py: Python<'py>, limits: Py<PyAny>) -> PyResult<PyGeneration> {
        let limits = if let Ok(limits) = limits.extract::<Vec<PyLimit>>(py) {
            limits
        } else if let Ok(limits) = limits.extract::<Bound<'_, PyList>>(py) {
            limits
                .iter()
                .map(|l| l.extract::<PyLimit>())
                .collect::<PyResult<Vec<_>>>()?
        } else if let Ok(limit) = limits.extract::<PyLimit>(py) {
            vec![limit]
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a list of limits",
            ));
        };

        if limits.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "At least one limit must be specified",
            ));
        }

        let param_dict = self.create_param_dict(py)?;

        if let Ok(_) = self.get_gene_type(py) {
            let engine = param_dict.extract::<Wrap<EngineWrapper>>(py)?.0;

            match engine {
                EngineWrapper::Int(int_engine) => {
                    let mut engine = Some(int_engine);
                    let generation = run_single_objective_engine(&mut engine, limits, true)?;
                    return Ok(generation);
                }
                EngineWrapper::Float(float_engine) => {
                    let mut engine = Some(float_engine);
                    let generation = run_single_objective_engine(&mut engine, limits, true)?;
                    return Ok(generation);
                }
                EngineWrapper::Char(char_engine) => {
                    let mut engine = Some(char_engine);
                    let generation = run_single_objective_engine(&mut engine, limits, true)?;
                    return Ok(generation);
                }
                EngineWrapper::Bit(bit_engine) => {
                    let mut engine = Some(bit_engine);
                    let generation = run_single_objective_engine(&mut engine, limits, true)?;
                    return Ok(generation);
                }
                EngineWrapper::IntMulti(int_multi_engine) => {
                    let mut engine = Some(int_multi_engine);
                    let generation = run_multi_objective_engine(&mut engine, limits, true)?;
                    return Ok(generation.into());
                }
                EngineWrapper::FloatMulti(float_multi_engine) => {
                    let mut engine = Some(float_multi_engine);
                    let generation = run_multi_objective_engine(&mut engine, limits, true)?;
                    return Ok(generation.into());
                }
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Unsupported engine type",
                    ));
                }
            }
        }

        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported gene type",
        ))
    }

    #[pyo3(signature = (size=100))]
    pub fn set_population_size<'py>(&self, py: Python<'py>, size: usize) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("population_size", size)
            .map_err(|e| e.into())
    }

    #[pyo3(signature = (fraction=0.8))]
    pub fn set_offspring_fraction<'py>(&self, py: Python<'py>, fraction: f32) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("offspring_fraction", fraction)
            .map_err(|e| e.into())
    }

    pub fn set_alters<'py>(&self, py: Python<'py>, alters: Vec<PyAlterer>) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("alters", PyList::new(py, alters)?)
            .map_err(|e| e.into())
    }

    pub fn set_survivor_selector<'py>(&self, py: Python<'py>, selector: Py<PyAny>) -> PyResult<()> {
        let selector = selector.extract::<PySelector>(py)?;
        self.params
            .bind(py)
            .set_item("survivor_selector", selector)
            .map_err(|e| e.into())
    }

    pub fn set_offspring_selector<'py>(
        &self,
        py: Python<'py>,
        selector: Py<PyAny>,
    ) -> PyResult<()> {
        let selector = selector.extract::<PySelector>(py)?;
        self.params
            .bind(py)
            .set_item("offspring_selector", selector)
            .map_err(|e| e.into())
    }

    pub fn set_objective<'py>(&self, py: Python<'py>, objective: Py<PyAny>) -> PyResult<()> {
        let objective = objective.extract::<PyObjective>(py)?;
        self.params
            .bind(py)
            .set_item("objective", objective)
            .map_err(|e| e.into())
    }

    pub fn set_species_threshold<'py>(&self, py: Python<'py>, threshold: f32) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("species_threshold", threshold)
            .map_err(|e| e.into())
    }

    pub fn set_max_phenotype_age<'py>(&self, py: Python<'py>, max_age: usize) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("max_phenotype_age", max_age)
            .map_err(|e| e.into())
    }

    pub fn set_front_range<'py>(&self, py: Python<'py>, front_range: Py<PyAny>) -> PyResult<()> {
        let front_range = front_range.extract::<Py<PyTuple>>(py)?;

        if front_range.bind_borrowed(py).len()? != 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "front_range must be a tuple of twqwdwo values (min, maxssss)",
            ));
        }

        self.params
            .bind(py)
            .set_item("front_range", front_range)
            .map_err(|e| e.into())
    }

    pub fn set_num_threads<'py>(&self, py: Python<'py>, num_threads: usize) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("num_threads", num_threads)
            .map_err(|e| e.into())
    }

    pub fn set_max_species_age<'py>(&self, py: Python<'py>, max_age: usize) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("max_species_age", max_age)
            .map_err(|e| e.into())
    }

    fn get_population_size<'py>(&self, py: Python<'py>) -> PyResult<usize> {
        self.params
            .bind(py)
            .get_item("population_size")?
            .map(|v| v.extract::<usize>())
            .unwrap_or(Ok(100))
    }

    pub fn get_offspring_fraction<'py>(&self, py: Python<'py>) -> PyResult<f32> {
        self.params
            .bind(py)
            .get_item("offspring_fraction")?
            .map(|v| v.extract::<f32>())
            .unwrap_or(Ok(0.8))
    }

    pub fn get_max_phenotype_age<'py>(&self, py: Python<'py>) -> PyResult<usize> {
        self.params
            .bind(py)
            .get_item("max_phenotype_age")?
            .map(|v| v.extract::<usize>())
            .unwrap_or(Ok(20))
    }

    pub fn get_species_threshold<'py>(&self, py: Python<'py>) -> PyResult<f32> {
        self.params
            .bind(py)
            .get_item("species_threshold")?
            .map(|v| v.extract::<f32>())
            .unwrap_or(Ok(0.1))
    }

    pub fn get_max_species_age<'py>(&self, py: Python<'py>) -> PyResult<usize> {
        self.params
            .bind(py)
            .get_item("max_species_age")?
            .map(|v| v.extract::<usize>())
            .unwrap_or(Ok(10))
    }

    pub fn get_alters<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyAlterer>> {
        Ok(self
            .params
            .bind(py)
            .get_item("alters")?
            .map(|v| v.extract::<Vec<PyAlterer>>().ok())
            .flatten()
            .unwrap_or(vec![
                PyAlterer::uniform_crossover(py, Some(0.5))?,
                PyAlterer::uniform_mutator(py, Some(0.1))?,
            ]))
    }

    pub fn get_survivor_selector<'py>(&self, py: Python<'py>) -> PyResult<PySelector> {
        self.params
            .bind(py)
            .get_item("survivor_selector")?
            .map(|v| v.extract::<PySelector>())
            .unwrap_or(Ok(PySelector::tournament_selector(py, Some(2))?))
    }

    pub fn get_offspring_selector<'py>(&self, py: Python<'py>) -> PyResult<PySelector> {
        self.params
            .bind(py)
            .get_item("offspring_selector")?
            .map(|v| v.extract::<PySelector>())
            .unwrap_or(Ok(PySelector::roulette_wheel_selector(py)?))
    }

    pub fn get_objective<'py>(&self, py: Python<'py>) -> PyResult<PyObjective> {
        self.params
            .bind(py)
            .get_item("objective")?
            .map(|v| v.extract::<PyObjective>())
            .unwrap_or(Ok(PyObjective::min()?))
    }

    pub fn get_front_range<'py>(&self, py: Python<'py>) -> PyResult<Py<PyTuple>> {
        let range = self.params.bind(py).get_item("front_range")?;
        if let Some(range) = range {
            if let Ok(tuple) = range.extract::<Py<PyTuple>>() {
                return Ok(tuple);
            }
        }

        let result = PyTuple::new(py, vec![800, 900])?.unbind();
        Ok(result)
    }

    pub fn get_num_threads<'py>(&self, py: Python<'py>) -> PyResult<usize> {
        self.params
            .bind(py)
            .get_item("num_threads")?
            .map(|v| v.extract::<usize>())
            .unwrap_or(Ok(1))
    }

    pub fn get_gene_type<'py>(&self, py: Python<'py>) -> PyResult<PyGeneType> {
        let codec_obj = self.codec.bind(py);

        if let Ok(_) = codec_obj.extract::<PyIntCodec>() {
            Ok(PyGeneType::Int)
        } else if let Ok(_) = codec_obj.extract::<PyFloatCodec>() {
            Ok(PyGeneType::Float)
        } else if let Ok(_) = codec_obj.extract::<PyBitCodec>() {
            Ok(PyGeneType::Bit)
        } else if let Ok(_) = codec_obj.extract::<PyCharCodec>() {
            Ok(PyGeneType::Char)
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Unsupported gene type",
            ))
        }
    }

    fn create_param_dict<'py>(&self, py: Python<'py>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("population_size", self.get_population_size(py)?)?;
        dict.set_item("offspring_fraction", self.get_offspring_fraction(py)?)?;
        dict.set_item("alters", self.get_alters(py)?)?;
        dict.set_item("survivor_selector", self.get_survivor_selector(py)?)?;
        dict.set_item("offspring_selector", self.get_offspring_selector(py)?)?;
        dict.set_item("objective", self.get_objective(py)?)?;
        dict.set_item("gene_type", self.get_gene_type(py)?)?;
        dict.set_item("fitness_func", self.fitness_func.clone_ref(py))?;
        dict.set_item("codec", self.codec.clone_ref(py))?;
        dict.set_item("front_range", self.get_front_range(py)?)?;
        dict.set_item("num_threads", self.get_num_threads(py)?)?;
        dict.set_item("max_phenotype_age", self.get_max_phenotype_age(py)?)?;
        dict.set_item("species_threshold", self.get_species_threshold(py)?)?;
        dict.set_item("max_species_age", self.get_max_species_age(py)?)?;

        Ok(dict.into())
    }
}

type SingleObjectiveEngine<C> = GeneticEngine<C, ObjectValue, Generation<C, ObjectValue>>;
type MultiObjectiveEngine<C> = GeneticEngine<C, ObjectValue, MultiObjectiveGeneration<C>>;

pub enum EngineWrapper {
    Int(SingleObjectiveEngine<IntChromosome<i32>>),
    Float(SingleObjectiveEngine<FloatChromosome>),
    Char(SingleObjectiveEngine<CharChromosome>),
    Bit(SingleObjectiveEngine<BitChromosome>),
    IntMulti(MultiObjectiveEngine<IntChromosome<i32>>),
    FloatMulti(MultiObjectiveEngine<FloatChromosome>),
    CharMulti(MultiObjectiveEngine<CharChromosome>),
    BitMulti(MultiObjectiveEngine<BitChromosome>),
}

impl<'py> FromPyObject<'py> for Wrap<EngineWrapper> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let params = ob.extract::<Py<PyAny>>()?;

        let gene_type = params
            .bind(ob.py())
            .get_item("gene_type")?
            .extract::<PyGeneType>()?;

        let fitness_fn = params
            .bind(ob.py())
            .get_item("fitness_func")?
            .extract::<Py<PyAny>>()?;
        let codec_obj = params.bind(ob.py()).get_item("codec")?;
        let objective = params
            .bind(ob.py())
            .get_item("objective")?
            .extract::<PyObjective>()?;

        let engine = match gene_type {
            PyGeneType::Int => {
                if let Ok(codec) = codec_obj.extract::<PyIntCodec>() {
                    match objective.is_multi() {
                        true => Ok(EngineWrapper::IntMulti(
                            create_multi_engine(ob.py(), codec.codec, fitness_fn, &params)?.build(),
                        )),
                        false => Ok(EngineWrapper::Int(
                            create_engine(ob.py(), codec.codec, fitness_fn, &params)?.build(),
                        )),
                    }
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Expected an IntCodec for IntChromosome",
                    ));
                }
            }
            PyGeneType::Float => {
                if let Ok(codec) = codec_obj.extract::<PyFloatCodec>() {
                    match objective.is_multi() {
                        true => Ok(EngineWrapper::FloatMulti(
                            create_multi_engine(ob.py(), codec.codec, fitness_fn, &params)?.build(),
                        )),
                        false => Ok(EngineWrapper::Float(
                            create_engine(ob.py(), codec.codec, fitness_fn, &params)?.build(),
                        )),
                    }
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Expected a FloatCodec for FloatChromosome",
                    ));
                }
            }
            PyGeneType::Char => {
                if let Ok(codec) = codec_obj.extract::<PyCharCodec>() {
                    Ok(EngineWrapper::Char(
                        create_engine(ob.py(), codec.codec, fitness_fn, &params)?.build(),
                    ))
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Expected a CharCodec for CharChromosome",
                    ));
                }
            }
            PyGeneType::Bit => {
                if let Ok(codec) = codec_obj.extract::<PyBitCodec>() {
                    Ok(EngineWrapper::Bit(
                        create_engine(ob.py(), codec.codec, fitness_fn, &params)?.build(),
                    ))
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Expected a BitCodec for BitChromosome",
                    ));
                }
            }
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Unsupported gene type",
                ));
            }
        };

        engine.map(Wrap)
    }
}

fn create_multi_engine<C, T>(
    py: Python<'_>,
    codec: PyCodec<C>,
    fitness_fn: Py<PyAny>,
    params: &Py<PyAny>,
) -> PyResult<GeneticEngineBuilder<C, T, MultiObjectiveGeneration<C>>>
where
    C: Chromosome + Clone + PartialEq + 'static,
    T: Clone + Send + Sync + 'static,
{
    let population_size = params
        .bind(py)
        .get_item("population_size")?
        .extract::<usize>()?;

    let offspring_fraction = params
        .bind(py)
        .get_item("offspring_fraction")?
        .extract::<f32>()?;

    let alters = params
        .bind(py)
        .get_item("alters")?
        .into_bound_py_any(py)?
        .extract::<Wrap<Vec<Box<dyn Alter<C>>>>>()?
        .0;

    let survivor_selector = params
        .bind(py)
        .get_item("survivor_selector")?
        .extract::<Wrap<Box<dyn Select<C>>>>()?
        .0;

    let offspring_selector = params
        .bind(py)
        .get_item("offspring_selector")?
        .extract::<Wrap<Box<dyn Select<C>>>>()?
        .0;

    let objective = params
        .bind(py)
        .get_item("objective")?
        .extract::<Wrap<Objective>>()?
        .0;

    let front_range = params
        .bind(py)
        .get_item("front_range")?
        .extract::<Py<PyTuple>>()?;

    let num_threads = params
        .bind(py)
        .get_item("num_threads")?
        .extract::<usize>()?;

    let max_age = params
        .bind(py)
        .get_item("max_phenotype_age")?
        .extract::<usize>()?;

    let max_species_age = params
        .bind(py)
        .get_item("max_species_age")?
        .extract::<usize>()?;

    let species_threshold = params
        .bind(py)
        .get_item("species_threshold")?
        .extract::<f32>()?;

    let executor = if num_threads > 1 {
        Arc::new(Executor::worker_pool(num_threads))
    } else {
        Arc::new(Executor::Serial)
    };

    let first_front = front_range.bind(py).get_item(0)?.extract::<usize>()?;
    let second_front = front_range.bind(py).get_item(1)?.extract::<usize>()?;

    let builder = GeneticEngine::builder()
        .problem(PyProblem::new(fitness_fn, codec))
        .population_size(population_size)
        .offspring_fraction(offspring_fraction)
        .max_age(max_age)
        .max_species_age(max_species_age)
        .species_threshold(species_threshold)
        .with_values(|config| {
            config.survivor_selector = survivor_selector.into();
            config.offspring_selector = offspring_selector.into();
        })
        .alter(alters)
        .multi_objective(match objective {
            Objective::Multi(opts) => opts,
            _ => vec![Optimize::Minimize],
        })
        .front_size(first_front..second_front)
        .evaluator(FreeThreadPyEvaluator::new(executor.clone()))
        .executor(executor.clone());

    Ok(unsafe { std::mem::transmute(builder) })
}

fn create_engine<C, T>(
    py: Python<'_>,
    codec: PyCodec<C>,
    fitness_fn: Py<PyAny>,
    params: &Py<PyAny>,
) -> PyResult<GeneticEngineBuilder<C, T, Generation<C, T>>>
where
    C: Chromosome + Clone + PartialEq + 'static,
    T: Clone + Send + Sync + 'static,
{
    let population_size = params
        .bind(py)
        .get_item("population_size")?
        .extract::<usize>()?;

    let offspring_fraction = params
        .bind(py)
        .get_item("offspring_fraction")?
        .extract::<f32>()?;

    let alters = params
        .bind(py)
        .get_item("alters")?
        .into_bound_py_any(py)?
        .extract::<Wrap<Vec<Box<dyn Alter<C>>>>>()?
        .0;

    let survivor_selector = params
        .bind(py)
        .get_item("survivor_selector")?
        .extract::<Wrap<Box<dyn Select<C>>>>()?
        .0;

    let offspring_selector = params
        .bind(py)
        .get_item("offspring_selector")?
        .extract::<Wrap<Box<dyn Select<C>>>>()?
        .0;

    let objective = params
        .bind(py)
        .get_item("objective")?
        .extract::<Wrap<Objective>>()?
        .0;

    let max_age = params
        .bind(py)
        .get_item("max_phenotype_age")?
        .extract::<usize>()?;

    let max_species_age = params
        .bind(py)
        .get_item("max_species_age")?
        .extract::<usize>()?;

    let species_threshold = params
        .bind(py)
        .get_item("species_threshold")?
        .extract::<f32>()?;

    let num_threads = params
        .bind(py)
        .get_item("num_threads")?
        .extract::<usize>()?;

    let executor = if num_threads > 1 {
        Arc::new(Executor::worker_pool(num_threads))
    } else {
        Arc::new(Executor::Serial)
    };

    let builder = GeneticEngine::builder()
        .problem(PyProblem::new(fitness_fn, codec))
        .population_size(population_size)
        .offspring_fraction(offspring_fraction)
        .max_age(max_age)
        .max_species_age(max_species_age)
        .species_threshold(species_threshold)
        .evaluator(FreeThreadPyEvaluator::new(executor.clone()))
        .executor(executor.clone())
        .alter(alters)
        .with_values(|config| {
            config.survivor_selector = survivor_selector.into();
            config.offspring_selector = offspring_selector.into();
        });

    Ok(unsafe {
        std::mem::transmute(match objective {
            Objective::Single(opt) => match opt {
                Optimize::Minimize => builder.minimizing(),
                Optimize::Maximize => builder.maximizing(),
            },
            _ => builder,
        })
    })
}

fn run_single_objective_engine<C, T>(
    engine: &mut Option<GeneticEngine<C, T, Generation<C, T>>>,
    limits: Vec<PyLimit>,
    log: bool,
) -> PyResult<PyGeneration>
where
    C: Chromosome + Clone,
    T: Debug + Clone + Send + Sync + 'static,
    Generation<C, T>: Into<PyGeneration>,
{
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
                    limits.iter().all(|limit| match limit {
                        PyLimit::Generation(lim) => epoch.index() < *lim,
                        PyLimit::Score(lim) => match epoch.objective() {
                            Objective::Single(opt) => match opt {
                                Optimize::Minimize => epoch.score().as_f32() > *lim,
                                Optimize::Maximize => epoch.score().as_f32() < *lim,
                            },
                            Objective::Multi(_) => false,
                        },
                        PyLimit::Seconds(val) => return epoch.seconds() < *val,
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

fn run_multi_objective_engine<C, T>(
    engine: &mut Option<GeneticEngine<C, T, MultiObjectiveGeneration<C>>>,
    limits: Vec<PyLimit>,
    _: bool,
) -> PyResult<PyGeneration>
where
    C: Chromosome + Clone,
    T: Debug + Clone + Send + Sync + 'static,
    MultiObjectiveGeneration<C>: Into<PyGeneration>,
{
    engine
        .take()
        .map(|engine| {
            engine
                .iter()
                .skip_while(|epoch| {
                    limits.iter().all(|limit| match limit {
                        PyLimit::Generation(lim) => epoch.index() < *lim,
                        PyLimit::Score(_) => false,
                        PyLimit::Seconds(val) => return epoch.seconds() < *val,
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
