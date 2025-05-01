use crate::conversion::Wrap;
use pyo3::{pyclass, pymethods};
use radiate::{
    AnyChromosome, BitChromosome, BitGene, CharChromosome, CharGene, FloatChromosome,
    IntChromosome, object::AnyValue,
};

#[pyclass(name = "FloatChromosome")]
pub struct PyFloatChromosome {
    inner: FloatChromosome,
}

#[pymethods]
impl PyFloatChromosome {
    #[new]
    #[pyo3(signature = (num_genes, range=None, bounds=None))]
    pub fn new(num_genes: usize, range: Option<(f32, f32)>, bounds: Option<(f32, f32)>) -> Self {
        let chromosome = if let Some(range) = range {
            if let Some(bounds) = bounds {
                FloatChromosome::from((num_genes, range.0..range.1, bounds.0..bounds.1))
            } else {
                FloatChromosome::from((num_genes, range.0..range.1))
            }
        } else {
            FloatChromosome::from((num_genes, -1.0..1.0))
        };

        PyFloatChromosome { inner: chromosome }
    }
}

#[pyclass(name = "IntChromosome")]
pub struct PyIntChromosome {
    inner: IntChromosome<i32>,
}

#[pymethods]
impl PyIntChromosome {
    #[new]
    #[pyo3(signature = (num_genes, range=None, bounds=None))]
    pub fn new(num_genes: usize, range: Option<(i32, i32)>, bounds: Option<(i32, i32)>) -> Self {
        let chromosome = if let Some(range) = range {
            if let Some(bounds) = bounds {
                IntChromosome::from((num_genes, range.0..range.1, bounds.0..bounds.1))
            } else {
                IntChromosome::from((num_genes, range.0..range.1))
            }
        } else {
            IntChromosome::from((num_genes, -100..100))
        };

        PyIntChromosome { inner: chromosome }
    }
}

#[pyclass(name = "BitChromosome")]
pub struct PyBitChromosome {
    inner: BitChromosome,
}

#[pymethods]
impl PyBitChromosome {
    #[new]
    pub fn new(num_genes: usize) -> Self {
        let chromosome = BitChromosome {
            genes: (0..num_genes).map(|_| BitGene::new()).collect(),
        };

        PyBitChromosome { inner: chromosome }
    }
}

#[pyclass(name = "CharChromosome")]
pub struct PyCharChromosome {
    inner: CharChromosome,
}

#[pymethods]
impl PyCharChromosome {
    #[new]
    #[pyo3(signature = (num_genes, char_set=None))]
    pub fn new(num_genes: usize, char_set: Option<String>) -> Self {
        let chromosome = if let Some(char_set) = char_set {
            CharChromosome {
                genes: (0..num_genes)
                    .map(|_| CharGene::new(char_set.chars().collect::<Vec<char>>().into()))
                    .collect(),
            }
        } else {
            CharChromosome {
                genes: (0..num_genes).map(|_| CharGene::default()).collect(),
            }
        };

        PyCharChromosome { inner: chromosome }
    }
}

#[pyclass(name = "AnyChromosome")]
pub struct PyAnyChromosome {
    inner: AnyChromosome<'static>,
}

#[pymethods]
impl PyAnyChromosome {
    #[new]
    #[pyo3(signature = (allele))]
    pub fn new(allele: Vec<Option<Wrap<AnyValue<'_>>>>) -> Self {
        let chromosome = AnyChromosome::new(
            allele
                .iter()
                .map(|val| {
                    if let Some(val) = val {
                        val.0.clone().into_static()
                    } else {
                        AnyValue::Null
                    }
                })
                .collect(),
        );

        PyAnyChromosome { inner: chromosome }
    }
}
