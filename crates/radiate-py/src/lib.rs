use pyo3::prelude::*;
use radiate::{ArithmeticGene, FloatGene, Gene};

#[pyclass(name = "FloatGene")]
#[repr(transparent)]
pub struct PyFloatGene {
    pub gene: FloatGene,
}

#[pymethods]
impl PyFloatGene {
    #[new]
    pub fn new(min: f32, max: f32) -> Self {
        PyFloatGene {
            gene: FloatGene::from(min..max),
        }
    }

    pub fn min(&self) -> f32 {
        *self.gene.min()
    }

    pub fn max(&self) -> f32 {
        *self.gene.max()
    }

    pub fn allele(&self) -> f32 {
        *self.gene.allele()
    }
}

impl From<FloatGene> for PyFloatGene {
    fn from(gene: FloatGene) -> Self {
        PyFloatGene { gene }
    }
}
