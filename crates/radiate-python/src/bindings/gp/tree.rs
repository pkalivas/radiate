use crate::{IntoPyAnyObject, PyAnyObject};
use pyo3::{IntoPyObjectExt, PyResult, Python, pyclass, pymethods};
use radiate::{Eval, Format, Node, Op, ToDot, Tree, TreeNode};

impl IntoPyAnyObject for Vec<Tree<Op<f32>>> {
    fn into_py<'py>(self, py: Python<'py>) -> PyAnyObject {
        PyAnyObject {
            inner: PyTree { inner: self }.into_py_any(py).unwrap(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyTree {
    pub inner: Vec<Tree<Op<f32>>>,
}

#[pymethods]
impl PyTree {
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

    pub fn nodes(&self) -> Vec<PyTreeNode> {
        self.inner
            .iter()
            .filter_map(|tree| tree.root())
            .map(|node| {
                let cloned = node.clone();

                PyTreeNode {
                    inner: cloned,
                    children: None,
                }
            })
            .collect()
    }

    pub fn eval(&mut self, inputs: Vec<Vec<f32>>) -> PyResult<Vec<Vec<f32>>> {
        Ok(inputs
            .into_iter()
            .map(|input| self.inner.eval(&input))
            .collect::<Vec<Vec<f32>>>())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap()
    }

    pub fn to_dot(&self) -> String {
        self.inner
            .iter()
            .map(|tree| tree.to_dot())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyTreeNode {
    pub inner: TreeNode<Op<f32>>,
    pub children: Option<Vec<PyTreeNode>>,
}

#[pymethods]
impl PyTreeNode {
    pub fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }

    pub fn value(&self) -> String {
        self.inner.value().to_string()
    }

    pub fn children(&mut self) -> Vec<PyTreeNode> {
        if self.children.is_none() {
            let inner_children = self.inner.take_children().map(|children| {
                children
                    .into_iter()
                    .map(|child| PyTreeNode {
                        inner: child,
                        children: None,
                    })
                    .collect::<Vec<_>>()
            });

            self.children = inner_children;
        }

        self.children.clone().unwrap_or_default()
    }
}
