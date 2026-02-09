use super::PyGenotype;
use crate::EpochHandle;
use crate::IntoPyAnyObject;
use crate::PyAnyObject;
use crate::PyEcosystem;
use crate::PyFront;
use crate::PyMetricSet;
use crate::PyPopulation;
use crate::PySpecies;
use crate::bindings::gp::{PyGraph, PyTree};
use numpy::PyArray1;
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::Generation;
use radiate::prelude::*;
use std::time::Duration;

#[pyclass(from_py_object)]
pub struct PyGeneration {
    pub(crate) inner: EpochHandle,
}

impl PyGeneration {
    pub fn new(inner: EpochHandle) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyGeneration {
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to serialize to JSON: {}", e))
        })
    }

    #[staticmethod]
    pub fn from_json(json_str: &str) -> PyResult<Self> {
        let handle = serde_json::from_str::<EpochHandle>(json_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

        Ok(PyGeneration::new(handle))
    }

    pub fn index(&self) -> usize {
        use EpochHandle::*;
        match &self.inner {
            UInt8(epoch) => epoch.index(),
            UInt16(epoch) => epoch.index(),
            UInt32(epoch) => epoch.index(),
            UInt64(epoch) => epoch.index(),
            Int8(epoch) => epoch.index(),
            Int16(epoch) => epoch.index(),
            Int32(epoch) => epoch.index(),
            Int64(epoch) => epoch.index(),
            Float32(epoch) => epoch.index(),
            Float64(epoch) => epoch.index(),
            Char(epoch) => epoch.index(),
            Bit(epoch) => epoch.index(),
            Any(epoch) => epoch.index(),
            Graph(epoch) => epoch.index(),
            Tree(epoch) => epoch.index(),
            Permutation(epoch) => epoch.index(),
        }
    }

    pub fn score<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f32>> {
        use EpochHandle::*;
        let inner_score = match &self.inner {
            UInt8(epoch) => Some(epoch.score()),
            UInt16(epoch) => Some(epoch.score()),
            UInt32(epoch) => Some(epoch.score()),
            UInt64(epoch) => Some(epoch.score()),
            Int8(epoch) => Some(epoch.score()),
            Int16(epoch) => Some(epoch.score()),
            Int32(epoch) => Some(epoch.score()),
            Int64(epoch) => Some(epoch.score()),
            Float32(epoch) => Some(epoch.score()),
            Float64(epoch) => Some(epoch.score()),
            Char(epoch) => Some(epoch.score()),
            Bit(epoch) => Some(epoch.score()),
            Any(epoch) => Some(epoch.score()),
            Graph(epoch) => Some(epoch.score()),
            Tree(epoch) => Some(epoch.score()),
            Permutation(epoch) => Some(epoch.score()),
        };

        let score = inner_score
            .map(|s| s.iter().cloned().collect::<Vec<f32>>())
            .unwrap_or_default();

        PyArray1::from_vec(py, score)
    }

    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        use EpochHandle::*;

        match &self.inner {
            UInt8(epoch) => get_value(py, epoch),
            UInt16(epoch) => get_value(py, epoch),
            UInt32(epoch) => get_value(py, epoch),
            UInt64(epoch) => get_value(py, epoch),
            Int8(epoch) => get_value(py, epoch),
            Int16(epoch) => get_value(py, epoch),
            Int32(epoch) => get_value(py, epoch),
            Int64(epoch) => get_value(py, epoch),
            Float32(epoch) => get_value(py, epoch),
            Float64(epoch) => get_value(py, epoch),
            Char(epoch) => get_value(py, epoch),
            Bit(epoch) => get_value(py, epoch),
            Any(epoch) => get_value(py, epoch),
            Permutation(epoch) => get_value(py, epoch),
            Graph(epoch) => PyGraph {
                inner: epoch.value().clone(),
                eval_cache: None,
            }
            .into_bound_py_any(py),
            Tree(epoch) => PyTree {
                inner: epoch.value().clone(),
            }
            .into_bound_py_any(py),
        }
    }

    pub fn front(&self) -> PyResult<PyFront> {
        use EpochHandle::*;
        match &self.inner {
            UInt8(epoch) => get_front(epoch),
            UInt16(epoch) => get_front(epoch),
            UInt32(epoch) => get_front(epoch),
            UInt64(epoch) => get_front(epoch),
            Int8(epoch) => get_front(epoch),
            Int16(epoch) => get_front(epoch),
            Int32(epoch) => get_front(epoch),
            Int64(epoch) => get_front(epoch),
            Float32(epoch) => get_front(epoch),
            Float64(epoch) => get_front(epoch),
            Char(epoch) => get_front(epoch),
            Bit(epoch) => get_front(epoch),
            Any(epoch) => get_front(epoch),
            Permutation(epoch) => get_front(epoch),
            Graph(epoch) => get_front(epoch),
            Tree(epoch) => get_front(epoch),
        }
    }

    pub fn metrics(&self) -> PyResult<PyMetricSet> {
        use EpochHandle::*;
        let metrics = match &self.inner {
            UInt8(epoch) => epoch.metrics(),
            UInt16(epoch) => epoch.metrics(),
            UInt32(epoch) => epoch.metrics(),
            UInt64(epoch) => epoch.metrics(),
            Int8(epoch) => epoch.metrics(),
            Int16(epoch) => epoch.metrics(),
            Int32(epoch) => epoch.metrics(),
            Int64(epoch) => epoch.metrics(),
            Float32(epoch) => epoch.metrics(),
            Float64(epoch) => epoch.metrics(),
            Char(epoch) => epoch.metrics(),
            Bit(epoch) => epoch.metrics(),
            Any(epoch) => epoch.metrics(),
            Permutation(epoch) => epoch.metrics(),
            Graph(epoch) => epoch.metrics(),
            Tree(epoch) => epoch.metrics(),
        };

        Ok(PyMetricSet::from(metrics.clone()))
    }

    pub fn ecosystem(&mut self) -> PyEcosystem {
        use EpochHandle::*;
        match &mut self.inner {
            UInt8(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            UInt16(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            UInt32(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            UInt64(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Int8(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Int16(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Int32(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Int64(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Float32(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Float64(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Char(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Bit(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Any(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Permutation(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Graph(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
            Tree(epoch) => PyEcosystem::from(epoch.ecosystem().clone()),
        }
    }

    pub fn species(&mut self) -> Option<Vec<PySpecies>> {
        use EpochHandle::*;
        match &mut self.inner {
            UInt8(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            UInt16(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            UInt32(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            UInt64(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Int8(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Int16(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Int32(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Int64(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Float32(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Float64(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Char(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Bit(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Any(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Permutation(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Graph(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
            Tree(epoch) => epoch
                .species()
                .map(|s| s.iter().cloned().map(PySpecies::from).collect()),
        }
    }

    pub fn population(&mut self) -> PyPopulation {
        use EpochHandle::*;
        match &mut self.inner {
            UInt8(epoch) => PyPopulation::from(epoch.population()),
            UInt16(epoch) => PyPopulation::from(epoch.population()),
            UInt32(epoch) => PyPopulation::from(epoch.population()),
            UInt64(epoch) => PyPopulation::from(epoch.population()),
            Int8(epoch) => PyPopulation::from(epoch.population()),
            Int16(epoch) => PyPopulation::from(epoch.population()),
            Int32(epoch) => PyPopulation::from(epoch.population()),
            Int64(epoch) => PyPopulation::from(epoch.population()),
            Float32(epoch) => PyPopulation::from(epoch.population()),
            Float64(epoch) => PyPopulation::from(epoch.population()),
            Char(epoch) => PyPopulation::from(epoch.population()),
            Bit(epoch) => PyPopulation::from(epoch.population()),
            Any(epoch) => PyPopulation::from(epoch.population()),
            Permutation(epoch) => PyPopulation::from(epoch.population()),
            Graph(epoch) => PyPopulation::from(epoch.population()),
            Tree(epoch) => PyPopulation::from(epoch.population()),
        }
    }

    pub fn duration(&self) -> Duration {
        use EpochHandle::*;
        match &self.inner {
            UInt8(epoch) => epoch.time(),
            UInt16(epoch) => epoch.time(),
            UInt32(epoch) => epoch.time(),
            UInt64(epoch) => epoch.time(),
            Int8(epoch) => epoch.time(),
            Int16(epoch) => epoch.time(),
            Int32(epoch) => epoch.time(),
            Int64(epoch) => epoch.time(),
            Float32(epoch) => epoch.time(),
            Float64(epoch) => epoch.time(),
            Char(epoch) => epoch.time(),
            Bit(epoch) => epoch.time(),
            Any(epoch) => epoch.time(),
            Permutation(epoch) => epoch.time(),
            Graph(epoch) => epoch.time(),
            Tree(epoch) => epoch.time(),
        }
    }

    pub fn objective(&self) -> Vec<String> {
        use EpochHandle::*;
        get_objective_names(match &self.inner {
            UInt8(epoch) => epoch.objective(),
            UInt16(epoch) => epoch.objective(),
            UInt32(epoch) => epoch.objective(),
            UInt64(epoch) => epoch.objective(),
            Int8(epoch) => epoch.objective(),
            Int16(epoch) => epoch.objective(),
            Int32(epoch) => epoch.objective(),
            Int64(epoch) => epoch.objective(),
            Float32(epoch) => epoch.objective(),
            Float64(epoch) => epoch.objective(),
            Char(epoch) => epoch.objective(),
            Bit(epoch) => epoch.objective(),
            Any(epoch) => epoch.objective(),
            Permutation(epoch) => epoch.objective(),
            Graph(epoch) => epoch.objective(),
            Tree(epoch) => epoch.objective(),
        })
    }

    pub fn dtype(&self) -> String {
        use EpochHandle::*;
        match &self.inner {
            UInt8(_) => DataType::UInt8.to_string(),
            UInt16(_) => DataType::UInt16.to_string(),
            UInt32(_) => DataType::UInt32.to_string(),
            UInt64(_) => DataType::UInt64.to_string(),
            Int8(_) => DataType::Int8.to_string(),
            Int16(_) => DataType::Int16.to_string(),
            Int32(_) => DataType::Int32.to_string(),
            Int64(_) => DataType::Int64.to_string(),
            Float32(_) => DataType::Float32.to_string(),
            Float64(_) => DataType::Float64.to_string(),
            Char(_) => DataType::Char.to_string(),
            Bit(_) => DataType::Boolean.to_string(),
            Any(epoch) => epoch.population().get(0).map_or_else(
                || DataType::Null.to_string(),
                |phenotype| phenotype.genotype()[0].get(0).allele().dtype().to_string(),
            ),
            Permutation(_) => DataType::Usize.to_string(),
            Graph(_) => DataType::Float32.to_string(),
            Tree(_) => DataType::Float32.to_string(),
        }
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        use EpochHandle::*;
        let score = self.score(py);
        let value = self.value(py)?;
        let metrics = self.metrics()?;
        let dtype = self.dtype();

        let (objective, index) = match &self.inner {
            UInt8(epoch) => (epoch.objective(), epoch.index()),
            UInt16(epoch) => (epoch.objective(), epoch.index()),
            UInt32(epoch) => (epoch.objective(), epoch.index()),
            UInt64(epoch) => (epoch.objective(), epoch.index()),
            Int8(epoch) => (epoch.objective(), epoch.index()),
            Int16(epoch) => (epoch.objective(), epoch.index()),
            Int32(epoch) => (epoch.objective(), epoch.index()),
            Int64(epoch) => (epoch.objective(), epoch.index()),
            Float32(epoch) => (epoch.objective(), epoch.index()),
            Float64(epoch) => (epoch.objective(), epoch.index()),
            Char(epoch) => (epoch.objective(), epoch.index()),
            Bit(epoch) => (epoch.objective(), epoch.index()),
            Any(epoch) => (epoch.objective(), epoch.index()),
            Permutation(epoch) => (epoch.objective(), epoch.index()),
            Graph(epoch) => (epoch.objective(), epoch.index()),
            Tree(epoch) => (epoch.objective(), epoch.index()),
        };

        Ok(format!(
            "(\n\tindex={},\n\tscore={},\n\t{},\n\t{},\n\tvalue={})",
            index,
            score,
            dtype,
            metrics.__repr__(),
            if objective.is_multi() {
                "ParetoFront".to_string()
            } else {
                value.to_string()
            },
        ))
    }
}

impl Clone for PyGeneration {
    fn clone(&self) -> Self {
        PyGeneration {
            inner: self.inner.clone(),
        }
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

fn get_front<'py, C, T: IntoPyAnyObject>(generation: &Generation<C, T>) -> PyResult<PyFront>
where
    C: Chromosome + Clone,
    PyGenotype: From<Genotype<C>>,
{
    let mut front_objs = Vec::new();
    if let Some(front) = generation.front() {
        for member in front.values().iter() {
            let temp = PyGenotype::from(member.genotype().clone());

            let fitness = member
                .score()
                .map(|inner| inner.iter().cloned().collect::<Vec<_>>());

            front_objs.push(super::front::PyFrontValue {
                genotype: temp,
                score: fitness,
            });
        }
    }

    Ok(PyFront::new(front_objs))
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

    return Ok(py.None().into_bound(py));
}
