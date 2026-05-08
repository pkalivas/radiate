use super::PyGenotype;
use crate::bindings::gp::{PyGraph, PyTree};
use crate::{
    EpochHandle, IntoPyAnyObject, PyAnyObject, PyEcosystem, PyFront, PyMetricSet, PyPopulation,
    PySpecies,
};

use pyo3::{
    Bound, BoundObject, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyBytes, PyBytesMethods, PyList},
};
use radiate::prelude::*;
use std::time::Duration;

macro_rules! match_epoch {
    ($handle:expr, $epoch:ident => $body:expr) => {{
        use crate::EpochHandle::*;
        match $handle {
            UInt8($epoch) => $body,
            UInt16($epoch) => $body,
            UInt32($epoch) => $body,
            UInt64($epoch) => $body,
            Int8($epoch) => $body,
            Int16($epoch) => $body,
            Int32($epoch) => $body,
            Int64($epoch) => $body,
            Float32($epoch) => $body,
            Float64($epoch) => $body,
            Char($epoch) => $body,
            Bit($epoch) => $body,
            Permutation($epoch) => $body,
            Graph($epoch) => $body,
            Tree($epoch) => $body,
        }
    }};
}

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

    pub fn to_pickle<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let pickle = serde_pickle::to_vec(&self.inner, serde_pickle::SerOptions::default())
            .map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Failed to serialize to pickle: {}",
                    e
                ))
            })?;

        Ok(PyBytes::new(python, &pickle).into_bound())
    }

    #[staticmethod]
    pub fn from_pickle<'py>(pickle_bytes: &Bound<'py, PyBytes>) -> PyResult<Self> {
        let pickle = pickle_bytes.as_bytes();
        let handle =
            serde_pickle::from_slice::<EpochHandle>(pickle, serde_pickle::DeOptions::default())
                .map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!(
                        "Failed to deserialize pickle: {}",
                        e
                    ))
                })?;

        Ok(PyGeneration::new(handle))
    }

    pub fn index(&self) -> usize {
        match_epoch!(&self.inner, epoch => epoch.index())
    }

    pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let inner_score = match_epoch!(&self.inner, epoch => epoch.score());
        let score = inner_score.iter().cloned().collect::<Vec<f32>>();

        Ok(PyList::new(py, score)?.into_bound())
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
        match_epoch!(&self.inner, epoch => get_front(epoch))
    }

    pub fn metrics(&self) -> PyResult<PyMetricSet> {
        let metrics = match_epoch!(&self.inner, epoch => epoch.metrics());
        Ok(PyMetricSet::from(metrics.clone()))
    }

    pub fn ecosystem(&self) -> PyEcosystem {
        match_epoch!(&self.inner, epoch => PyEcosystem::from(epoch.ecosystem().clone()))
    }

    pub fn species(&self) -> Option<Vec<PySpecies>> {
        match_epoch!(&self.inner, epoch => epoch.species().map(|s| s.iter().cloned().map(PySpecies::from).collect()))
    }

    pub fn population(&self) -> PyPopulation {
        match_epoch!(&self.inner, epoch => PyPopulation::from(epoch.population()))
    }

    pub fn duration(&self) -> Duration {
        match_epoch!(&self.inner, epoch => epoch.time())
    }

    pub fn objective(&self) -> Vec<String> {
        match_epoch!(&self.inner, epoch => get_objective_names(epoch.objective()))
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.population().dtype(py)
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py)?;
        let value = self.value(py)?;
        let metrics = self.metrics()?;
        let dtype = self.dtype(py)?;

        let (objective, index) =
            match_epoch!(&self.inner, epoch => (epoch.objective(), epoch.index()));

        Ok(format!(
            "Generation(\n\tindex={},\n\tscore={},\n\tdtype={},\n\t{},\n\tvalue={}\n)",
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

fn get_front<C, T: IntoPyAnyObject>(generation: &Generation<C, T>) -> PyResult<PyFront>
where
    C: Chromosome + Clone,
    PyGenotype: From<Genotype<C>>,
{
    let mut front_objs = Vec::new();
    if let Some(front) = generation.front() {
        for member in front.values().iter() {
            let temp = PyGenotype::from(member.genotype().clone());

            front_objs.push(super::front::PyFrontValue {
                genotype: temp,
                score: member.score().cloned(),
            });
        }
    }

    Ok(PyFront::new(front_objs, generation.objective().clone()))
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

    Ok(py.None().into_bound(py))
}
