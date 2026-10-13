use crate::{
    PyEngineInput, PyEngineInputType,
    bindings::converters::{FromPyEngineInput, Registry},
    registry,
};
use pyo3::{
    Py, PyAny, PyResult, Python, exceptions::PyTypeError, pyclass, pymethods, types::PyAnyMethods,
};
use radiate::{CosineDistance, EuclideanDistance, FitnessFunction, HammingDistance, NoveltySearch};

/// Novelty compares descriptors (`Vec<f32>`), not genotypes, so only the vector distances
/// apply. `NoveltySearch::new` needs a concrete distance type, so the registry builds the
/// search itself rather than a boxed distance.
fn novelty_searches() -> Registry<NoveltySearch<Vec<f32>>> {
    use crate::constants::components;
    registry!("novelty distance", |d| NoveltySearch::new(|v: &Vec<f32>| v.clone(), d); {
        components::EUCLIDEAN_DISTANCE => EuclideanDistance,
        components::COSINE_DISTANCE    => CosineDistance,
        components::HAMMING_DISTANCE   => HammingDistance,
    })
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
        distance: PyEngineInput,
    ) -> PyResult<Self> {
        distance.expect_type(PyEngineInputType::Diversity)?;
        let search = novelty_searches().convert(&distance)?;

        Ok(PyNoveltySearch {
            descriptor,
            inner: search.k(k).threshold(threshold).archive_size(archive_size),
        })
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
