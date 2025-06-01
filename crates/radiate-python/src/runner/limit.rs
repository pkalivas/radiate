use pyo3::pyclass;

#[pyclass(unsendable)]
#[derive(Clone, Debug)]
pub enum PyLimit {
    Generation(usize),
    Seconds(f64),
    Score(f32),
}
