use crate::ObjectValue;
use crate::PyMetricSet;
use crate::conversion;
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::MultiObjectiveGeneration;
use radiate::{BitChromosome, CharChromosome, Epoch, FloatChromosome, Generation, IntChromosome};

#[pyclass(unsendable, name = "Generation")]
pub struct PyGeneration {
    pub index: usize,
    pub score: Py<PyList>,
    pub value: Py<PyAny>,
    pub metrics: PyMetricSet,
    pub pareto_front: Option<Py<PyList>>,
}

#[pymethods]
impl PyGeneration {
    #[new]
    #[pyo3(signature = (index, score, value, metrics, pareto_front=None))]
    pub fn new(
        index: usize,
        score: Py<PyList>,
        value: Py<PyAny>,
        metrics: PyMetricSet,
        pareto_front: Option<Py<PyList>>,
    ) -> Self {
        Self {
            index,
            score,
            value,
            metrics,
            pareto_front,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.score.as_any().into_bound_py_any(py)
    }

    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.value.as_any().into_bound_py_any(py)
    }

    pub fn get_metric<'py>(
        &self,
        py: Python<'py>,
        name: String,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        self.metrics
            .get_metric(name)
            .map(|metric| metric.into_bound_py_any(py))
            .transpose()
    }

    pub fn get_pareto_front<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if let Some(pareto_front) = &self.pareto_front {
            let temp = pareto_front.as_any().bind(py).to_owned();
            Ok(Some(temp))
        } else {
            Ok(None)
        }
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py)?;
        let value = self.value(py)?;

        Ok(format!(
            "Generation(\n\tindex={},\n\tscore={},\n\tvalue={},\n\t metrics={})",
            self.index,
            score,
            value,
            self.metrics.__repr__()
        ))
    }
}

#[pyclass(unsendable, name = "MultiObjectiveGeneration")]
pub struct PyMultiObjectiveGeneration {
    pub index: usize,
    pub score: Py<PyList>,
    pub metrics: PyMetricSet,
}

impl Into<PyGeneration> for Generation<FloatChromosome, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}

impl Into<PyGeneration> for Generation<IntChromosome<i32>, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}
impl Into<PyGeneration> for Generation<CharChromosome, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}
impl Into<PyGeneration> for Generation<BitChromosome, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
                pareto_front: None,
            }
        })
    }
}

impl Into<PyGeneration> for MultiObjectiveGeneration<IntChromosome<i32>> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);
            let pareto_front = self
                .value()
                .values()
                .iter()
                .map(|phenotype| (*(*phenotype)).clone())
                .collect::<Vec<_>>();

            let pareto_front = conversion::pareto_front_to_py_object(py, &pareto_front)
                .expect("Failed to convert pareto front to PyObject");

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: py.None(),
                metrics: self.metrics().clone().into(),
                pareto_front: Some(pareto_front.unbind()),
            }
        })
    }
}
impl Into<PyGeneration> for MultiObjectiveGeneration<FloatChromosome> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);
            let pareto_front = self
                .value()
                .values()
                .iter()
                .map(|phenotype| (*(*phenotype)).clone())
                .collect::<Vec<_>>();

            let pareto_front = conversion::pareto_front_to_py_object(py, &pareto_front)
                .expect("Failed to convert pareto front to PyObject");

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: py.None(),
                metrics: self.metrics().clone().into(),
                pareto_front: Some(pareto_front.unbind()),
            }
        })
    }
}
