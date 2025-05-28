use pyo3::{pyclass, pymethods};
use radiate::{Gene, IntChromosome, IntGene};

#[pyclass(name = "IntGene")]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyIntGene {
    pub inner: IntGene<i32>,
}

#[pymethods]
impl PyIntGene {
    #[new]
    #[pyo3(signature=(min=0, max=100, min_bound=None, max_bound=None))]
    pub fn new(min: i32, max: i32, min_bound: Option<i32>, max_bound: Option<i32>) -> Self {
        let range = min..max;
        let bounds = if let (Some(min_bound), Some(max_bound)) = (min_bound, max_bound) {
            min_bound..max_bound
        } else {
            range.clone()
        };

        Self {
            inner: IntGene::from((range, bounds)),
        }
    }

    pub fn allele(&self) -> &i32 {
        self.inner.allele()
    }
}

impl AsRef<IntGene<i32>> for PyIntGene {
    fn as_ref(&self) -> &IntGene<i32> {
        &self.inner
    }
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyIntChromosome {
    pub inner: IntChromosome<i32>,
}

#[pymethods]
impl PyIntChromosome {
    #[new]
    #[pyo3(signature=(length=1, value_range=None, bound_range=None))]
    pub fn new(
        length: usize,
        value_range: Option<(i32, i32)>,
        bound_range: Option<(i32, i32)>,
    ) -> Self {
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0..1);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        Self {
            inner: IntChromosome::from((length, val_range, bound_range)),
        }
    }
}

impl AsRef<IntChromosome<i32>> for PyIntChromosome {
    fn as_ref(&self) -> &IntChromosome<i32> {
        &self.inner
    }
}
