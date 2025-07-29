use pyo3::{
    Py, PyAny, PyResult, Python, exceptions::PyTypeError, pyclass, pymethods, types::PyAnyMethods,
};
use radiate::{CosineDistance, EuclideanDistance, FitnessFunction, HammingDistance, NoveltySearch};

pub enum NoveltyInner {
    Euclidean(NoveltySearch<Vec<f32>, EuclideanDistance>),
    Cosine(NoveltySearch<Vec<f32>, CosineDistance>),
    Hamming(NoveltySearch<Vec<f32>, HammingDistance>),
}

#[pyclass]
pub struct PyNoveltySearch {
    pub descriptor: Option<Py<PyAny>>,
    pub novelty: NoveltyInner,
}

#[pymethods]
impl PyNoveltySearch {
    #[new]
    pub fn new(
        descriptor: Option<Py<PyAny>>,
        distance: String,
        k: usize,
        threshold: f32,
        archive_size: usize,
    ) -> Self {
        PyNoveltySearch {
            descriptor,
            novelty: match distance.as_str() {
                "EuclideanDistance" => NoveltyInner::Euclidean(
                    NoveltySearch::new(EuclideanDistance, k, threshold)
                        .with_max_archive_size(archive_size),
                ),
                "CosineDistance" => NoveltyInner::Cosine(
                    NoveltySearch::new(CosineDistance, k, threshold)
                        .with_max_archive_size(archive_size),
                ),
                "HammingDistance" => NoveltyInner::Hamming(
                    NoveltySearch::new(HammingDistance, k, threshold)
                        .with_max_archive_size(archive_size),
                ),
                _ => panic!("Invalid distance metric: {}", distance),
            },
        }
    }

    pub fn __call__<'py>(&self, py: Python<'py>, genotype: Py<PyAny>) -> PyResult<f32> {
        let described = match &self.descriptor {
            Some(desc) => desc.bind(py).call1((genotype,))?,
            None => genotype.into_bound(py),
        };

        if let Ok(vals) = described.extract::<Vec<f32>>() {
            Ok(match &self.novelty {
                NoveltyInner::Euclidean(n) => n.evaluate(&vals),
                NoveltyInner::Cosine(n) => n.evaluate(&vals),
                NoveltyInner::Hamming(n) => n.evaluate(&vals),
            })
        } else if let Ok(vals) = described.extract::<Vec<i32>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(match &self.novelty {
                NoveltyInner::Euclidean(n) => n.evaluate(&vals),
                NoveltyInner::Cosine(n) => n.evaluate(&vals),
                NoveltyInner::Hamming(n) => n.evaluate(&vals),
            })
        } else if let Ok(vals) = described.extract::<Vec<f64>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(match &self.novelty {
                NoveltyInner::Euclidean(n) => n.evaluate(&vals),
                NoveltyInner::Cosine(n) => n.evaluate(&vals),
                NoveltyInner::Hamming(n) => n.evaluate(&vals),
            })
        } else if let Ok(vals) = described.extract::<Vec<usize>>() {
            let vals: Vec<f32> = vals.into_iter().map(|v| v as f32).collect();
            Ok(match &self.novelty {
                NoveltyInner::Euclidean(n) => n.evaluate(&vals),
                NoveltyInner::Cosine(n) => n.evaluate(&vals),
                NoveltyInner::Hamming(n) => n.evaluate(&vals),
            })
        } else {
            Err(PyTypeError::new_err(
                "Descriptor did not return a vector of f32 values",
            ))
        }
    }
}
