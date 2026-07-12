use crate::{IntoPyAnyObject, PyAnyObject};
use numpy::PyArrayDyn;
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, Python, prelude::FromPyObjectOwned, pyclass, pymethods,
};
use radiate::{Eval, Format, Op, ToDot, Tree};
use radiate_utils::Float;
use serde::{Deserialize, Serialize};

fn eval_trees<'py, F>(
    py: Python<'py>,
    trees: &[Tree<Op<F>>],
    output_len: usize,
    inputs: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyArrayDyn<F>>>
where
    F: Float + numpy::Element + FromPyObjectOwned<'py>,
{
    super::generic_eval_runner(py, output_len, inputs, |slice| {
        trees
            .iter()
            .map(|tree| tree.eval(slice))
            .collect::<Vec<F>>()
    })
}

impl IntoPyAnyObject for Vec<Tree<Op<f32>>> {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject {
        PyAnyObject {
            inner: PyTree {
                inner: PyTreeInner::Float32(self),
            }
            .into_py_any(py)
            .unwrap(),
        }
    }
}

impl IntoPyAnyObject for Vec<Tree<Op<f64>>> {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject {
        PyAnyObject {
            inner: PyTree {
                inner: PyTreeInner::Float64(self),
            }
            .into_py_any(py)
            .unwrap(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum PyTreeInner {
    Float32(Vec<Tree<Op<f32>>>),
    Float64(Vec<Tree<Op<f64>>>),
}

impl From<Vec<Tree<Op<f32>>>> for PyTreeInner {
    fn from(trees: Vec<Tree<Op<f32>>>) -> Self {
        PyTreeInner::Float32(trees)
    }
}

impl From<Vec<Tree<Op<f64>>>> for PyTreeInner {
    fn from(trees: Vec<Tree<Op<f64>>>) -> Self {
        PyTreeInner::Float64(trees)
    }
}

#[pyclass(from_py_object)]
#[derive(Clone, Serialize, Deserialize)]
pub struct PyTree {
    pub inner: PyTreeInner,
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
        match &self.inner {
            PyTreeInner::Float32(trees) => trees
                .iter()
                .map(|tree| tree.to_dot())
                .collect::<Vec<String>>()
                .join("\n"),
            PyTreeInner::Float64(trees) => trees
                .iter()
                .map(|tree| tree.to_dot())
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }

    pub fn eval<'py>(
        &self,
        py: Python<'py>,
        inputs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let output_len = self.__len__();
        match &self.inner {
            PyTreeInner::Float32(trees) => {
                Ok(eval_trees(py, trees, output_len, inputs)?.into_any())
            }
            PyTreeInner::Float64(trees) => {
                Ok(eval_trees(py, trees, output_len, inputs)?.into_any())
            }
        }
    }

    pub fn __repr__(&self) -> String {
        let mut result = String::new();
        result.push_str("Tree(\n");
        match &self.inner {
            PyTreeInner::Float32(trees) => {
                for tree in trees {
                    result.push_str(tree.format().as_str());
                }
            }
            PyTreeInner::Float64(trees) => {
                for tree in trees {
                    result.push_str(tree.format().as_str());
                }
            }
        }
        result.push(')');

        result
    }

    pub fn __str__(&self) -> String {
        let mut result = String::new();
        result.push_str("Tree(\n");
        match &self.inner {
            PyTreeInner::Float32(trees) => {
                for tree in trees {
                    result.push_str(&format!("{:?}\n", tree));
                }
            }
            PyTreeInner::Float64(trees) => {
                for tree in trees {
                    result.push_str(&format!("{:?}\n", tree));
                }
            }
        }
        result.push(')');

        result
    }

    pub fn __len__(&self) -> usize {
        match &self.inner {
            PyTreeInner::Float32(trees) => trees.len(),
            PyTreeInner::Float64(trees) => trees.len(),
        }
    }

    pub fn __eq__(&self, other: &PyTree) -> bool {
        self.inner == other.inner
    }
}

impl From<Vec<Tree<Op<f32>>>> for PyTree {
    fn from(trees: Vec<Tree<Op<f32>>>) -> Self {
        PyTree {
            inner: PyTreeInner::Float32(trees),
        }
    }
}
impl From<Vec<Tree<Op<f64>>>> for PyTree {
    fn from(trees: Vec<Tree<Op<f64>>>) -> Self {
        PyTree {
            inner: PyTreeInner::Float64(trees),
        }
    }
}
