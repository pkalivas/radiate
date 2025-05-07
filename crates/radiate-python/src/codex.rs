use pyo3::{pyclass, pymethods};
use std::ops::Range;

#[pyclass]
#[derive(Clone)]
pub struct PyFloatCodex {
    pub chromosome_lengths: Vec<usize>,
    pub value_range: Range<f32>,
    pub bound_range: Range<f32>,
}

#[pymethods]
impl PyFloatCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None))]
    pub fn new(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
    ) -> Self {
        let val_range = value_range.unwrap_or((0.0, 1.0));
        let bound_range = bound_range.unwrap_or(val_range);
        PyFloatCodex {
            chromosome_lengths: chromosome_lengths.unwrap_or(vec![1]),
            value_range: val_range.0..val_range.1,
            bound_range: bound_range.0..bound_range.1,
        }
    }
}
