use super::PyGenotype;
use super::PyPopulation;
use crate::ObjectValue;
use crate::conversion::Wrap;
use pyo3::IntoPyObject;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyDict;
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::prelude::*;

const SINGLE_OBJECTIVE_GENERATION: &str = "Generation";
const MULTI_OBJECTIVE_GENERATION: &str = "MultiObjectiveGeneration";

#[pyclass(unsendable)]
pub struct PyGeneration {
    pub generation_type: String,
    pub index: usize,
    pub score: Py<PyAny>,
    pub value: Py<PyAny>,
    pub metrics: Py<PyAny>,
    pub population: PyPopulation,
}

#[pymethods]
impl PyGeneration {
    pub fn geneneration_type(&self) -> String {
        self.generation_type.clone()
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

    pub fn metrics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.metrics.as_any().into_bound_py_any(py)
    }

    pub fn population<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyPopulation>> {
        Bound::new(py, self.population.clone())
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py)?;
        let value = self.value(py)?;

        Ok(format!(
            "{}(\n\tindex={},\n\tscore={},\n\tvalue={},\n\t metrics={})",
            self.generation_type,
            self.index,
            score,
            if self.generation_type == MULTI_OBJECTIVE_GENERATION {
                "ParetoFront".to_string()
            } else {
                value.to_string()
            },
            self.metrics
        ))
    }
}

impl<C> Into<PyGeneration> for Generation<C, ObjectValue>
where
    C: Chromosome + Clone,
    PyPopulation: From<Population<C>>,
{
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| PyGeneration {
            generation_type: SINGLE_OBJECTIVE_GENERATION.to_string(),
            index: self.index(),
            score: PyList::new(py, (*self.score().values).to_vec())
                .unwrap()
                .into_py_any(py)
                .unwrap(),
            value: self.value().clone().inner,
            metrics: Wrap(self.metrics())
                .into_pyobject(py)
                .unwrap()
                .into_py_any(py)
                .unwrap(),
            population: PyPopulation::from(self.ecosystem().population().clone()),
        })
    }
}

impl<C: Chromosome + Clone> Into<PyGeneration> for MultiObjectiveGeneration<C>
where
    PyPopulation: From<Population<C>>,
    PyGenotype: From<Genotype<C>>,
{
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let result = PyList::empty(py);

            for member in self.value().values().iter() {
                let temp = PyGenotype::from(member.genotype().clone());

                let fitness = member
                    .score()
                    .unwrap()
                    .values
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();

                let member = PyDict::new(py);
                member.set_item("genotype", temp).unwrap();
                member.set_item("fitness", fitness).unwrap();

                result.append(member).unwrap();
            }

            PyGeneration {
                generation_type: MULTI_OBJECTIVE_GENERATION.to_string(),
                index: self.index(),
                score: py.None(),
                value: result.into_py_any(py).unwrap(),
                metrics: Wrap(self.metrics())
                    .into_pyobject(py)
                    .unwrap()
                    .into_py_any(py)
                    .unwrap(),
                population: PyPopulation::from(self.ecosystem().population().clone()),
            }
        })
    }
}
