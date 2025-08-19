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
        PyNoveltySearch {
            descriptor,
            inner: if distance == crate::names::EUCLIDEAN_DISTANCE {
                NoveltySearch::new(EuclideanDistance, k, threshold)
                    .with_max_archive_size(archive_size)
                    .euclidean_distance()
            } else if distance == crate::names::COSINE_DISTANCE {
                NoveltySearch::new(CosineDistance, k, threshold)
                    .with_max_archive_size(archive_size)
                    .cosine_distance()
            } else {
                NoveltySearch::new(HammingDistance, k, threshold)
                    .with_max_archive_size(archive_size)
                    .hamming_distance()
            },
        }
    }

    pub fn __call__<'py>(&self, py: Python<'py>, genotype: Py<PyAny>) -> PyResult<f32> {
        let described = self.descriptor.bind(py).call1((genotype,))?;

        if let Ok(vals) = described.extract::<Vec<f32>>() {
            Ok(self.inner.evaluate(&vals))
        } else if let Ok(vals) = described.extract::<Vec<i32>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(&vals))
        } else if let Ok(vals) = described.extract::<Vec<f64>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(&vals))
        } else if let Ok(vals) = described.extract::<Vec<usize>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(self.inner.evaluate(&vals))
        } else {
            Err(PyTypeError::new_err(
                "Descriptor did not return a vector of f32 values",
            ))
        }
    }
}
