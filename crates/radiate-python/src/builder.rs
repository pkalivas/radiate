// use crate::{ObjectValue, PyGeneType, PyIntCodec, PyProblem};
// use pyo3::{
//     Borrowed, Bound, FromPyObject, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyErr, PyObject,
//     PyResult, Python,
//     conversion::FromPyObjectBound,
//     pyclass, pymethods,
//     types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods, PyString},
// };
// use radiate::{GeneticEngine, GeneticEngineBuilder};
// use std::{borrow::Borrow, vec};

// #[pyclass(unsendable, name = "Alterer")]
// #[derive(Clone, Debug)]
// pub struct PyAlterer {
//     name: String,
//     args: ObjectValue,
//     allowed_genes: Vec<PyGeneType>,
// }

// #[pymethods]
// impl PyAlterer {
//     pub fn name(&self) -> &str {
//         &self.name
//     }

//     pub fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.args.inner.bind(py).into_bound_py_any(py)
//     }

//     pub fn allowed_genes(&self) -> PyResult<Vec<PyGeneType>> {
//         Ok(self.allowed_genes.clone())
//     }

//     pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.__repr__(py)
//     }

//     pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         let repr = format!(
//             "Operator(name={}, args={})",
//             self.name,
//             self.args.inner.bind(py)
//         );

//         PyString::new(py, &repr).into_bound_py_any(py)
//     }

//     ///
//     /// Crossovers
//     ///

//     #[staticmethod]
//     #[pyo3(signature = (rate=None, alpha=None))]
//     pub fn blend_crossover<'py>(
//         py: Python<'py>,
//         rate: Option<f32>,
//         alpha: Option<f32>,
//     ) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }
//         if let Some(a) = alpha {
//             args.set_item("alpha", a)?;
//         }

//         Ok(PyAlterer {
//             name: "blend_crossover".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Float],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None, alpha=None))]
//     pub fn intermediate_crossover<'py>(
//         py: Python<'py>,
//         rate: Option<f32>,
//         alpha: Option<f32>,
//     ) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }
//         if let Some(a) = alpha {
//             args.set_item("alpha", a)?;
//         }

//         Ok(PyAlterer {
//             name: "intermediate_crossover".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Float],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn mean_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "mean_crossover".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Float, PyGeneType::Int],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn multi_point_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "multi_point_crossover".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn shuffle_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "shuffle_crossover".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Bit, PyGeneType::Char],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None, contiguty=None))]
//     pub fn simulated_binary_crossover<'py>(
//         py: Python<'py>,
//         rate: Option<f32>,
//         contiguty: Option<f32>,
//     ) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }
//         if let Some(c) = contiguty {
//             args.set_item("contiguty", c)?;
//         }

//         Ok(PyAlterer {
//             name: "simulated_binary_crossover".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Float],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn uniform_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "uniform_crossover".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     ///
//     /// Mutators
//     ///

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn arithmetic_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "arithmetic_mutator".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Float],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn gaussian_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "gaussian_mutator".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Float],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn scramble_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "scramble_mutator".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Bit, PyGeneType::Char],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn swap_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "swap_mutator".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![PyGeneType::Bit, PyGeneType::Char],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (rate=None))]
//     pub fn uniform_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
//         let args = PyDict::new(py);
//         if let Some(r) = rate {
//             args.set_item("rate", r)?;
//         }

//         Ok(PyAlterer {
//             name: "uniform_mutator".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }
// }

// #[pyclass(unsendable, name = "Selector")]
// #[derive(Clone, Debug)]
// pub struct PySelector {
//     name: String,
//     args: ObjectValue,
//     allowed_genes: Vec<PyGeneType>,
// }

// #[pymethods]
// impl PySelector {
//     pub fn name(&self) -> &str {
//         &self.name
//     }

//     pub fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.args.inner.bind(py).into_bound_py_any(py)
//     }

//     pub fn allowed_genes(&self) -> PyResult<Vec<PyGeneType>> {
//         Ok(self.allowed_genes.clone())
//     }

//     pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.__repr__(py)
//     }

//     pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         let repr = format!(
//             "Selector(name={}, args={})",
//             self.name,
//             self.args.inner.bind(py)
//         );

//         PyString::new(py, &repr).into_bound_py_any(py)
//     }

//     #[staticmethod]
//     #[pyo3(signature = (tournament_size=None))]
//     pub fn tournament_selector<'py>(
//         py: Python<'py>,
//         tournament_size: Option<usize>,
//     ) -> PyResult<PySelector> {
//         let args = PyDict::new(py);
//         if let Some(size) = tournament_size {
//             args.set_item("tournament_size", size)?;
//         }

//         Ok(PySelector {
//             name: "tournament_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     pub fn roulette_wheel_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);

//         Ok(PySelector {
//             name: "roulette_wheel_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     pub fn rank_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);

//         Ok(PySelector {
//             name: "rank_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     pub fn steady_state_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);

//         Ok(PySelector {
//             name: "steady_state_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     pub fn stochastic_universal_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);

//         Ok(PySelector {
//             name: "stochastic_universal_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     #[pyo3(signature = (temp=None))]
//     pub fn boltzmann_selector<'py>(py: Python<'py>, temp: Option<f32>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);
//         if let Some(t) = temp {
//             args.set_item("temp", t)?;
//         }

//         Ok(PySelector {
//             name: "boltzmann_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     pub fn elite_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);

//         Ok(PySelector {
//             name: "elite_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     pub fn random_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);

//         Ok(PySelector {
//             name: "random_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }

//     #[staticmethod]
//     pub fn nsga2_selector<'py>(py: Python<'py>) -> PyResult<PySelector> {
//         let args = PyDict::new(py);

//         Ok(PySelector {
//             name: "nsga2_selector".into(),
//             args: ObjectValue { inner: args.into() },
//             allowed_genes: vec![
//                 PyGeneType::Float,
//                 PyGeneType::Int,
//                 PyGeneType::Bit,
//                 PyGeneType::Char,
//             ],
//         })
//     }
// }

// #[pyclass(unsendable, name = "Objective")]
// #[derive(Clone, Debug)]
// pub struct PyObjective {
//     optimize: Vec<String>,
// }

// #[pymethods]
// impl PyObjective {
//     pub fn optimize(&self) -> Vec<String> {
//         self.optimize.clone()
//     }

//     pub fn is_multi(&self) -> bool {
//         self.optimize.len() > 1
//     }

//     pub fn is_single(&self) -> bool {
//         self.optimize.len() == 1
//     }

//     pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.__repr__(py)
//     }

//     pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         let repr = format!("Objective(optimize={})", self.optimize.join(", "));
//         PyString::new(py, &repr).into_bound_py_any(py)
//     }

//     #[staticmethod]
//     pub fn max() -> PyResult<PyObjective> {
//         Ok(PyObjective {
//             optimize: vec!["max".to_string()],
//         })
//     }

//     #[staticmethod]
//     pub fn min() -> PyResult<PyObjective> {
//         Ok(PyObjective {
//             optimize: vec!["min".to_string()],
//         })
//     }

//     #[staticmethod]
//     pub fn multi(optimize: Vec<String>) -> PyResult<PyObjective> {
//         if optimize.is_empty() {
//             return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
//                 "At least one optimization direction must be specified",
//             ));
//         }

//         Ok(PyObjective { optimize })
//     }
// }

// #[pyclass(unsendable)]
// #[derive(Clone, Debug)]
// pub enum PyLimit {
//     Generation(usize),
//     Time(f64),
//     Score(f32),
// }

// #[pyclass]
// pub struct EngineBuilderTemp {
//     fitness_func: PyObject,
//     codec: PyObject,
//     params: Py<PyDict>,
// }

// #[pymethods]
// impl EngineBuilderTemp {
//     #[new]
//     #[pyo3(signature = (fitness_func, codec, **kwds))]
//     pub fn new<'py>(
//         py: Python<'py>,
//         fitness_func: PyObject,
//         codec: PyObject,
//         kwds: Option<&Bound<'_, PyDict>>,
//     ) -> PyResult<Self> {
//         let params = kwds.map(|d| d.to_owned()).unwrap_or(PyDict::new(py));

//         Ok(Self {
//             fitness_func,
//             codec,
//             params: params.into(),
//         })
//     }

//     pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         let repr = format!(
//             "EngineBuilderTemp(
//                 population_size={},
//                 offspring_fraction={},
//                 alters={},
//                 survivor_selector={},
//                 offspring_selector={},
//                 objective={})",
//             self.get_population_size(py)?,
//             self.get_offspring_fraction(py)?,
//             self.get_alters(py)?
//                 .iter()
//                 .map(|a| a.name())
//                 .collect::<Vec<_>>()
//                 .join(", "),
//             self.get_survivor_selector(py)?.name(),
//             self.get_offspring_selector(py)?.name(),
//             self.get_objective(py)?.optimize().join(", ")
//         );

//         PyString::new(py, &repr).into_bound_py_any(py)
//     }

//     pub fn run<'py>(&self, py: Python<'py>, limits: Py<PyAny>) -> PyResult<()> {
//         let limits = if let Ok(limits) = limits.extract::<Vec<PyLimit>>(py) {
//             limits
//         } else if let Ok(limits) = limits.extract::<Bound<'_, PyList>>(py) {
//             limits
//                 .iter()
//                 .map(|l| l.extract::<PyLimit>())
//                 .collect::<PyResult<Vec<_>>>()?
//         } else if let Ok(limit) = limits.extract::<PyLimit>(py) {
//             vec![limit]
//         } else {
//             return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                 "Expected a list of limits",
//             ));
//         };

//         if limits.is_empty() {
//             return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
//                 "At least one limit must be specified",
//             ));
//         }

//         let population_size = self.get_population_size(py)?;
//         let offspring_fraction = self.get_offspring_fraction(py)?;
//         let alters = self.get_alters(py)?;
//         let survivor_selector = self.get_survivor_selector(py)?;
//         let offspring_selector = self.get_offspring_selector(py)?;
//         let objective = self.get_objective(py)?;

//         if let Some(int) = self.codec.extract::<PyIntCodec>(py).ok() {
//             let builder = GeneticEngine::builder()
//                 .problem(PyProblem::new(self.fitness_func, int.codec.clone()))
//                 .population_size(population_size)
//                 .offspring_fraction(offspring_fraction)
//                 .alters(alters)
//                 .survivor_selector(survivor_selector)
//                 .offspring_selector(offspring_selector)
//                 .objective(objective);
//         }

//         panic!()

//         // let engine = PyEngine::new(
//         //     py,
//         //     self.codec.clone(),
//         //     self.fitness_func.clone(),
//         //     population_size,
//         //     offspring_fraction,
//         //     alters,
//         //     survivor_selector,
//         //     offspring_selector,
//         //     objective,
//         // )?;

//         // Ok(engine)
//     }

//     #[pyo3(signature = (size=100))]
//     pub fn population_size<'py>(&self, py: Python<'py>, size: usize) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item("population_size", size)
//             .map_err(|e| e.into())
//     }

//     #[pyo3(signature = (fraction=0.8))]
//     pub fn offspring_fraction<'py>(&self, py: Python<'py>, fraction: f32) -> PyResult<()> {
//         self.params
//             .bind(py)
//             .set_item("offspring_fraction", fraction)
//             .map_err(|e| e.into())
//     }

//     pub fn alters<'py>(&self, py: Python<'py>, alters: Py<PyAny>) -> PyResult<()> {
//         let new_alters = if let Ok(alters) = alters.extract::<Vec<PyAlterer>>(py) {
//             if alters.is_empty() {
//                 return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
//                     "At least one alterer must be specified",
//                 ));
//             }
//             alters
//         } else if let Ok(alters) = alters.extract::<Bound<'_, PyList>>(py) {
//             if alters.is_empty() {
//                 return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
//                     "At least one alterer must be specified",
//                 ));
//             }
//             alters
//                 .iter()
//                 .map(|a| a.extract::<PyAlterer>())
//                 .collect::<PyResult<Vec<_>>>()?
//         } else if let Ok(alters) = alters.extract::<PyAlterer>(py) {
//             vec![alters]
//         } else {
//             return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                 "Expected a list of Alterers",
//             ));
//         };

//         self.params
//             .bind(py)
//             .set_item("alters", PyList::new(py, new_alters)?)
//             .map_err(|e| e.into())
//     }

//     pub fn survivor_selector<'py>(&self, py: Python<'py>, selector: Py<PyAny>) -> PyResult<()> {
//         let selector = selector.extract::<PySelector>(py)?;
//         self.params
//             .bind(py)
//             .set_item("survivor_selector", selector)
//             .map_err(|e| e.into())
//     }

//     pub fn offspring_selector<'py>(&self, py: Python<'py>, selector: Py<PyAny>) -> PyResult<()> {
//         let selector = selector.extract::<PySelector>(py)?;
//         self.params
//             .bind(py)
//             .set_item("offspring_selector", selector)
//             .map_err(|e| e.into())
//     }

//     pub fn objective<'py>(&self, py: Python<'py>, objective: Py<PyAny>) -> PyResult<()> {
//         let objective = objective.extract::<PyObjective>(py)?;
//         self.params
//             .bind(py)
//             .set_item("objective", objective)
//             .map_err(|e| e.into())
//     }

//     fn get_population_size<'py>(&self, py: Python<'py>) -> PyResult<usize> {
//         self.params
//             .bind(py)
//             .get_item("population_size")?
//             .map(|v| v.extract::<usize>())
//             .unwrap_or(Ok(100))
//     }

//     pub fn get_offspring_fraction<'py>(&self, py: Python<'py>) -> PyResult<f32> {
//         self.params
//             .bind(py)
//             .get_item("offspring_fraction")?
//             .map(|v| v.extract::<f32>())
//             .unwrap_or(Ok(0.8))
//     }

//     pub fn get_alters<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyAlterer>> {
//         Ok(self
//             .params
//             .bind(py)
//             .get_item("alters")?
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
//             .get_item("survivor_selector")?
//             .map(|v| v.extract::<PySelector>())
//             .unwrap_or(Ok(PySelector::tournament_selector(py, Some(2))?))
//     }

//     pub fn get_offspring_selector<'py>(&self, py: Python<'py>) -> PyResult<PySelector> {
//         self.params
//             .bind(py)
//             .get_item("offspring_selector")?
//             .map(|v| v.extract::<PySelector>())
//             .unwrap_or(Ok(PySelector::roulette_wheel_selector(py)?))
//     }

//     pub fn get_objective<'py>(&self, py: Python<'py>) -> PyResult<PyObjective> {
//         self.params
//             .bind(py)
//             .get_item("objective")?
//             .map(|v| v.extract::<PyObjective>())
//             .unwrap_or(Ok(PyObjective::max()?))
//     }
// }

// struct EngineInputs {
//     fitness_func: PyObject,
//     codec: PyObject,
//     population_size: usize,
//     offspring_fraction: f32,
//     alters: Vec<PyAlterer>,
//     survivor_selector: PySelector,
//     offspring_selector: PySelector,
//     objective: PyObjective,
// }

// impl EngineInputs {
//     fn new(
//         py: Python<'_>,
//         fitness_func: PyObject,
//         codec: PyObject,
//         population_size: usize,
//         offspring_fraction: f32,
//         alters: Vec<PyAlterer>,
//         survivor_selector: PySelector,
//         offspring_selector: PySelector,
//         objective: PyObjective,
//     ) -> Self {
//         Self {
//             fitness_func,
//             codec,
//             population_size,
//             offspring_fraction,
//             alters,
//             survivor_selector,
//             offspring_selector,
//             objective,
//         }
//     }
// }
