use crate::{IntoPyAnyObject, PyAnyObject};
use pyo3::{IntoPyObjectExt, PyResult, Python, pyclass, pymethods};
use radiate::{Eval, Format, Op, ToDot, Tree};
use serde::{Deserialize, Serialize};

impl IntoPyAnyObject for Vec<Tree<Op<f32>>> {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject {
        PyAnyObject {
            inner: PyTree { inner: self }.into_py_any(py).unwrap(),
        }
    }
}

#[pyclass]
#[derive(Clone, Serialize, Deserialize)]
pub struct PyTree {
    pub inner: Vec<Tree<Op<f32>>>,
}

#[pymethods]
impl PyTree {
    #[staticmethod]
    pub fn from_json(json: &str) -> PyResult<Self> {
        serde_json::from_str::<PyTree>(json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e)))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_dot(&self) -> String {
        self.inner
            .iter()
            .map(|tree| tree.to_dot())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn eval(&mut self, inputs: Vec<Vec<f32>>) -> PyResult<Vec<Vec<f32>>> {
        Ok(inputs
            .into_iter()
            .map(|input| self.inner.eval(&input))
            .collect::<Vec<Vec<f32>>>())
    }

    pub fn __repr__(&self) -> String {
        let mut result = String::new();
        result.push_str("Tree(\n");
        for tree in self.inner.iter() {
            result.push_str(tree.format().as_str());
        }
        result.push(')');

        result
    }

    pub fn __str__(&self) -> String {
        let mut result = String::new();
        result.push_str("Tree(\n");
        for node in self.inner.iter() {
            result.push_str(&format!("{:?}\n", node));
        }
        result.push(')');

        result
    }

    pub fn __len__(&self) -> usize {
        self.inner.len()
    }

    pub fn __eq__(&self, other: &PyTree) -> bool {
        self.inner == other.inner
    }
}
