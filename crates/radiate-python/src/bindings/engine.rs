// use super::{PyObjective, subscriber::PySubscriber};
// use crate::{
//     FreeThreadPyEvaluator, IntoPyObjectValue, ObjectValue, PyBitCodec, PyCharCodec, PyFloatCodec,
//     PyGeneType, PyGeneration, PyGraphCodec, PyIntCodec, PyLimit, PyProblem, PyProblemBuilder,
//     bindings::builder::*, conversion::Wrap, events::PyEventHandler,
// };
// use pyo3::{
//     Bound, FromPyObject, IntoPyObjectExt, Py, PyAny, PyErr, PyResult, Python,
//     exceptions::{PyRuntimeError, PyTypeError},
//     pyclass, pymethods,
//     types::{PyAnyMethods, PyDict, PyString, PyTuple},
// };
// use radiate::*;
// use std::{fmt::Debug, vec};

// #[pyclass]
// pub struct PyEngine {
//     params: Py<PyDict>,
//     limits: Vec<PyLimit>,
//     inner: Option<EngineInner>,
// }

// #[pymethods]
// impl PyEngine {
//     #[new]
//     pub fn new<'py>(py: Python<'py>, limits: Vec<PyLimit>, params: Py<PyDict>) -> PyResult<Self> {
//         let inner = params.extract::<Wrap<EngineInner>>(py)?;
//         Ok(PyEngine {
//             params,
//             limits,
//             inner: Some(inner.0),
//         })
//     }

//     pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         let repr = format!("Engine({})", self.params);
//         PyString::new(py, &repr).into_bound_py_any(py)
//     }

//     pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.__str__(py)
//     }

//     pub fn run<'py>(&mut self, py: Python<'py>, log: bool) -> PyResult<PyGeneration> {
//         if self.inner.is_none() {
//             let new_inner = self.params.extract::<Wrap<EngineInner>>(py)?;
//             self.inner = Some(new_inner.0);
//         }

//         let limits = self.limits.clone();
//         if let Some(inner) = self.inner.take() {
//             match inner {
//                 EngineInner::Int(engine) => {
//                     run_single_objective_engine(&mut Some(engine), limits, log)
//                 }
//                 EngineInner::Float(engine) => {
//                     run_single_objective_engine(&mut Some(engine), limits, log)
//                 }
//                 EngineInner::Char(engine) => {
//                     run_single_objective_engine(&mut Some(engine), limits, log)
//                 }
//                 EngineInner::Bit(engine) => {
//                     run_single_objective_engine(&mut Some(engine), limits, log)
//                 }
//                 EngineInner::IntMulti(engine) => {
//                     run_multi_objective_engine(&mut Some(engine), limits, log)
//                 }
//                 EngineInner::FloatMulti(engine) => {
//                     run_multi_objective_engine(&mut Some(engine), limits, log)
//                 }
//                 EngineInner::GraphRegression(engine) => {
//                     run_single_objective_engine(&mut Some(engine), limits, log)
//                 }
//             }
//         } else {
//             Err(PyErr::new::<PyRuntimeError, _>(
//                 "Engine has already been run",
//             ))
//         }
//     }
// }

// type SingleObjectiveEngine<C> = GeneticEngine<C, ObjectValue, Generation<C, ObjectValue>>;
// type MultiObjectiveEngine<C> = GeneticEngine<C, ObjectValue, ParetoGeneration<C>>;
// type RegressionEngine<C, T> = GeneticEngine<C, T, Generation<C, T>>;

// enum EngineInner {
//     Int(SingleObjectiveEngine<IntChromosome<i32>>),
//     Float(SingleObjectiveEngine<FloatChromosome>),
//     Char(SingleObjectiveEngine<CharChromosome>),
//     Bit(SingleObjectiveEngine<BitChromosome>),
//     IntMulti(MultiObjectiveEngine<IntChromosome<i32>>),
//     FloatMulti(MultiObjectiveEngine<FloatChromosome>),
//     GraphRegression(RegressionEngine<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
// }

// impl<'py> FromPyObject<'py> for Wrap<EngineInner> {
//     fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
//         let params = ob.extract::<Py<PyAny>>()?;

//         let gene_type = params
//             .bind(ob.py())
//             .get_item(GENE_TYPE)?
//             .extract::<PyGeneType>()?;

//         let codec_obj = params.bind(ob.py()).get_item(CODEC)?;
//         let objective = params
//             .bind(ob.py())
//             .get_item(OBJECTIVE)?
//             .extract::<PyObjective>()?;

//         let problem = params
//             .bind(ob.py())
//             .get_item(PROBLEM)?
//             .extract::<PyProblemBuilder>()?;

//         let engine = if problem.name() == "Custom" {
//             let fitness_fn = problem
//                 .args(ob.py())
//                 .and_then(|args| args.get_item("fitness_func"))
//                 .and_then(|f| f.extract::<Py<PyAny>>())?;

//             match gene_type {
//                 PyGeneType::Int => {
//                     if let Ok(codec) = codec_obj.extract::<PyIntCodec>() {
//                         let int_problem = PyProblem::new(fitness_fn, codec.codec);
//                         match objective.is_multi() {
//                             true => Ok(EngineInner::IntMulti(
//                                 create_multi_engine(ob.py(), int_problem, &params)?.build(),
//                             )),
//                             false => Ok(EngineInner::Int(
//                                 create_engine(ob.py(), int_problem, &params)?.build(),
//                             )),
//                         }
//                     } else {
//                         return Err(PyErr::new::<PyTypeError, _>(
//                             "Expected an IntCodec for IntChromosome",
//                         ));
//                     }
//                 }
//                 PyGeneType::Float => {
//                     if let Ok(codec) = codec_obj.extract::<PyFloatCodec>() {
//                         let float_problem = PyProblem::new(fitness_fn, codec.codec);
//                         match objective.is_multi() {
//                             true => Ok(EngineInner::FloatMulti(
//                                 create_multi_engine(ob.py(), float_problem, &params)?.build(),
//                             )),
//                             false => Ok(EngineInner::Float(
//                                 create_engine(ob.py(), float_problem, &params)?.build(),
//                             )),
//                         }
//                     } else {
//                         return Err(PyErr::new::<PyTypeError, _>(
//                             "Expected a FloatCodec for FloatChromosome",
//                         ));
//                     }
//                 }
//                 PyGeneType::Char => {
//                     if let Ok(codec) = codec_obj.extract::<PyCharCodec>() {
//                         let char_problem = PyProblem::new(fitness_fn, codec.codec);
//                         Ok(EngineInner::Char(
//                             create_engine(ob.py(), char_problem, &params)?.build(),
//                         ))
//                     } else {
//                         return Err(PyErr::new::<PyTypeError, _>(
//                             "Expected a CharCodec for CharChromosome",
//                         ));
//                     }
//                 }
//                 PyGeneType::Bit => {
//                     if let Ok(codec) = codec_obj.extract::<PyBitCodec>() {
//                         let bit_problem = PyProblem::new(fitness_fn, codec.codec);
//                         Ok(EngineInner::Bit(
//                             create_engine(ob.py(), bit_problem, &params)?.build(),
//                         ))
//                     } else {
//                         return Err(PyErr::new::<PyTypeError, _>(
//                             "Expected a BitCodec for BitChromosome",
//                         ));
//                     }
//                 }
//                 PyGeneType::Graph => {
//                     if let Ok(_) = codec_obj.extract::<PyGraphCodec>() {
//                         return Err(PyErr::new::<PyTypeError, _>(
//                             "GraphChromosome only supports regression problems",
//                         ));
//                     } else {
//                         return Err(PyErr::new::<PyTypeError, _>(
//                             "Expected a GraphCodec for GraphChromosome",
//                         ));
//                     }
//                 }
//                 _ => {
//                     return Err(PyErr::new::<PyTypeError, _>(
//                         "Unsupported gene type for Custom problem",
//                     ));
//                 }
//             }
//         } else if problem.name() == "Regression" {
//             let features = problem
//                 .args(ob.py())
//                 .and_then(|args| args.get_item("features"))
//                 .and_then(|f| f.extract::<Vec<Vec<f32>>>())?;
//             let targets = problem
//                 .args(ob.py())
//                 .and_then(|args| args.get_item("targets"))
//                 .and_then(|t| t.extract::<Vec<Vec<f32>>>())?;

//             let data_set = DataSet::new(features, targets);

//             match gene_type {
//                 PyGeneType::Graph => {
//                     if let Ok(codec) = codec_obj.extract::<PyGraphCodec>() {
//                         let loss_str = problem
//                             .args(ob.py())
//                             .and_then(|args| args.get_item("loss"))
//                             .and_then(|l| l.extract::<String>())
//                             .map(|s| s.to_uppercase())
//                             .unwrap_or("MSE".into());
//                         let loss = match loss_str.as_str() {
//                             "MSE" => Loss::MSE,
//                             "MAE" => Loss::MAE,
//                             _ => {
//                                 return Err(PyErr::new::<PyTypeError, _>(
//                                     "Unsupported loss function for regression",
//                                 ));
//                             }
//                         };

//                         let regression = Regression::new(data_set, loss, codec.codec);
//                         Ok(EngineInner::GraphRegression(
//                             create_regression_engine(ob.py(), regression, &params)?.build(),
//                         ))
//                     } else {
//                         return Err(PyErr::new::<PyTypeError, _>(
//                             "Expected a GraphCodec for GraphChromosome",
//                         ));
//                     }
//                 }
//                 _ => Err(PyErr::new::<PyTypeError, _>(
//                     "Regression problem only supports GraphChromosome",
//                 )),
//             }
//         } else {
//             return Err(PyErr::new::<PyTypeError, _>(
//                 "Unsupported problem type. Only 'DefaultProblem' and 'Regression' are supported",
//             ));
//         };

//         engine.map(Wrap)
//     }
// }

// fn create_multi_engine<C>(
//     py: Python<'_>,
//     problem: PyProblem<C>,
//     parameters: &Py<PyAny>,
// ) -> PyResult<GeneticEngineBuilder<C, ObjectValue, ParetoGeneration<C>>>
// where
//     C: Chromosome + Clone + PartialEq + 'static,
// {
//     let params = parameters.bind(py);

//     let population_size = params.get_item(POPULATION_SIZE)?.extract::<usize>()?;
//     let offspring_fraction = params.get_item(OFFSPRING_FRACTION)?.extract::<f32>()?;
//     let objective = params.get_item(OBJECTIVE)?.extract::<Wrap<Objective>>()?.0;
//     let front_range = params.get_item(FRONT_RANGE)?.extract::<Py<PyTuple>>()?;
//     let max_age = params.get_item(MAX_PHENOTYPE_AGE)?.extract::<usize>()?;
//     let max_species_age = params.get_item(MAX_SPECIES_AGE)?.extract::<usize>()?;
//     let species_threshold = params.get_item(SPECIES_THRESHOLD)?.extract::<f32>()?;
//     let first_front = front_range.bind(py).get_item(0)?.extract::<usize>()?;
//     let second_front = front_range.bind(py).get_item(1)?.extract::<usize>()?;

//     let alters = params
//         .get_item(ALTERS)?
//         .into_bound_py_any(py)?
//         .extract::<Wrap<Vec<Box<dyn Alter<C>>>>>()?
//         .0;

//     let survivor_selector = params
//         .get_item(SURVIVOR_SELECTOR)?
//         .extract::<Wrap<Box<dyn Select<C>>>>()?
//         .0;

//     let offspring_selector = params
//         .get_item(OFFSPRING_SELECTOR)?
//         .extract::<Wrap<Box<dyn Select<C>>>>()?
//         .0;

//     let diversity = params
//         .get_item(DIVERSITY)?
//         .extract::<Wrap<Option<Box<dyn Diversity<C>>>>>()?
//         .0;

//     let subscribers = params
//         .get_item(SUBSCRIBERS)?
//         .into_bound_py_any(py)?
//         .extract::<Vec<PySubscriber>>()?;

//     let executor = params.get_item(EXECUTOR)?.extract::<Wrap<Executor>>()?.0;

//     let mut builder = GeneticEngine::builder()
//         .problem(problem.clone())
//         .population_size(population_size)
//         .offspring_fraction(offspring_fraction)
//         .max_age(max_age)
//         .max_species_age(max_species_age)
//         .species_threshold(species_threshold)
//         .boxed_diversity(diversity)
//         .boxed_offspring_selector(offspring_selector)
//         .boxed_survivor_selector(survivor_selector)
//         .alter(alters)
//         .multi_objective(match objective {
//             Objective::Multi(opts) => opts,
//             _ => vec![Optimize::Minimize],
//         })
//         .front_size(first_front..second_front)
//         .executor(executor.clone())
//         .evaluator(FreeThreadPyEvaluator::new(executor, problem))
//         .bus_executor(Executor::default());

//     builder = match subscribers.len() > 0 {
//         true => builder.subscribe(PyEventHandler::new(subscribers)),
//         false => builder,
//     };

//     Ok(unsafe { std::mem::transmute(builder) })
// }

// fn create_engine<C>(
//     py: Python<'_>,
//     problem: PyProblem<C>,
//     parameters: &Py<PyAny>,
// ) -> PyResult<GeneticEngineBuilder<C, ObjectValue, Generation<C, ObjectValue>>>
// where
//     C: Chromosome + Clone + PartialEq + 'static,
// {
//     let params = parameters.bind(py);

//     let population_size = params.get_item(POPULATION_SIZE)?.extract::<usize>()?;
//     let offspring_fraction = params.get_item(OFFSPRING_FRACTION)?.extract::<f32>()?;
//     let max_age = params.get_item(MAX_PHENOTYPE_AGE)?.extract::<usize>()?;
//     let max_species_age = params.get_item(MAX_SPECIES_AGE)?.extract::<usize>()?;
//     let species_threshold = params.get_item(SPECIES_THRESHOLD)?.extract::<f32>()?;

//     let alters = params
//         .get_item(ALTERS)?
//         .into_bound_py_any(py)?
//         .extract::<Wrap<Vec<Box<dyn Alter<C>>>>>()?
//         .0;

//     let survivor_selector = params
//         .get_item(SURVIVOR_SELECTOR)?
//         .extract::<Wrap<Box<dyn Select<C>>>>()?
//         .0;

//     let offspring_selector = params
//         .get_item(OFFSPRING_SELECTOR)?
//         .extract::<Wrap<Box<dyn Select<C>>>>()?
//         .0;

//     let objective = params.get_item(OBJECTIVE)?.extract::<Wrap<Objective>>()?.0;

//     let diversity = params
//         .get_item(DIVERSITY)?
//         .extract::<Wrap<Option<Box<dyn Diversity<C>>>>>()?
//         .0;

//     let subscribers = params
//         .get_item(SUBSCRIBERS)?
//         .into_bound_py_any(py)?
//         .extract::<Vec<PySubscriber>>()?;

//     let executor = params.get_item(EXECUTOR)?.extract::<Wrap<Executor>>()?.0;

//     let mut builder = GeneticEngine::builder()
//         .problem(problem.clone())
//         .population_size(population_size)
//         .offspring_fraction(offspring_fraction)
//         .max_age(max_age)
//         .max_species_age(max_species_age)
//         .species_threshold(species_threshold)
//         .alter(alters)
//         .executor(executor.clone())
//         .evaluator(FreeThreadPyEvaluator::new(executor.clone(), problem))
//         .bus_executor(Executor::default())
//         .boxed_diversity(diversity)
//         .boxed_offspring_selector(offspring_selector)
//         .boxed_survivor_selector(survivor_selector);

//     builder = match subscribers.len() > 0 {
//         true => builder.subscribe(PyEventHandler::new(subscribers)),
//         false => builder,
//     };

//     Ok(unsafe {
//         std::mem::transmute(match objective {
//             Objective::Single(opt) => match opt {
//                 Optimize::Minimize => builder.minimizing(),
//                 Optimize::Maximize => builder.maximizing(),
//             },
//             _ => builder,
//         })
//     })
// }

// fn create_regression_engine<C, T>(
//     py: Python<'_>,
//     regression: impl Problem<C, T> + 'static,
//     parameters: &Py<PyAny>,
// ) -> PyResult<GeneticEngineBuilder<C, T, Generation<C, T>>>
// where
//     C: Chromosome + Clone + PartialEq + 'static,
//     T: IntoPyObjectValue + Clone + Send + Sync + 'static,
// {
//     let params = parameters.bind(py);

//     let population_size = params.get_item(POPULATION_SIZE)?.extract::<usize>()?;
//     let offspring_fraction = params.get_item(OFFSPRING_FRACTION)?.extract::<f32>()?;
//     let max_age = params.get_item(MAX_PHENOTYPE_AGE)?.extract::<usize>()?;
//     let max_species_age = params.get_item(MAX_SPECIES_AGE)?.extract::<usize>()?;
//     let species_threshold = params.get_item(SPECIES_THRESHOLD)?.extract::<f32>()?;

//     let alters = params
//         .get_item(ALTERS)?
//         .into_bound_py_any(py)?
//         .extract::<Wrap<Vec<Box<dyn Alter<C>>>>>()?
//         .0;

//     let survivor_selector = params
//         .get_item(SURVIVOR_SELECTOR)?
//         .extract::<Wrap<Box<dyn Select<C>>>>()?
//         .0;

//     let offspring_selector = params
//         .get_item(OFFSPRING_SELECTOR)?
//         .extract::<Wrap<Box<dyn Select<C>>>>()?
//         .0;

//     let objective = params.get_item(OBJECTIVE)?.extract::<Wrap<Objective>>()?.0;

//     let diversity = params
//         .get_item(DIVERSITY)?
//         .extract::<Wrap<Option<Box<dyn Diversity<C>>>>>()?
//         .0;

//     let subscribers = params
//         .get_item(SUBSCRIBERS)?
//         .into_bound_py_any(py)?
//         .extract::<Vec<PySubscriber>>()?;

//     let executor = params.get_item(EXECUTOR)?.extract::<Wrap<Executor>>()?.0;

//     let mut builder = GeneticEngine::builder()
//         .problem(regression)
//         .population_size(population_size)
//         .offspring_fraction(offspring_fraction)
//         .max_age(max_age)
//         .max_species_age(max_species_age)
//         .species_threshold(species_threshold)
//         .alter(alters)
//         .executor(executor.clone())
//         .bus_executor(Executor::default())
//         .boxed_diversity(diversity)
//         .boxed_offspring_selector(offspring_selector)
//         .boxed_survivor_selector(survivor_selector);

//     builder = match subscribers.len() > 0 {
//         true => builder.subscribe(PyEventHandler::new(subscribers)),
//         false => builder,
//     };

//     Ok(unsafe {
//         std::mem::transmute(match objective {
//             Objective::Single(opt) => match opt {
//                 Optimize::Minimize => builder.minimizing(),
//                 Optimize::Maximize => builder.maximizing(),
//             },
//             _ => builder,
//         })
//     })
// }

// fn run_single_objective_engine<C, T>(
//     engine: &mut Option<GeneticEngine<C, T, Generation<C, T>>>,
//     limits: Vec<PyLimit>,
//     log: bool,
// ) -> PyResult<PyGeneration>
// where
//     C: Chromosome + Clone,
//     T: Debug + Clone + Send + Sync + 'static,
//     Generation<C, T>: Into<PyGeneration>,
// {
//     engine
//         .take()
//         .map(|engine| {
//             engine
//                 .iter()
//                 .inspect(|epoch| {
//                     if log {
//                         log_ctx!(epoch);
//                     }
//                 })
//                 .skip_while(|epoch| {
//                     limits.iter().all(|limit| match limit {
//                         PyLimit::Generation(lim) => epoch.index() < *lim,
//                         PyLimit::Score(lim) => match epoch.objective() {
//                             Objective::Single(opt) => match opt {
//                                 Optimize::Minimize => epoch.score().as_f32() > *lim,
//                                 Optimize::Maximize => epoch.score().as_f32() < *lim,
//                             },
//                             Objective::Multi(_) => false,
//                         },
//                         PyLimit::Seconds(val) => return epoch.seconds() < *val,
//                     })
//                 })
//                 .take(1)
//                 .last()
//                 .inspect(|epoch| {
//                     if log {
//                         println!("{:?}", epoch);
//                     }
//                 })
//                 .map(|epoch| epoch.into())
//         })
//         .flatten()
//         .ok_or(PyRuntimeError::new_err(
//             "No generation found that meets the limits",
//         ))
// }

// fn run_multi_objective_engine<C, T>(
//     engine: &mut Option<GeneticEngine<C, T, ParetoGeneration<C>>>,
//     limits: Vec<PyLimit>,
//     _: bool,
// ) -> PyResult<PyGeneration>
// where
//     C: Chromosome + Clone,
//     T: Debug + Clone + Send + Sync + 'static,
//     ParetoGeneration<C>: Into<PyGeneration>,
// {
//     engine
//         .take()
//         .map(|engine| {
//             engine
//                 .iter()
//                 .skip_while(|epoch| {
//                     limits.iter().all(|limit| match limit {
//                         PyLimit::Generation(lim) => epoch.index() < *lim,
//                         PyLimit::Score(_) => false,
//                         PyLimit::Seconds(val) => return epoch.seconds() < *val,
//                     })
//                 })
//                 .take(1)
//                 .last()
//                 .map(|epoch| epoch.into())
//         })
//         .flatten()
//         .ok_or(PyRuntimeError::new_err(
//             "No generation found that meets the limits",
//         ))
// }
