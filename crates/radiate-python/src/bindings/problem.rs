use pyo3::{PyObject, pyclass, pymethods};
use radiate::DataSet;

#[pyclass]
pub struct PyDefaultProblem {
    pub fitness_func: PyObject,
    pub codec: PyObject,
}

#[pymethods]
impl PyDefaultProblem {
    #[new]
    pub fn new(fitness_func: PyObject, codec: PyObject) -> Self {
        PyDefaultProblem {
            fitness_func,
            codec,
        }
    }
}

#[pyclass]
pub struct PyRegression {
    pub data: DataSet,
    pub loss: String,
}

#[pymethods]
impl PyRegression {
    #[new]
    pub fn new(features: Vec<Vec<f32>>, targets: Vec<Vec<f32>>, loss: String) -> Self {
        let data = DataSet::new(features, targets);
        PyRegression { data, loss }
    }
}
