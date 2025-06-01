use pyo3::{
    Borrowed, Bound, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass,
    pymethods,
    types::{PyList, PyString},
};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, FloatChromosome, FloatGene,
    Genotype, IntChromosome, IntGene,
};

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyPopulation {
    phenotypes: Vec<PyPhenotype>,
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPhenotype {
    genotype: PyGenotype,
    score: Vec<f32>,
    id: u64,
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyGenotype {
    chromosomes: Vec<PyChromosome>,
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyChromosome {
    genes: Vec<PyGene>,
}

#[derive(Clone, Debug)]
enum GeneInner {
    Float(FloatGene),
    Int(IntGene<i32>),
    Bit(BitGene),
    Char(CharGene),
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyGene {
    inner: GeneInner,
}

#[pymethods]
impl PyGene {
    pub fn gene_type(&self) -> String {
        match &self.inner {
            GeneInner::Float(_) => "FloatGene".to_string(),
            GeneInner::Int(_) => "IntGene".to_string(),
            GeneInner::Bit(_) => "BitGene".to_string(),
            GeneInner::Char(_) => "CharGene".to_string(),
        }
    }

    pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let repr = format!(
            "{}",
            match &self.inner {
                GeneInner::Float(gene) => format!("{:?}", gene),
                GeneInner::Int(gene) => format!("{:?}", gene),
                GeneInner::Bit(gene) => format!("{:?}", gene),
                GeneInner::Char(gene) => format!("{:?}", gene),
            }
        );

        PyString::new(py, &repr).into_bound_py_any(py)
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.__str__(py)
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, range=None, bounds=None))]
    pub fn float(
        allele: Option<f32>,
        range: Option<(f32, f32)>,
        bounds: Option<(f32, f32)>,
    ) -> PyGene {
        let range = range.unwrap_or((std::f32::MIN, std::f32::MAX));
        let bounds = bounds.unwrap_or(range.clone());

        PyGene {
            inner: GeneInner::Float(match allele {
                Some(a) => FloatGene::new(a, range.0..range.1, bounds.0..bounds.1),
                None => FloatGene::from((range.0..range.1, bounds.0..bounds.1)),
            }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, range=None, bounds=None))]
    pub fn int(
        allele: Option<i32>,
        range: Option<(i32, i32)>,
        bounds: Option<(i32, i32)>,
    ) -> PyGene {
        let range = range.unwrap_or((i32::MIN, i32::MAX));
        let bounds = bounds.unwrap_or(range.clone());

        PyGene {
            inner: GeneInner::Int(match allele {
                Some(a) => IntGene::new(a, range.0..range.1, bounds.0..bounds.1),
                None => IntGene::from((range.0..range.1, bounds.0..bounds.1)),
            }),
        }
    }

    #[staticmethod]
    pub fn bit(allele: Option<bool>) -> PyGene {
        PyGene {
            inner: GeneInner::Bit(BitGene::from(allele.unwrap_or(false))),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (allele=None, char_set=None))]
    pub fn char(allele: Option<char>, char_set: Option<Vec<char>>) -> PyGene {
        PyGene {
            inner: GeneInner::Char(match char_set {
                Some(chars) => match allele {
                    Some(a) => CharGene::from((a, chars.into_iter().collect())),
                    None => CharGene::new(chars.into_iter().collect()),
                },
                None => match allele {
                    Some(a) => CharGene::from(a),
                    None => CharGene::default(),
                },
            }),
        }
    }
}

macro_rules! impl_into_py_gene {
    ($gene_type:ty, $gene_variant:ident) => {
        impl From<$gene_type> for PyGene {
            fn from(gene: $gene_type) -> Self {
                PyGene {
                    inner: GeneInner::$gene_variant(gene),
                }
            }
        }
    };
}

impl_into_py_gene!(FloatGene, Float);
impl_into_py_gene!(IntGene<i32>, Int);
impl_into_py_gene!(BitGene, Bit);
impl_into_py_gene!(CharGene, Char);

macro_rules! impl_into_py_chromosome {
    ($chromosome_type:ty, $chromosome_variant:ident) => {
        impl From<$chromosome_type> for PyChromosome {
            fn from(chromosome: $chromosome_type) -> Self {
                PyChromosome {
                    genes: chromosome
                        .genes()
                        .iter()
                        .map(|gene| PyGene::from(gene.clone()))
                        .collect(),
                }
            }
        }
    };
}

impl_into_py_chromosome!(FloatChromosome, Float);
impl_into_py_chromosome!(IntChromosome<i32>, Int);
impl_into_py_chromosome!(BitChromosome, Bit);
impl_into_py_chromosome!(CharChromosome, Char);

macro_rules! impl_into_py_genotype {
    ($genotype_type:ty) => {
        impl From<$genotype_type> for PyGenotype {
            fn from(genotype: $genotype_type) -> Self {
                PyGenotype {
                    chromosomes: genotype
                        .iter()
                        .map(|chromosome| PyChromosome {
                            genes: chromosome
                                .genes()
                                .iter()
                                .map(|gene| PyGene::from(gene.clone()))
                                .collect(),
                        })
                        .collect(),
                }
            }
        }
    };
}

impl_into_py_genotype!(Genotype<FloatChromosome>);
impl_into_py_genotype!(Genotype<IntChromosome<i32>>);
impl_into_py_genotype!(Genotype<BitChromosome>);
impl_into_py_genotype!(Genotype<CharChromosome>);

macro_rules! impl_into_py_population {
    ($population_type:ty, $population_variant:ident) => {
        impl From<$population_type> for PyPopulation {
            fn from(population: $population_type) -> Self {
                PyPopulation {
                    phenotypes: population
                        .iter()
                        .map(|phenotype| PyPhenotype {
                            genotype: PyGenotype {
                                chromosomes: phenotype
                                    .genotype()
                                    .iter()
                                    .map(|chromosome| PyChromosome {
                                        genes: chromosome
                                            .genes()
                                            .iter()
                                            .map(|gene| PyGene::from(gene.clone()))
                                            .collect(),
                                    })
                                    .collect(),
                            },
                            score: phenotype.score().unwrap().as_ref().to_vec(),
                            id: *phenotype.id(),
                        })
                        .collect(),
                }
            }
        }
    };
}

impl_into_py_population!(radiate::Population<FloatChromosome>, Float);
impl_into_py_population!(radiate::Population<IntChromosome<i32>>, Int);
impl_into_py_population!(radiate::Population<BitChromosome>, Bit);
impl_into_py_population!(radiate::Population<CharChromosome>, Char);
