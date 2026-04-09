use pyo3::pyclass;

#[pyclass(from_py_object)] // Not marked as frozen for pickling, but that's the only &mut self method.
#[repr(transparent)]
#[derive(Clone)]
pub struct PyExpr {
    expr: String,
}
