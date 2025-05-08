use crate::{AnyValue, conversion::Wrap};
use pyo3::{pyclass, pymethods};
use radiate::{Gene, Valid};

#[pyclass(name = "Gene")]
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
pub struct PyGene {
    pub allele: AnyValue<'static>,
}

#[pymethods]
impl PyGene {
    #[new]
    #[pyo3(signature = (allele))]
    pub fn new(allele: Wrap<AnyValue<'_>>) -> Self {
        Self {
            allele: allele.0.into_static(),
        }
    }
}

impl Valid for PyGene {
    fn is_valid(&self) -> bool {
        true
    }
}

impl Gene for PyGene {
    type Allele = AnyValue<'static>;

    fn allele(&self) -> &Self::Allele {
        &self.allele
    }

    fn new_instance(&self) -> Self {
        Self {
            allele: self.allele.clone(),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        Self {
            allele: allele.clone(),
        }
    }
}
