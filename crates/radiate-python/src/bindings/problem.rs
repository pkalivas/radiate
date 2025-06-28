use crate::{ObjectValue, PyChromosomeType, PyGeneType};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyDict, PyDictMethods},
};

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyTestProblem {
    pub name: String,
    pub args: ObjectValue,
    pub allowed_genes: Vec<PyGeneType>,
    pub allowed_chromosomes: Vec<PyChromosomeType>,
}

#[pymethods]
impl PyTestProblem {
    pub fn __repr__(&self) -> String {
        format!(
            "PyTestProblem(name='{}', args={:?}, allowed_genes={:?}, allowed_chromosomes={:?})",
            self.name, self.args, self.allowed_genes, self.allowed_chromosomes
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
    pub fn default<'py>(py: Python<'py>, fitness_fn: Py<PyAny>) -> Self {
        let args = PyDict::new(py);
        args.set_item("fitness_func", fitness_fn).unwrap();

        PyTestProblem {
            name: "DefaultProblem".into(),
            args: ObjectValue {
                inner: args.unbind().into_any(),
            },
            allowed_genes: vec![],
            allowed_chromosomes: vec![],
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

        PyTestProblem {
            name: "Regression".into(),
            args: ObjectValue {
                inner: args.unbind().into_any(),
            },
            allowed_genes: vec![PyGeneType::Graph],
            allowed_chromosomes: vec![PyChromosomeType::Graph],
        }
    }
}
