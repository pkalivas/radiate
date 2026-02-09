use crate::{PyPhenotype, Wrap};

use super::PyGenotype;
use pyo3::{
    Bound, IntoPyObject, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods,
    types::{PyDict, PyDictMethods},
};
use radiate::{
    Front, Objective, Optimize, Score,
    objectives::{FrontAddResult, Scored},
};
use std::sync::Arc;

pub enum PyFrontInner {
    Front(Arc<Front<PyFrontValue>>),
    Values(Vec<PyFrontValue>),
}

#[pyclass(skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct PyFrontValue {
    pub(crate) genotype: PyGenotype,
    pub(crate) score: Option<Score>,
}

#[pymethods]
impl PyFrontValue {
    pub fn genotype(&self) -> PyGenotype {
        self.genotype.clone()
    }

    pub fn score(&self) -> Option<Vec<f32>> {
        self.score.as_ref().map(|s| s.clone().into())
    }
}

impl Eq for PyFrontValue {}

impl Scored for PyFrontValue {
    fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }
}

#[pyclass(skip_from_py_object)]
pub struct PyFront {
    pub(crate) inner: PyFrontInner,
    pub(crate) objective: Objective,
}

impl PyFront {
    pub fn new(inner: Vec<PyFrontValue>, objective: Objective) -> Self {
        Self {
            inner: PyFrontInner::Values(inner),
            objective,
        }
    }
}

#[pymethods]
impl PyFront {
    #[new]
    #[pyo3(signature = (range, objective))]
    pub fn py_new<'py>(range: (usize, usize), objective: Vec<String>) -> Self {
        let objective: Objective = objective
            .into_iter()
            .map(|s| Optimize::from(s.as_str()))
            .collect::<Vec<Optimize>>()
            .into();
        let inner = PyFrontInner::Front(Arc::new(Front::new(range.0..range.1, objective.clone())));
        Self { inner, objective }
    }

    pub fn __len__(&self) -> PyResult<usize> {
        match &self.inner {
            PyFrontInner::Front(front) => Ok(front.len()),
            PyFrontInner::Values(values) => Ok(values.len()),
        }
    }

    pub fn __getitem__(&self, index: isize) -> PyResult<PyFrontValue> {
        let len = match &self.inner {
            PyFrontInner::Front(front) => front.len() as isize,
            PyFrontInner::Values(values) => values.len() as isize,
        };
        let idx = if index < 0 { len + index } else { index };

        if idx < 0 || idx >= len {
            Err(pyo3::exceptions::PyIndexError::new_err(
                "Index out of range",
            ))
        } else {
            match &self.inner {
                PyFrontInner::Front(front) => Ok((*front.values()[idx as usize]).clone()),
                PyFrontInner::Values(values) => Ok(values[idx as usize].clone()),
            }
        }
    }

    pub fn add<'py>(
        &mut self,
        py: Python<'py>,
        phenotypes: Vec<PyPhenotype>,
    ) -> PyResult<Py<PyAny>> {
        self.to_front();

        let value = phenotypes
            .into_iter()
            .map(|p| PyFrontValue {
                genotype: p.genotype,
                score: if p.score.is_empty() {
                    None
                } else {
                    Some(p.score.into())
                },
            })
            .collect::<Vec<_>>();

        if let PyFrontInner::Front(front) = &mut self.inner {
            Ok(Wrap(Arc::make_mut(front).add_all(value))
                .into_pyobject(py)?
                .unbind()
                .into())
        } else {
            Ok(py.None())
        }
    }

    pub fn values(&self) -> Vec<PyFrontValue> {
        match &self.inner {
            PyFrontInner::Front(front) => front
                .values()
                .iter()
                .map(|v| (*(*v)).clone())
                .collect::<Vec<_>>(),
            PyFrontInner::Values(values) => values.clone(),
        }
    }

    pub fn remove_outliers(&mut self, trim: f32) -> Option<usize> {
        self.to_front();

        if let PyFrontInner::Front(front) = &mut self.inner {
            Arc::make_mut(front).remove_outliers(trim)
        } else {
            None
        }
    }

    pub fn entropy(&mut self) -> Option<f32> {
        self.to_front();

        if let PyFrontInner::Front(front) = &mut self.inner {
            Arc::make_mut(front).entropy()
        } else {
            None
        }
    }

    pub fn crowding_distance(&mut self) -> Option<Vec<f32>> {
        self.to_front();

        if let PyFrontInner::Front(front) = &mut self.inner {
            Arc::make_mut(front)
                .crowding_distance()
                .map(|distances| distances.to_vec())
        } else {
            None
        }
    }

    pub fn fronts(&mut self) -> Vec<PyFront> {
        self.to_front();

        if let PyFrontInner::Front(front) = &mut self.inner {
            Arc::make_mut(front)
                .fronts()
                .into_iter()
                .map(|f| PyFront {
                    inner: PyFrontInner::Front(Arc::new(f)),
                    objective: self.objective.clone(),
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }

    fn to_front(&mut self) {
        if let PyFrontInner::Values(values) = &mut self.inner {
            let rng = values.len()..values.len();
            let mut front = Front::new(rng, self.objective.clone());
            front.add_all(values.clone());
            let arc_front = Arc::new(front);
            self.inner = PyFrontInner::Front(arc_front.clone());
        }
    }
}

impl<'py> IntoPyObject<'py> for Wrap<FrontAddResult> {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: pyo3::Python<'py>) -> Result<Self::Output, Self::Error> {
        let add = self.0;
        let dict = pyo3::types::PyDict::new(py);

        dict.set_item("added", add.added_count)?;
        dict.set_item("removed", add.removed_count)?;
        dict.set_item("filtered", add.filter_count)?;
        dict.set_item("comparisons", add.comparisons)?;
        dict.set_item("size", add.size)?;

        Ok(dict.into())
    }
}
