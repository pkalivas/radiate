use pyo3::{pyclass, pymethods};
use radiate::{Codex, FloatChromosome, FloatCodex, Genotype};

#[pyclass(name = "FloatCodex")]
#[derive(Default, Clone)]
pub struct PyFloatCodex {
    pub scalar: Option<FloatCodex>,
    pub vector: Option<FloatCodex<Vec<f32>>>,
    pub matrix: Option<FloatCodex<Vec<Vec<f32>>>>,
}

#[pymethods]
impl PyFloatCodex {
    #[new]
    #[pyo3(signature = (num_genes=1, num_chromosomes=0, range=None, bounds=None))]
    pub fn new(
        num_genes: usize,
        num_chromosomes: usize,
        range: Option<(f32, f32)>,
        bounds: Option<(f32, f32)>,
    ) -> Self {
        let range = if let Some((min, max)) = range {
            min..max
        } else {
            0.0..1.0
        };
        let bounds = if let Some((min, max)) = bounds {
            min..max
        } else {
            range.clone()
        };

        if num_chromosomes == 1 {
            let codex = FloatCodex::vector(num_genes, range).with_bounds(bounds);
            return PyFloatCodex {
                vector: Some(codex),
                ..Default::default()
            };
        } else if num_chromosomes > 1 {
            let codex = FloatCodex::matrix(num_chromosomes, num_genes, range).with_bounds(bounds);
            return PyFloatCodex {
                matrix: Some(codex),
                ..Default::default()
            };
        }
        let codex = FloatCodex::scalar(range).with_bounds(bounds);

        PyFloatCodex {
            scalar: Some(codex),
            ..Default::default()
        }
    }
}
