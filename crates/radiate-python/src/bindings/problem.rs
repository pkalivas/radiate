use crate::{ObjectValue, PyGeneType};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyDict, PyDictMethods},
};

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyProblemBuilder {
    pub name: String,
    pub args: ObjectValue,
    pub allowed_genes: Vec<PyGeneType>,
}

#[pymethods]
impl PyProblemBuilder {
    pub fn __repr__(&self) -> String {
        format!(
            "PyTestProblem(name='{}', args={:?}, allowed_genes={:?})",
            self.name, self.args, self.allowed_genes
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.args.inner.bind(py).into_bound_py_any(py)
    }

    #[staticmethod]
    pub fn custom<'py>(py: Python<'py>, fitness_fn: Py<PyAny>) -> Self {
        let args = PyDict::new(py);
        args.set_item("fitness_func", fitness_fn).unwrap();

        PyProblemBuilder {
            name: "Custom".into(),
            args: ObjectValue {
                inner: args.unbind().into_any(),
            },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Char,
                PyGeneType::Bit,
                PyGeneType::Permutation,
            ],
        }
    }

    #[staticmethod]
    pub fn regression<'py>(
        py: Python<'py>,
        features: Vec<Vec<f32>>,
        targets: Vec<Vec<f32>>,
        loss: String,
    ) -> Self {
        let args = PyDict::new(py);

        args.set_item("features", features).unwrap();
        args.set_item("targets", targets).unwrap();
        args.set_item("loss", loss).unwrap();

        PyProblemBuilder {
            name: "Regression".into(),
            args: ObjectValue {
                inner: args.unbind().into_any(),
            },
            allowed_genes: vec![PyGeneType::Graph],
        }
    }
}
