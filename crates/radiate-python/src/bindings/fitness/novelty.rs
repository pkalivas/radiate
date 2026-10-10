use pyo3::{
    Py, PyAny, PyResult, Python, exceptions::PyTypeError, pyclass, pymethods, types::PyAnyMethods,
};
use radiate::{CosineDistance, EuclideanDistance, FitnessFunction, HammingDistance, NoveltySearch};

#[pyclass]
pub struct PyNoveltySearch {
    descriptor: Py<PyAny>,
    inner: NoveltySearch<Vec<f32>>,
}

#[pymethods]
impl PyNoveltySearch {
    #[new]
    pub fn new(
        descriptor: Py<PyAny>,
        k: usize,
        threshold: f32,
        archive_size: usize,
        distance: String,
    ) -> Self {
        // The Python descriptor runs in `__call__`, so the inner search receives the
        // finished descriptor and only needs to pass it through.
        let identity = |described: &Vec<f32>| described.clone();
        let search = if distance == crate::constants::components::EUCLIDEAN_DISTANCE {
            NoveltySearch::new(identity, EuclideanDistance)
        } else if distance == crate::constants::components::COSINE_DISTANCE {
            NoveltySearch::new(identity, CosineDistance)
        } else {
            NoveltySearch::new(identity, HammingDistance)
        };

        PyNoveltySearch {
            descriptor,
            inner: search.k(k).threshold(threshold).archive_size(archive_size),
        }
    }

    pub fn __call__<'py>(&self, py: Python<'py>, genotype: Py<PyAny>) -> PyResult<f32> {
        let described = self.descriptor.bind(py).call1((genotype,))?;

        if let Ok(vals) = described.extract::<Vec<f32>>() {
            Ok(self.inner.evaluate(vals))
        } else if let Ok(vals) = described.extract::<Vec<f64>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(vals))
        } else if let Ok(vals) = described.extract::<Vec<i32>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(vals))
        } else if let Ok(vals) = described.extract::<Vec<f64>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(vals))
        } else if let Ok(vals) = described.extract::<Vec<usize>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(vals))
        } else {
            Err(PyTypeError::new_err(
                "Descriptor did not return a vector of f32 values",
            ))
        }
    }
}
