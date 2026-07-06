use crate::{PyAnyObject, PyGeneType};
use pyo3::{
    Py, PyAny, PyResult, Python, exceptions::PyKeyError, prelude::FromPyObjectOwned, pyclass,
    pymethods,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum PyEngineInputType {
    Alterer,
    OffspringSelector,
    SurvivorSelector,
    Diversity,
    Objective,
    Limit,
    PopulationSize,
    OffspringFraction,
    MaxSpeciesAge,
    MaxPhenotypeAge,
    FrontRange,
    Executor,
    Evaluator,
    SpeciesThreshold,
    TargetSpecies,
    Population,
    Subscriber,
    Generation,
    Checkpoint,
    Metric,
    Codec,
    FitnessFunction,
    Filter,
}

#[pyclass(from_py_object)]
#[derive(Clone, Serialize, Deserialize)]
pub struct PyEngineInput {
    pub component: String,
    pub input_type: PyEngineInputType,
    pub allowed_genes: HashSet<PyGeneType>,
    pub args: HashMap<String, PyAnyObject>,
}

#[pymethods]
impl PyEngineInput {
    #[new]
    pub fn new(
        component: String,
        input_type: PyEngineInputType,
        allowed_genes: HashSet<PyGeneType>,
        args: HashMap<String, Py<PyAny>>,
    ) -> Self {
        PyEngineInput {
            component,
            input_type,
            allowed_genes,
            args: args
                .into_iter()
                .map(|(k, v)| (k, PyAnyObject { inner: v }))
                .collect(),
        }
    }

    pub fn component(&self) -> String {
        self.component.clone()
    }

    pub fn input_type(&self) -> PyEngineInputType {
        self.input_type
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl PyEngineInput {
    pub fn extract<T: for<'py> FromPyObjectOwned<'py>>(&self, key: &str) -> PyResult<T> {
        Python::attach(|py| match self.args.get(key) {
            Some(v) => v.extract::<T>(py).map_err(|e| {
                PyKeyError::new_err(format!(
                    "Failed to extract key '{}' from PyEngineInput args: {}",
                    key, e
                ))
            }),
            None => Err(PyKeyError::new_err(format!(
                "Key '{}' not found in PyEngineInput args",
                key
            ))),
        })
    }

    pub fn get(&self, key: &str) -> Option<&PyAnyObject> {
        self.args.get(key)
    }
}

impl Debug for PyEngineInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut args = self
            .args
            .iter()
            .map(|(k, v)| format!("\t\t{}: {:?}", k, v))
            .collect::<Vec<String>>()
            .join("\n");
        if !args.is_empty() {
            args = format!("\n{}", args);
            args.push('\n');
        }
        write!(
            f,
            "PyEngineInput {{ \n\tcomponent: {}, \n\tinput_type: {:?}, \n\tallowed_genes: {:?}, \n\targs: {{{}}} \n}}",
            self.component, self.input_type, self.allowed_genes, args
        )
    }
}
