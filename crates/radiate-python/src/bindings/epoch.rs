use super::PyGenotype;
use crate::EpochHandle;
use crate::ObjectValue;
use crate::PyPopulation;
use crate::bindings::codec::PyGraph;
use crate::bindings::codec::PyTree;
use crate::object::Wrap;
use pyo3::IntoPyObject;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyDict;
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::Generation;
use radiate::prelude::*;

#[pyclass(unsendable)]
pub struct PyGeneration {
    inner: EpochHandle,
}

impl PyGeneration {
    pub fn new(inner: EpochHandle) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyGeneration {
    pub fn index(&self) -> usize {
        match &self.inner {
            EpochHandle::Int(epoch) => epoch.index(),
            EpochHandle::Float(epoch) => epoch.index(),
            EpochHandle::Char(epoch) => epoch.index(),
            EpochHandle::Bit(epoch) => epoch.index(),
            EpochHandle::Graph(epoch) => epoch.index(),
            EpochHandle::Tree(epoch) => epoch.index(),
            EpochHandle::Permutation(epoch) => epoch.index(),
        }
    }

    pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner_score = match &self.inner {
            EpochHandle::Int(epoch) => Some(epoch.score()),
            EpochHandle::Float(epoch) => Some(epoch.score()),
            EpochHandle::Char(epoch) => Some(epoch.score()),
            EpochHandle::Bit(epoch) => Some(epoch.score()),
            EpochHandle::Graph(epoch) => Some(epoch.score()),
            EpochHandle::Tree(epoch) => Some(epoch.score()),
            EpochHandle::Permutation(epoch) => Some(epoch.score()),
        };

        match inner_score {
            Some(score) => Ok(PyList::new(py, score.iter().cloned().collect::<Vec<_>>())
                .unwrap()
                .into_any()),

            None => Ok(py.None().into_bound(py)),
        }
    }

    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            EpochHandle::Int(epoch) => get_value(py, epoch),
            EpochHandle::Float(epoch) => get_value(py, epoch),
            EpochHandle::Char(epoch) => get_value(py, epoch),
            EpochHandle::Bit(epoch) => get_value(py, epoch),
            EpochHandle::Permutation(epoch) => get_value(py, epoch),
            EpochHandle::Graph(epoch) => PyGraph {
                inner: epoch.value().clone(),
                eval_cache: None,
            }
            .into_bound_py_any(py),
            EpochHandle::Tree(epoch) => PyTree {
                inner: epoch.value().clone(),
            }
            .into_bound_py_any(py),
        }
    }

    pub fn metrics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let metrics = match &self.inner {
            EpochHandle::Int(epoch) => epoch.metrics(),
            EpochHandle::Float(epoch) => epoch.metrics(),
            EpochHandle::Char(epoch) => epoch.metrics(),
            EpochHandle::Bit(epoch) => epoch.metrics(),
            EpochHandle::Permutation(epoch) => epoch.metrics(),
            EpochHandle::Graph(epoch) => epoch.metrics(),
            EpochHandle::Tree(epoch) => epoch.metrics(),
        };

        Wrap(metrics)
            .into_pyobject(py)
            .unwrap()
            .into_py_any(py)
            .map(|b| b.into_bound(py))
    }

    pub fn population(&self) -> PyPopulation {
        match &self.inner {
            EpochHandle::Int(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Float(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Char(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Bit(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Permutation(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Graph(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Tree(epoch) => PyPopulation::from(epoch.population()),
        }
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py)?;
        let value = self.value(py)?;
        let metrics = self.metrics(py)?;

        let (objective, index) = match &self.inner {
            EpochHandle::Int(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Float(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Char(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Bit(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Permutation(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Graph(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Tree(epoch) => (epoch.objective(), epoch.index()),
        };

        Ok(format!(
            "(\n\tindex={},\n\tscore={},\n\tvalue={},\n\t metrics={})",
            index,
            score,
            if objective.is_multi() {
                "ParetoFront".to_string()
            } else {
                value.to_string()
            },
            metrics
        ))
    }
}

fn get_value<'py, C>(
    py: Python<'py>,
    generation: &Generation<C, ObjectValue>,
) -> PyResult<Bound<'py, PyAny>>
where
    C: Chromosome + Clone,
    PyGenotype: From<Genotype<C>>,
{
    if generation.objective().is_single() {
        return generation.value().clone().inner.into_bound_py_any(py);
    }

    let result = PyList::empty(py);

    if let Some(front) = generation.front() {
        for member in front.values().iter() {
            let temp = PyGenotype::from(member.genotype().clone());

            let fitness = member.score().unwrap().iter().cloned().collect::<Vec<_>>();

            let member = PyDict::new(py);
            member.set_item("genotype", temp).unwrap();
            member.set_item("fitness", fitness).unwrap();

            result.append(member).unwrap();
        }
    }

    Ok(result.into_any())
}
