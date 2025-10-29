use crate::{PyAnyObject, PyGeneType};
use pyo3::{
    Py, PyAny, PyResult, Python, exceptions::PyKeyError, prelude::FromPyObjectOwned, pyclass,
    pymethods,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
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
    Population,
    Subscriber,
}

#[pyclass]
#[derive(Clone)]
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
                .map(|(k, v)| (k, PyAnyObject { inner: v.into() }))
                .collect(),
        }
    }

    pub fn component(&self) -> String {
        self.component.clone()
    }

    pub fn input_type(&self) -> PyEngineInputType {
        self.input_type.clone()
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
            Some(v) => v.extract(py),
            None => Err(PyKeyError::new_err(format!(
                "Key '{}' not found in PyEngineInput args",
                key
            ))),
        })
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.args.get(key).and_then(|v| v.get_string())
    }

    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.args.get(key).and_then(|v| v.get_i32())
    }

    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.args.get(key).and_then(|v| v.get_f32())
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.args.get(key).and_then(|v| v.get_f64())
    }

    pub fn get_usize(&self, key: &str) -> Option<usize> {
        self.args.get(key).and_then(|v| v.get_usize())
    }

    pub fn get_vec_f32(&self, key: &str) -> Option<Vec<f32>> {
        self.args.get(key).and_then(|v| v.get_vec_f32())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.args.get(key).and_then(|v| v.get_bool())
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
        if args.len() > 0 {
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
