use pyo3::pyclass;

#[pyclass(name = "Epoch")]
#[derive(Clone, Debug)]
pub struct PyEpoch {
    pub index: usize,
    pub value: f64,
    pub score: f64,
}
