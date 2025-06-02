use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods, types::PyString};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, FloatChromosome, FloatGene, Gene,
    Genotype, IntChromosome, IntGene, Phenotype, Population,
};

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyPopulation {
    #[pyo3(get)]
    phenotypes: Vec<PyPhenotype>,
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPhenotype {
    #[pyo3(get)]
    genotype: PyGenotype,
    #[pyo3(get)]
    score: Vec<f32>,
    #[pyo3(get)]
    id: u64,
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyGenotype {
    #[pyo3(get)]
    chromosomes: Vec<PyChromosome>,
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyChromosome {
    #[pyo3(get)]
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

    pub fn allele<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            GeneInner::Float(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Int(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Bit(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Char(gene) => gene.allele().into_bound_py_any(py),
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
    ($chromosome:ty) => {
        impl From<Genotype<$chromosome>> for PyGenotype
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(genotype: Genotype<$chromosome>) -> Self {
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

impl_into_py_genotype!(FloatChromosome);
impl_into_py_genotype!(IntChromosome<i32>);
impl_into_py_genotype!(BitChromosome);
impl_into_py_genotype!(CharChromosome);

macro_rules! impl_from_py_phenotype {
    ($chromosome:ty) => {
        impl From<Phenotype<$chromosome>> for PyPhenotype
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(phenotype: Phenotype<$chromosome>) -> Self {
                PyPhenotype {
                    genotype: PyGenotype::from(phenotype.genotype().clone()),
                    score: phenotype.score().unwrap().as_ref().to_vec(),
                    id: *phenotype.id(),
                }
            }
        }
    };
}

impl_from_py_phenotype!(FloatChromosome);
impl_from_py_phenotype!(IntChromosome<i32>);
impl_from_py_phenotype!(BitChromosome);
impl_from_py_phenotype!(CharChromosome);

macro_rules! impl_into_py_population {
    ($chromosome:ty) => {
        impl From<Population<$chromosome>> for PyPopulation
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(population: Population<$chromosome>) -> Self {
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

impl_into_py_population!(FloatChromosome);
impl_into_py_population!(IntChromosome<i32>);
impl_into_py_population!(BitChromosome);
impl_into_py_population!(CharChromosome);
