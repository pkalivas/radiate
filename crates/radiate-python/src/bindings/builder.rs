use super::{PyAlterer, PyDiversity, PyEngine, PyObjective, PySelector, subscriber::PySubscriber};
use crate::{
    PyBitCodec, PyCharCodec, PyFloatCodec, PyGeneType, PyIntCodec, PyLimit, conversion::Wrap,
};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyErr, PyObject, PyResult, Python, pyclass, pymethods,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyString, PyTuple},
};
use std::vec;

#[pyclass]
pub struct PyEngineBuilder {
    fitness_func: PyObject,
    codec: PyObject,
    params: Py<PyDict>,
}

#[pymethods]
impl PyEngineBuilder {
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
            "Engine(
                gene_type={:?},
                population_size={},
                offspring_fraction={},
                alters={},
                survivor_selector={},
                offspring_selector={},
                objective={},
                front_range={},
                diversity={},
                limits={},
                num_threads={},
                max_phenotype_age={},
                species_threshold={},
                max_species_age={}
            )",
            self.get_gene_type(py)?,
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
                .join(", "),
            self.get_diversity(py)?
                .map(|d| d.name().to_string())
                .unwrap_or_else(|| "None".to_string()),
            self.get_limits(py)?
                .iter()
                .map(|l| format!("{:?}", l))
                .collect::<Vec<_>>()
                .join(", "),
            self.get_num_threads(py)?,
            self.get_max_phenotype_age(py)?,
            self.get_species_threshold(py)?,
            self.get_max_species_age(py)?.to_string()
        );

        PyString::new(py, &repr).into_bound_py_any(py)
    }

    pub fn __dict__<'py>(&self, py: Python<'py>) -> PyResult<Py<PyDict>> {
        self.create_param_dict(py)
    }

    pub fn build<'py>(&self, py: Python<'py>) -> PyResult<PyEngine> {
        let param_dict = self.create_param_dict(py)?;
        let limits = self.get_limits(py)?;
        PyEngine::new(py, limits, param_dict)
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

    pub fn set_survivor_selector<'py>(
        &self,
        py: Python<'py>,
        selector: Py<PySelector>,
    ) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("survivor_selector", selector)
            .map_err(|e| e.into())
    }

    pub fn set_subscribers<'py>(
        &self,
        py: Python<'py>,
        subscribers: Vec<PySubscriber>,
    ) -> PyResult<()> {
        let mut current_subscribers = self.get_subscribers(py)?;
        current_subscribers.extend(subscribers);
        self.params
            .bind(py)
            .set_item("subscribers", PyList::new(py, current_subscribers)?)
            .map_err(|e| e.into())
    }

    pub fn set_offspring_selector<'py>(
        &self,
        py: Python<'py>,
        selector: Py<PySelector>,
    ) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("offspring_selector", selector)
            .map_err(|e| e.into())
    }

    pub fn set_diversity<'py>(&self, py: Python<'py>, diversity: Py<PyDiversity>) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("diversity", diversity)
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

    pub fn set_limits<'py>(&self, py: Python<'py>, limits: Py<PyAny>) -> PyResult<()> {
        self.params
            .bind(py)
            .set_item("limits", limits)
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

    pub fn get_diversity<'py>(&self, py: Python<'py>) -> PyResult<Option<PyDiversity>> {
        self.params
            .bind(py)
            .get_item("diversity")?
            .map(|v| v.extract::<Option<PyDiversity>>())
            .unwrap_or(Ok(None))
    }

    pub fn get_subscribers<'py>(&self, py: Python<'py>) -> PyResult<Vec<PySubscriber>> {
        self.params
            .bind(py)
            .get_item("subscribers")?
            .map(|v| v.extract::<Vec<PySubscriber>>())
            .unwrap_or(Ok(vec![]))
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

    pub fn get_limits<'py>(&self, py: Python<'py>) -> PyResult<Vec<PyLimit>> {
        self.params
            .bind(py)
            .get_item("limits")?
            .map(|v| v.extract::<Wrap<Vec<PyLimit>>>())
            .unwrap_or(Ok(Wrap(vec![])))
            .map(|wrap| wrap.0)
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
        dict.set_item("limits", self.get_limits(py)?)?;
        dict.set_item("diversity", self.get_diversity(py)?)?;
        dict.set_item("subscribers", self.get_subscribers(py)?)?;

        Ok(dict.into())
    }
}
