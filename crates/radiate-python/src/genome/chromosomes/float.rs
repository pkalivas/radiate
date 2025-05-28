use pyo3::{pyclass, pymethods};
use radiate::{FloatChromosome, FloatGene, Gene};

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyFloatGene {
    pub inner: FloatGene,
}

#[pymethods]
impl PyFloatGene {
    #[new]
    #[pyo3(signature=(min=0.0, max=1.0, min_bound=None, max_bound=None))]
    pub fn new(min: f32, max: f32, min_bound: Option<f32>, max_bound: Option<f32>) -> Self {
        let range = min..max;
        let bounds = if let (Some(min_bound), Some(max_bound)) = (min_bound, max_bound) {
            min_bound..max_bound
        } else {
            range.clone()
        };

        Self {
            inner: FloatGene::from((range, bounds)),
        }
    }

    pub fn allele(&self) -> &f32 {
        self.inner.allele()
    }
}

impl AsRef<FloatGene> for PyFloatGene {
    fn as_ref(&self) -> &FloatGene {
        &self.inner
    }
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyFloatChromosome {
    pub inner: FloatChromosome,
}

#[pymethods]
impl PyFloatChromosome {
    #[new]
    #[pyo3(signature=(length=1, value_range=None, bound_range=None))]
    pub fn new(
        length: usize,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
    ) -> Self {
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0.0..1.0);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        Self {
            inner: FloatChromosome::from((length, val_range, bound_range)),
        }
    }
}

impl AsRef<FloatChromosome> for PyFloatChromosome {
    fn as_ref(&self) -> &FloatChromosome {
        &self.inner
    }
}
