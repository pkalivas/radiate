// use super::{PyAlterer, PyDiversity, PyEngine, PyObjective, PySelector, subscriber::PySubscriber};
// use crate::{
//     PyBitCodec, PyCharCodec, PyExecutor, PyFloatCodec, PyGeneType, PyGraphCodec, PyIntCodec,
//     PyLimit, PyProblemBuilder, conversion::Wrap,
// };
// use pyo3::{
//     Bound, IntoPyObjectExt, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods,
//     types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyString, PyTuple},
// };
// use std::vec;

// pub(crate) const POPULATION_SIZE: &'static str = "population_size";
// pub(crate) const OFFSPRING_FRACTION: &'static str = "offspring_fraction";
// pub(crate) const ALTERS: &'static str = "alters";
// pub(crate) const SURVIVOR_SELECTOR: &'static str = "survivor_selector";
// pub(crate) const OFFSPRING_SELECTOR: &'static str = "offspring_selector";
// pub(crate) const SUBSCRIBERS: &'static str = "subscribers";
// pub(crate) const DIVERSITY: &'static str = "diversity";
// pub(crate) const OBJECTIVE: &'static str = "objective";
// pub(crate) const SPECIES_THRESHOLD: &'static str = "species_threshold";
// pub(crate) const MAX_PHENOTYPE_AGE: &'static str = "max_phenotype_age";
// pub(crate) const FRONT_RANGE: &'static str = "front_range";
// pub(crate) const LIMITS: &'static str = "limits";
// pub(crate) const MAX_SPECIES_AGE: &'static str = "max_species_age";
// pub(crate) const GENE_TYPE: &'static str = "gene_type";
// pub(crate) const CODEC: &'static str = "codec";
// pub(crate) const EXECUTOR: &'static str = "executor";
// pub(crate) const PROBLEM: &'static str = "problem";

// #[pyclass]
// pub struct PyEngineBuilder {
//     params: Py<PyDict>,
// }

// #[pymethods]
// impl PyEngineBuilder {
//     #[new]
//     #[pyo3(signature = (**kwds))]
//     pub fn new<'py>(py: Python<'py>, kwds: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
//         let params = kwds.map(|d| d.to_owned()).unwrap_or(PyDict::new(py));

//         Ok(Self {
//             params: params.into(),
//         })
//     }

//     pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         let repr = format!(
//             "Engine(
//                 gene_type={:?},
//                 population_size={},
//                 offspring_fraction={},
//                 alters={},
//                 survivor_selector={},
//                 offspring_selector={},
//                 objective={},
//                 front_range={},
//                 diversity={},
//                 limits={},
//                 executor={},
//                 max_phenotype_age={},
//                 species_threshold={},
//                 max_species_age={}
//             )",
//             self.get_gene_type(py)?,
//             self.get_population_size(py)?,
//             self.get_offspring_fraction(py)?,
//             self.get_alters(py)?
//                 .iter()
//                 .map(|a| a.name())
//                 .collect::<Vec<_>>()
//                 .join(", "),
//             self.get_survivor_selector(py)?.name(),
//             self.get_offspring_selector(py)?.name(),
//             self.get_objective(py)?.optimize().join(", "),
//             self.get_front_range(py)?
//                 .bind_borrowed(py)
//                 .try_iter()
//                 .iter()
//                 .take(2)
//                 .map(|v| v.extract::<usize>().unwrap_or(0))
//                 .collect::<Vec<_>>()
//                 .iter()
//                 .map(|v| v.to_string())
//                 .collect::<Vec<_>>()
//                 .join(", "),
//             self.get_diversity(py)?
//                 .map(|d| d.name().to_string())
//                 .unwrap_or_else(|| "None".to_string()),
//             self.get_limits(py)?
//                 .iter()
//                 .map(|l| format!("{:?}", l))
//                 .collect::<Vec<_>>()
//                 .join(", "),
//             self.get_executor(py)?,
//             self.get_max_phenotype_age(py)?,
//             self.get_species_threshold(py)?,
//             self.get_max_species_age(py)?.to_string()
//         );

//         PyString::new(py, &repr).into_bound_py_any(py)
//     }

//     pub fn __dict__<'py>(&self, py: Python<'py>) -> PyResult<Py<PyDict>> {
//         self.create_param_dict(py)
//     }

//     pub fn build<'py>(&self, py: Python<'py>) -> PyResult<PyEngine> {
//         let param_dict = self.create_param_dict(py)?;
//         let limits = self.get_limits(py)?;
//         PyEngine::new(py, limits, param_dict)
//     }

//     #[pyo3(signature = (codec))]
//     pub fn set_codec<'py>(&self, py: Python<'py>, codec: Py<PyAny>) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(CODEC, codec)
//             .map_err(|e| e.into())
//     }

//     #[pyo3(signature = (size=100))]
//     pub fn set_population_size<'py>(&self, py: Python<'py>, size: usize) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(POPULATION_SIZE, size)
//             .map_err(|e| e.into())
//     }

//     #[pyo3(signature = (fraction=0.8))]
//     pub fn set_offspring_fraction<'py>(&self, py: Python<'py>, fraction: f32) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(OFFSPRING_FRACTION, fraction)
//             .map_err(|e| e.into())
//     }

//     pub fn set_alters<'py>(&self, py: Python<'py>, alters: Vec<PyAlterer>) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(ALTERS, PyList::new(py, alters)?)
//             .map_err(|e| e.into())
//     }

//     pub fn set_survivor_selector<'py>(
//         &self,
//         py: Python<'py>,
//         selector: Py<PySelector>,
//     ) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(SURVIVOR_SELECTOR, selector)
//             .map_err(|e| e.into())
//     }

//     pub fn set_offspring_selector<'py>(
//         &self,
//         py: Python<'py>,
//         selector: Py<PySelector>,
//     ) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(OFFSPRING_SELECTOR, selector)
//             .map_err(|e| e.into())
//     }

//     pub fn set_subscribers<'py>(
//         &self,
//         py: Python<'py>,
//         subscribers: Option<Vec<PySubscriber>>,
//     ) -> PyResult<()> {
//         if let Some(subscribers) = subscribers {
//             let mut current_subscribers = self.get_subscribers(py)?;
//             current_subscribers.extend(subscribers);
//             return self
//                 .params
//                 .bind(py)
//                 .set_item(SUBSCRIBERS, PyList::new(py, current_subscribers)?)
//                 .map_err(|e| e.into());
//         }

//         Ok(())
//     }

//     pub fn set_diversity<'py>(&self, py: Python<'py>, diversity: Py<PyDiversity>) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(DIVERSITY, diversity)
//             .map_err(|e| e.into())
//     }

//     pub fn set_objective<'py>(&self, py: Python<'py>, objective: Py<PyAny>) -> PyResult<()> {
//         let objective = objective.extract::<PyObjective>(py)?;
//         self.params
//             .bind(py)
//             .set_item(OBJECTIVE, objective)
//             .map_err(|e| e.into())
//     }

//     pub fn set_species_threshold<'py>(&self, py: Python<'py>, threshold: f32) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(SPECIES_THRESHOLD, threshold)
//             .map_err(|e| e.into())
//     }

//     pub fn set_max_phenotype_age<'py>(&self, py: Python<'py>, max_age: usize) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(MAX_PHENOTYPE_AGE, max_age)
//             .map_err(|e| e.into())
//     }

//     pub fn set_front_range<'py>(&self, py: Python<'py>, front_range: Py<PyAny>) -> PyResult<()> {
//         let front_range = front_range.extract::<Py<PyTuple>>(py)?;

//         if front_range.bind_borrowed(py).len()? != 2 {
//             return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
//                 "front_range must be a tuple of twqwdwo values (min, maxssss)",
//             ));
//         }

//         self.params
//             .bind(py)
//             .set_item(FRONT_RANGE, front_range)
//             .map_err(|e| e.into())
//     }

//     pub fn set_limits<'py>(&self, py: Python<'py>, limits: Py<PyAny>) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(LIMITS, limits)
//             .map_err(|e| e.into())
//     }

//     pub fn set_max_species_age<'py>(&self, py: Python<'py>, max_age: usize) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(MAX_SPECIES_AGE, max_age)
//             .map_err(|e| e.into())
//     }

//     pub fn set_executor<'py>(&self, py: Python<'py>, executor: PyExecutor) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(EXECUTOR, executor)
//             .map_err(|e| e.into())
//     }

//     pub fn set_problem<'py>(&self, py: Python<'py>, problem: Py<PyProblemBuilder>) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item(PROBLEM, problem)
//             .map_err(|e| e.into())
//     }

//     pub fn get_problem<'py>(&self, py: Python<'py>) -> PyResult<PyProblemBuilder> {
//         self.params
//             .bind(py)
//             .get_item(PROBLEM)?
//             .map(|v| v.extract::<PyProblemBuilder>())
//             .unwrap_or_else(|| {
//                 Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                     "Problem not set or invalid type",
//                 ))
//             })
//     }

//     pub fn get_executor<'py>(&self, py: Python<'py>) -> PyResult<PyExecutor> {
//         self.params
//             .bind(py)
//             .get_item(EXECUTOR)?
//             .map(|v| v.extract::<PyExecutor>())
//             .unwrap_or(Ok(PyExecutor::serial()))
//     }

//     fn get_population_size<'py>(&self, py: Python<'py>) -> PyResult<usize> {
//         self.params
//             .bind(py)
//             .get_item(POPULATION_SIZE)?
//             .map(|v| v.extract::<usize>())
//             .unwrap_or(Ok(100))
//     }

//     pub fn get_offspring_fraction<'py>(&self, py: Python<'py>) -> PyResult<f32> {
//         self.params
//             .bind(py)
//             .get_item(OFFSPRING_FRACTION)?
//             .map(|v| v.extract::<f32>())
//             .unwrap_or(Ok(0.8))
//     }

//     pub fn get_max_phenotype_age<'py>(&self, py: Python<'py>) -> PyResult<usize> {
//         self.params
//             .bind(py)
//             .get_item(MAX_PHENOTYPE_AGE)?
//             .map(|v| v.extract::<usize>())
//             .unwrap_or(Ok(20))
//     }

//     pub fn get_species_threshold<'py>(&self, py: Python<'py>) -> PyResult<f32> {
//         self.params
//             .bind(py)
//             .get_item(SPECIES_THRESHOLD)?
//             .map(|v| v.extract::<f32>())
//             .unwrap_or(Ok(0.1))
//     }

//     pub fn get_max_species_age<'py>(&self, py: Python<'py>) -> PyResult<usize> {
//         self.params
//             .bind(py)
//             .get_item(MAX_SPECIES_AGE)?
//             .map(|v| v.extract::<usize>())
//             .unwrap_or(Ok(10))
//     }

//     pub fn get_alters<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyAlterer>> {
//         Ok(self
//             .params
//             .bind(py)
//             .get_item(ALTERS)?
//             .map(|v| v.extract::<Vec<PyAlterer>>().ok())
//             .flatten()
//             .unwrap_or(vec![
//                 PyAlterer::uniform_crossover(py, Some(0.5))?,
//                 PyAlterer::uniform_mutator(py, Some(0.1))?,
//             ]))
//     }

//     pub fn get_survivor_selector<'py>(&self, py: Python<'py>) -> PyResult<PySelector> {
//         self.params
//             .bind(py)
//             .get_item(SURVIVOR_SELECTOR)?
//             .map(|v| v.extract::<PySelector>())
//             .unwrap_or(Ok(PySelector::tournament_selector(py, Some(2))?))
//     }

//     pub fn get_offspring_selector<'py>(&self, py: Python<'py>) -> PyResult<PySelector> {
//         self.params
//             .bind(py)
//             .get_item(OFFSPRING_SELECTOR)?
//             .map(|v| v.extract::<PySelector>())
//             .unwrap_or(Ok(PySelector::roulette_wheel_selector(py)?))
//     }

//     pub fn get_objective<'py>(&self, py: Python<'py>) -> PyResult<PyObjective> {
//         self.params
//             .bind(py)
//             .get_item(OBJECTIVE)?
//             .map(|v| v.extract::<PyObjective>())
//             .unwrap_or(Ok(PyObjective::min()?))
//     }

//     pub fn get_diversity<'py>(&self, py: Python<'py>) -> PyResult<Option<PyDiversity>> {
//         self.params
//             .bind(py)
//             .get_item(DIVERSITY)?
//             .map(|v| v.extract::<Option<PyDiversity>>())
//             .unwrap_or(Ok(None))
//     }

//     pub fn get_subscribers<'py>(&self, py: Python<'py>) -> PyResult<Vec<PySubscriber>> {
//         self.params
//             .bind(py)
//             .get_item(SUBSCRIBERS)?
//             .map(|v| v.extract::<Vec<PySubscriber>>())
//             .unwrap_or(Ok(vec![]))
//     }

//     pub fn get_front_range<'py>(&self, py: Python<'py>) -> PyResult<Py<PyTuple>> {
//         let range = self.params.bind(py).get_item(FRONT_RANGE)?;
//         if let Some(range) = range {
//             if let Ok(tuple) = range.extract::<Py<PyTuple>>() {
//                 return Ok(tuple);
//             }
//         }

//         let result = PyTuple::new(py, vec![800, 900])?.unbind();
//         Ok(result)
//     }

//     pub fn get_limits<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyLimit>> {
//         self.params
//             .bind(py)
//             .get_item(LIMITS)?
//             .map(|v| v.extract::<Wrap<Vec<PyLimit>>>())
//             .unwrap_or(Ok(Wrap(vec![])))
//             .map(|wrap| wrap.0)
//     }

//     pub fn get_codec<'py>(&self, py: Python<'py>) -> PyResult<Py<PyAny>> {
//         self.params
//             .bind(py)
//             .get_item(CODEC)?
//             .map(|v| v.extract::<Py<PyAny>>())
//             .unwrap_or_else(|| {
//                 Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                     "Codec not set or invalid type",
//                 ))
//             })
//     }

//     pub fn get_gene_type<'py>(&self, py: Python<'py>) -> PyResult<PyGeneType> {
//         let codec_obj = self.get_codec(py)?.into_bound_py_any(py)?;

//         if let Ok(_) = codec_obj.extract::<PyIntCodec>() {
//             Ok(PyGeneType::Int)
//         } else if let Ok(_) = codec_obj.extract::<PyFloatCodec>() {
//             Ok(PyGeneType::Float)
//         } else if let Ok(_) = codec_obj.extract::<PyBitCodec>() {
//             Ok(PyGeneType::Bit)
//         } else if let Ok(_) = codec_obj.extract::<PyCharCodec>() {
//             Ok(PyGeneType::Char)
//         } else if let Ok(_) = codec_obj.extract::<PyGraphCodec>() {
//             Ok(PyGeneType::Graph)
//         } else {
//             Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                 "Unsupported gene type",
//             ))
//         }
//     }

//     fn create_param_dict<'py>(&self, py: Python<'py>) -> PyResult<Py<PyDict>> {
//         let dict = PyDict::new(py);

//         dict.set_item(POPULATION_SIZE, self.get_population_size(py)?)?;
//         dict.set_item(OFFSPRING_FRACTION, self.get_offspring_fraction(py)?)?;
//         dict.set_item(ALTERS, self.get_alters(py)?)?;
//         dict.set_item(SURVIVOR_SELECTOR, self.get_survivor_selector(py)?)?;
//         dict.set_item(OFFSPRING_SELECTOR, self.get_offspring_selector(py)?)?;
//         dict.set_item(OBJECTIVE, self.get_objective(py)?)?;
//         dict.set_item(GENE_TYPE, self.get_gene_type(py)?)?;
//         dict.set_item(CODEC, self.get_codec(py)?)?;
//         dict.set_item(FRONT_RANGE, self.get_front_range(py)?)?;
//         dict.set_item(MAX_PHENOTYPE_AGE, self.get_max_phenotype_age(py)?)?;
//         dict.set_item(SPECIES_THRESHOLD, self.get_species_threshold(py)?)?;
//         dict.set_item(MAX_SPECIES_AGE, self.get_max_species_age(py)?)?;
//         dict.set_item(LIMITS, self.get_limits(py)?)?;
//         dict.set_item(DIVERSITY, self.get_diversity(py)?)?;
//         dict.set_item(SUBSCRIBERS, self.get_subscribers(py)?)?;
//         dict.set_item(EXECUTOR, self.get_executor(py)?)?;
//         dict.set_item(PROBLEM, self.get_problem(py)?)?;

//         Ok(dict.into())
//     }
// }
