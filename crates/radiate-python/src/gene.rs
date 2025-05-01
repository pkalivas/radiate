use std::any::Any;

use pyo3::{
    Bound, PyAny, pyclass, pymethods,
    types::{PyList, PySequence},
};
use radiate::{
    AnyChromosome, AnyGene, BitGene, CharGene, FloatGene, Gene, Genotype, IntGene, object::AnyValue,
};

use crate::conversion::{ObjectValue, Wrap, py_object_to_any_value};

#[derive(Clone, Debug)]
pub enum GeneType {
    Float,
    Int,
    Char,
    Bit,
}

impl From<String> for GeneType {
    fn from(gene_type: String) -> Self {
        match gene_type.to_lowercase().trim() {
            "float" => GeneType::Float,
            "int" => GeneType::Int,
            "char" => GeneType::Char,
            "bit" => GeneType::Bit,
            _ => panic!("Invalid gene type"),
        }
    }
}

impl From<GeneType> for String {
    fn from(gene_type: GeneType) -> Self {
        match gene_type {
            GeneType::Float => "float".to_string(),
            GeneType::Int => "int".to_string(),
            GeneType::Char => "char".to_string(),
            GeneType::Bit => "bit".to_string(),
        }
    }
}

#[pyclass(name = "FloatGene")]
#[derive(Clone, Debug)]
pub struct PyFloatGene {
    pub inner: FloatGene,
}

#[pymethods]
impl PyFloatGene {
    #[new]
    #[pyo3(signature = (range, bounds=None))]
    pub fn new(range: (f32, f32), bounds: Option<(f32, f32)>) -> Self {
        let inner = if let Some(bounds) = bounds {
            FloatGene::from((range.0..range.1, bounds.0..bounds.1))
        } else {
            FloatGene::from(range.0..range.1)
        };
        Self { inner }
    }

    pub fn allele(&self) -> &f32 {
        self.inner.allele()
    }
}

impl From<FloatGene> for PyFloatGene {
    fn from(gene: FloatGene) -> Self {
        Self { inner: gene }
    }
}

impl From<PyFloatGene> for FloatGene {
    fn from(gene: PyFloatGene) -> Self {
        gene.inner
    }
}

#[pyclass(name = "IntGene")]
#[derive(Clone, Debug)]
pub struct PyIntGene {
    pub inner: IntGene<i32>,
}

#[pymethods]
impl PyIntGene {
    #[new]
    #[pyo3(signature = (range, bounds=None))]
    pub fn new(range: (i32, i32), bounds: Option<(i32, i32)>) -> Self {
        Self {
            inner: if let Some(bounds) = bounds {
                IntGene::from((range.0..range.1, bounds.0..bounds.1))
            } else {
                IntGene::from(range.0..range.1)
            },
        }
    }

    pub fn allele(&self) -> &i32 {
        self.inner.allele()
    }
}

impl From<IntGene<i32>> for PyIntGene {
    fn from(gene: IntGene<i32>) -> Self {
        Self { inner: gene }
    }
}

impl From<PyIntGene> for IntGene<i32> {
    fn from(gene: PyIntGene) -> Self {
        gene.inner
    }
}

#[pyclass(name = "CharGene")]
#[derive(Clone, Debug)]
pub struct PyCharGene {
    pub inner: CharGene,
}

#[pymethods]
impl PyCharGene {
    #[new]
    #[pyo3(signature = (allele, char_set=None))]
    pub fn new(allele: char, char_set: Option<String>) -> Self {
        Self {
            inner: if let Some(char_set) = char_set {
                CharGene::from((allele, char_set.chars().collect::<Vec<char>>().into()))
            } else {
                CharGene::from(allele)
            },
        }
    }

    pub fn allele(&self) -> &char {
        self.inner.allele()
    }
}

impl From<CharGene> for PyCharGene {
    fn from(gene: CharGene) -> Self {
        Self { inner: gene }
    }
}

impl From<PyCharGene> for CharGene {
    fn from(gene: PyCharGene) -> Self {
        gene.inner
    }
}

#[pyclass(name = "BitGene")]
#[derive(Clone, Debug)]
pub struct PyBitGene {
    pub inner: BitGene,
}

#[pymethods]
impl PyBitGene {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: BitGene::new(),
        }
    }

    pub fn allele(&self) -> &bool {
        self.inner.allele()
    }
}

impl From<BitGene> for PyBitGene {
    fn from(gene: BitGene) -> Self {
        Self { inner: gene }
    }
}

impl From<PyBitGene> for BitGene {
    fn from(gene: PyBitGene) -> Self {
        gene.inner
    }
}

pub enum InnerGene {
    Float(FloatGene),
    Int(IntGene<i32>),
    Char(CharGene),
    Bit(BitGene),
    Any(AnyGene<'static>),
    // Custom(String, Box<dyn Gene>),
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyGene {
    inner: AnyGene<'static>,
}

#[pymethods]
impl PyGene {
    #[new]
    #[pyo3(signature = (allele=None))]
    pub fn new(allele: Option<Wrap<AnyValue<'_>>>) -> Self {
        let inner = if let Some(allele) = allele.as_ref() {
            AnyGene::new(allele.0.clone().into_static())
        } else {
            AnyGene::new(AnyValue::Null)
        };
        Self { inner }
    }
}

#[pyclass(name = "Chromosome")]
#[derive(Clone, Debug)]
pub struct PyChromosome {
    inner: AnyChromosome<'static>,
}

#[pymethods]
impl PyChromosome {
    #[new]
    #[pyo3(signature = (genes))]
    pub fn new(genes: Vec<Wrap<AnyValue<'_>>>) -> Self {
        let inner = AnyChromosome::new(
            genes
                .into_iter()
                .map(|gene| gene.0.clone().into_static())
                .collect::<Vec<_>>(),
        );
        Self { inner }
    }

    pub fn __str__(&self) -> String {
        self.inner
            .as_ref()
            .iter()
            .map(|gene| gene.allele().type_name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[pyclass(name = "Geneotype")]
#[derive(Clone, Debug)]
pub struct PyGeneotype {
    inner: Vec<AnyChromosome<'static>>,
}

#[pymethods]
impl PyGeneotype {
    #[new]
    #[pyo3(signature = (chromosomes=None ))]
    pub fn new(chromosomes: Option<Vec<PyChromosome>>) -> Self {
        Self {
            inner: chromosomes
                .map(|chromosomes| {
                    chromosomes
                        .into_iter()
                        .map(|chromosome| chromosome.inner)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        }
    }
}

impl PyGeneotype {
    pub fn take(self) -> Vec<AnyChromosome<'static>> {
        self.inner
    }
}
