use super::PyGenotype;
use crate::EpochHandle;
use crate::bindings::codec::PyGraph;
use crate::conversion::Wrap;
use pyo3::IntoPyObject;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyDict;
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::prelude::*;

const SINGLE_OBJECTIVE_GENERATION: &str = "Generation";
const MULTI_OBJECTIVE_GENERATION: &str = "MultiObjectiveGeneration";

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
            EpochHandle::IntMulti(epoch) => epoch.index(),
            EpochHandle::FloatMulti(epoch) => epoch.index(),
            EpochHandle::GraphRegression(epoch) => epoch.index(),
        }
    }

    pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner_score = match &self.inner {
            EpochHandle::Int(epoch) => Some(epoch.score()),
            EpochHandle::Float(epoch) => Some(epoch.score()),
            EpochHandle::Char(epoch) => Some(epoch.score()),
            EpochHandle::Bit(epoch) => Some(epoch.score()),
            EpochHandle::IntMulti(_) => None,
            EpochHandle::FloatMulti(_) => None,
            EpochHandle::GraphRegression(epoch) => Some(epoch.score()),
        };

        match inner_score {
            Some(score) => Ok(PyList::new(py, (*score.values).to_vec())
                .unwrap()
                .into_any()),

            None => Ok(py.None().into_bound(py)),
        }
    }

    pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            EpochHandle::Int(epoch) => epoch.value().clone().inner.into_bound_py_any(py),
            EpochHandle::Float(epoch) => epoch.value().clone().inner.into_bound_py_any(py),
            EpochHandle::Char(epoch) => epoch.value().clone().inner.into_bound_py_any(py),
            EpochHandle::Bit(epoch) => epoch.value().clone().inner.into_bound_py_any(py),
            EpochHandle::IntMulti(epoch) => into_pareto_front(py, epoch),
            EpochHandle::FloatMulti(epoch) => into_pareto_front(py, epoch),
            EpochHandle::GraphRegression(epoch) => PyGraph {
                inner: epoch.value().clone(),
                eval_cache: None,
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
            EpochHandle::IntMulti(epoch) => epoch.metrics(),
            EpochHandle::FloatMulti(epoch) => epoch.metrics(),
            EpochHandle::GraphRegression(epoch) => epoch.metrics(),
        };

        Wrap(metrics)
            .into_pyobject(py)
            .unwrap()
            .into_py_any(py)
            .map(|b| b.into_bound(py))
    }

    // pub fn population<'py>(&self, py: Python<'py>) -> PyPopulation {
    //     match &self.inner {
    //         EpochHandle::Int(epoch) => PyPopulation::new(epoch.population().clone()),
    //         EpochHandle::Float(epoch) => epoch.population(),
    //         EpochHandle::Char(epoch) => epoch.population(),
    //         EpochHandle::Bit(epoch) => epoch.population(),
    //         EpochHandle::IntMulti(epoch) => epoch.population(),
    //         EpochHandle::FloatMulti(epoch) => epoch.population(),
    //         EpochHandle::GraphRegression(epoch) => epoch.population(),
    //     }
    // }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let score = self.score(py)?;
        let value = self.value(py)?;
        let metrics = self.metrics(py)?;

        let (generation_type, index) = match &self.inner {
            EpochHandle::Int(epoch) => (SINGLE_OBJECTIVE_GENERATION, epoch.index()),
            EpochHandle::Float(epoch) => (SINGLE_OBJECTIVE_GENERATION, epoch.index()),
            EpochHandle::Char(epoch) => (SINGLE_OBJECTIVE_GENERATION, epoch.index()),
            EpochHandle::Bit(epoch) => (SINGLE_OBJECTIVE_GENERATION, epoch.index()),
            EpochHandle::IntMulti(epoch) => (MULTI_OBJECTIVE_GENERATION, epoch.index()),
            EpochHandle::FloatMulti(epoch) => (MULTI_OBJECTIVE_GENERATION, epoch.index()),
            EpochHandle::GraphRegression(epoch) => (SINGLE_OBJECTIVE_GENERATION, epoch.index()),
        };

        Ok(format!(
            "{}(\n\tindex={},\n\tscore={},\n\tvalue={},\n\t metrics={})",
            generation_type,
            index,
            score,
            if generation_type == MULTI_OBJECTIVE_GENERATION {
                "ParetoFront".to_string()
            } else {
                value.to_string()
            },
            metrics
        ))
    }
}

fn into_pareto_front<'py, C>(
    py: Python<'py>,
    generation: &ParetoGeneration<C>,
) -> PyResult<Bound<'py, PyAny>>
where
    C: Chromosome + Clone,
    PyGenotype: From<Genotype<C>>,
{
    let result = PyList::empty(py);

    for member in generation.value().values().iter() {
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

    Ok(result.into_any())
}

// #[pyclass(unsendable)]
// pub struct PyGeneration {
//     pub generation_type: String,
//     pub index: usize,
//     pub score: Py<PyAny>,
//     pub value: Py<PyAny>,
//     pub metrics: Py<PyAny>,
//     pub population: PyPopulation,
// }

// #[pymethods]
// impl PyGeneration {
//     pub fn geneneration_type(&self) -> String {
//         self.generation_type.clone()
//     }

//     pub fn index(&self) -> usize {
//         self.index
//     }

//     pub fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.score.as_any().into_bound_py_any(py)
//     }

//     pub fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.value.as_any().into_bound_py_any(py)
//     }

//     pub fn metrics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//         self.metrics.as_any().into_bound_py_any(py)
//     }

//     pub fn population<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyPopulation>> {
//         Bound::new(py, self.population.clone())
//     }

//     pub fn __repr__(&self, py: Python) -> PyResult<String> {
//         let score = self.score(py)?;
//         let value = self.value(py)?;

//         Ok(format!(
//             "{}(\n\tindex={},\n\tscore={},\n\tvalue={},\n\t metrics={})",
//             self.generation_type,
//             self.index,
//             score,
//             if self.generation_type == MULTI_OBJECTIVE_GENERATION {
//                 "ParetoFront".to_string()
//             } else {
//                 value.to_string()
//             },
//             self.metrics
//         ))
//     }
// }

// impl<C> Into<PyGeneration> for Generation<C, ObjectValue>
// where
//     C: Chromosome + Clone,
//     PyPopulation: From<Population<C>>,
// {
//     fn into(self) -> PyGeneration {
//         Python::with_gil(|py| PyGeneration {
//             generation_type: SINGLE_OBJECTIVE_GENERATION.to_string(),
//             index: self.index(),
//             score: PyList::new(py, (*self.score().values).to_vec())
//                 .unwrap()
//                 .into_py_any(py)
//                 .unwrap(),
//             value: self.value().clone().inner,
//             metrics: Wrap(self.metrics())
//                 .into_pyobject(py)
//                 .unwrap()
//                 .into_py_any(py)
//                 .unwrap(),
//             population: PyPopulation::from(self.ecosystem().population().clone()),
//         })
//     }
// }

// impl<C> Into<PyGeneration> for Generation<C, Graph<Op<f32>>>
// where
//     C: Chromosome + Clone,
//     PyPopulation: From<Population<C>>,
// {
//     fn into(self) -> PyGeneration {
//         Python::with_gil(|py| PyGeneration {
//             generation_type: SINGLE_OBJECTIVE_GENERATION.to_string(),
//             index: self.index(),
//             score: PyList::new(py, (*self.score().values).to_vec())
//                 .unwrap()
//                 .into_py_any(py)
//                 .unwrap(),
//             value: PyGraph {
//                 inner: self.value().clone(),
//                 eval_cache: None,
//             }
//             .into_py_any(py)
//             .unwrap(),
//             metrics: Wrap(self.metrics())
//                 .into_pyobject(py)
//                 .unwrap()
//                 .into_py_any(py)
//                 .unwrap(),
//             population: PyPopulation::from(self.ecosystem().population().clone()),
//         })
//     }
// }

// impl<C: Chromosome + Clone> Into<PyGeneration> for ParetoGeneration<C>
// where
//     PyPopulation: From<Population<C>>,
//     PyGenotype: From<Genotype<C>>,
// {
//     fn into(self) -> PyGeneration {
//         Python::with_gil(|py| {
//             let result = PyList::empty(py);

//             for member in self.value().values().iter() {
//                 let temp = PyGenotype::from(member.genotype().clone());

//                 let fitness = member
//                     .score()
//                     .unwrap()
//                     .values
//                     .iter()
//                     .cloned()
//                     .collect::<Vec<_>>();

//                 let member = PyDict::new(py);
//                 member.set_item("genotype", temp).unwrap();
//                 member.set_item("fitness", fitness).unwrap();

//                 result.append(member).unwrap();
//             }

//             PyGeneration {
//                 generation_type: MULTI_OBJECTIVE_GENERATION.to_string(),
//                 index: self.index(),
//                 score: py.None(),
//                 value: result.into_py_any(py).unwrap(),
//                 metrics: Wrap(self.metrics())
//                     .into_pyobject(py)
//                     .unwrap()
//                     .into_py_any(py)
//                     .unwrap(),
//                 population: PyPopulation::from(self.ecosystem().population().clone()),
//             }
//         })
//     }
// }
