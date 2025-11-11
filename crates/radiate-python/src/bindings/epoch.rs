use super::PyGenotype;
use crate::EpochHandle;
use crate::PyAnyObject;
use crate::PyEcosystem;
use crate::PyMetricSet;
use crate::PyPopulation;
use crate::PySpecies;
use crate::bindings::gp::{PyGraph, PyTree};
use numpy::PyArray1;
use pyo3::intern;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyDict;
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::Generation;
use radiate::prelude::*;
use std::time::Duration;

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
            EpochHandle::Any(epoch) => epoch.index(),
            EpochHandle::Graph(epoch) => epoch.index(),
            EpochHandle::Tree(epoch) => epoch.index(),
            EpochHandle::Permutation(epoch) => epoch.index(),
        }
    }

    pub fn score<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f32>> {
        let inner_score = match &self.inner {
            EpochHandle::Int(epoch) => Some(epoch.score()),
            EpochHandle::Float(epoch) => Some(epoch.score()),
            EpochHandle::Char(epoch) => Some(epoch.score()),
            EpochHandle::Bit(epoch) => Some(epoch.score()),
            EpochHandle::Any(epoch) => Some(epoch.score()),
            EpochHandle::Graph(epoch) => Some(epoch.score()),
            EpochHandle::Tree(epoch) => Some(epoch.score()),
            EpochHandle::Permutation(epoch) => Some(epoch.score()),
        };

        let score = inner_score
            .map(|s| s.iter().cloned().collect::<Vec<f32>>())
            .unwrap_or_default();

        PyArray1::from_vec(py, score)
    }

    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            EpochHandle::Int(epoch) => get_value(py, epoch),
            EpochHandle::Float(epoch) => get_value(py, epoch),
            EpochHandle::Char(epoch) => get_value(py, epoch),
            EpochHandle::Bit(epoch) => get_value(py, epoch),
            EpochHandle::Any(epoch) => get_value(py, epoch),
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

    pub fn metrics(&self) -> PyResult<PyMetricSet> {
        let metrics = match &self.inner {
            EpochHandle::Int(epoch) => epoch.metrics(),
            EpochHandle::Float(epoch) => epoch.metrics(),
            EpochHandle::Char(epoch) => epoch.metrics(),
            EpochHandle::Bit(epoch) => epoch.metrics(),
            EpochHandle::Any(epoch) => epoch.metrics(),
            EpochHandle::Permutation(epoch) => epoch.metrics(),
            EpochHandle::Graph(epoch) => epoch.metrics(),
            EpochHandle::Tree(epoch) => epoch.metrics(),
        };

        Ok(PyMetricSet::from(metrics.clone()))
    }

    pub fn ecosystem(&self) -> PyEcosystem {
        match &self.inner {
            EpochHandle::Int(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            EpochHandle::Float(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            EpochHandle::Char(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            EpochHandle::Bit(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            EpochHandle::Any(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            EpochHandle::Permutation(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            EpochHandle::Graph(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            EpochHandle::Tree(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
        }
    }

    pub fn species(&self) -> Option<Vec<PySpecies>> {
        match &self.inner {
            EpochHandle::Int(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            EpochHandle::Float(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            EpochHandle::Char(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            EpochHandle::Bit(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            EpochHandle::Any(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            EpochHandle::Permutation(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            EpochHandle::Graph(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            EpochHandle::Tree(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
        }
    }

    pub fn population(&self) -> PyPopulation {
        match &self.inner {
            EpochHandle::Int(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Float(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Char(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Bit(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Any(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Permutation(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Graph(epoch) => PyPopulation::from(epoch.population()),
            EpochHandle::Tree(epoch) => PyPopulation::from(epoch.population()),
        }
    }

    pub fn duration(&self) -> Duration {
        match &self.inner {
            EpochHandle::Int(epoch) => epoch.time(),
            EpochHandle::Float(epoch) => epoch.time(),
            EpochHandle::Char(epoch) => epoch.time(),
            EpochHandle::Bit(epoch) => epoch.time(),
            EpochHandle::Any(epoch) => epoch.time(),
            EpochHandle::Permutation(epoch) => epoch.time(),
            EpochHandle::Graph(epoch) => epoch.time(),
            EpochHandle::Tree(epoch) => epoch.time(),
        }
    }

    pub fn objective(&self) -> Vec<String> {
        get_objective_names(match &self.inner {
            EpochHandle::Int(epoch) => epoch.objective(),
            EpochHandle::Float(epoch) => epoch.objective(),
            EpochHandle::Char(epoch) => epoch.objective(),
            EpochHandle::Bit(epoch) => epoch.objective(),
            EpochHandle::Any(epoch) => epoch.objective(),
            EpochHandle::Permutation(epoch) => epoch.objective(),
            EpochHandle::Graph(epoch) => epoch.objective(),
            EpochHandle::Tree(epoch) => epoch.objective(),
        })
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py);
        let value = self.value(py)?;
        let metrics = self.metrics()?;

        let (objective, index) = match &self.inner {
            EpochHandle::Int(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Float(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Char(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Bit(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Any(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Permutation(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Graph(epoch) => (epoch.objective(), epoch.index()),
            EpochHandle::Tree(epoch) => (epoch.objective(), epoch.index()),
        };

        Ok(format!(
            "(\n\tindex={},\n\tscore={},\n\t{},\n\tvalue={})",
            index,
            score,
            metrics.__repr__(),
            if objective.is_multi() {
                "ParetoFront".to_string()
            } else {
                value.to_string()
            },
        ))
    }
}

fn get_objective_names(objective: &Objective) -> Vec<String> {
    match objective {
        Objective::Single(opt) => {
            vec![match opt {
                Optimize::Minimize => String::from("min"),
                Optimize::Maximize => String::from("max"),
            }]
        }
        Objective::Multi(opts) => opts
            .iter()
            .map(|opt| {
                match opt {
                    Optimize::Minimize => "min",
                    Optimize::Maximize => "max",
                }
                .to_string()
            })
            .collect(),
    }
}

fn get_value<'py, C>(
    py: Python<'py>,
    generation: &Generation<C, PyAnyObject>,
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

            let fitness = member
                .score()
                .map(|inner| inner.iter().cloned().collect::<Vec<_>>());

            let member = PyDict::new(py);
            member.set_item(intern!(py, "genotype"), temp).unwrap();
            member.set_item(intern!(py, "fitness"), fitness).unwrap();

            result.append(member).unwrap();
        }
    }

    Ok(result.into_any())
}
