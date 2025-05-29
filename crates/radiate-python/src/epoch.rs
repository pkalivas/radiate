use crate::PyMetricSet;
use crate::conversion::ObjectValue;
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{BitChromosome, CharChromosome, Epoch, FloatChromosome, Generation, IntChromosome};

#[pyclass(unsendable)]
pub struct PyGeneration {
    pub index: usize,
    pub score: Py<PyList>,
    pub value: Py<PyAny>,
    pub metrics: PyMetricSet,
}

#[pymethods]
impl PyGeneration {
    #[new]
    #[pyo3(signature = (index, score, value, metrics))]
    pub fn new(index: usize, score: Py<PyList>, value: Py<PyAny>, metrics: PyMetricSet) -> Self {
        Self {
            index,
            score,
            value,
            metrics,
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
            }
        })
    }
}
