use pyo3::{
    Py, PyAny, PyResult, Python, exceptions::PyTypeError, pyclass, pymethods, types::PyAnyMethods,
};
use radiate::{FitnessFunction, NoveltySearch, fitness::Novelty};

pub struct PyDescriptor;

impl Novelty<Vec<f32>> for PyDescriptor {
    fn description(&self, desc: &Vec<f32>) -> Vec<f32> {
        desc.clone()
    }
}

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
        let mut search =
            NoveltySearch::new(PyDescriptor, k, threshold).with_max_archive_size(archive_size);

        if distance == "EuclideanDistance" {
            search = search.euclidean_distance();
        } else if distance == "CosineDistance" {
            search = search.cosine_distance();
        } else {
            search = search.hamming_distance();
        }

        PyNoveltySearch {
            descriptor,
            inner: search,
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
