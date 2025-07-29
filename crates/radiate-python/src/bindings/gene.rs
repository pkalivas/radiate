use crate::{AnyGene, any::AnyChromosome, prelude::Wrap};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, FloatChromosome, FloatGene, Gene,
    Genotype, GraphChromosome, GraphNode, IntChromosome, IntGene, Op, PermutationChromosome,
    PermutationGene, Phenotype, Population, TreeChromosome, TreeNode, random_provider,
};

pub const FLOAT_GENE_TYPE: &'static str = "FloatGene";
pub const INT_GENE_TYPE: &'static str = "IntGene";
pub const BIT_GENE_TYPE: &'static str = "BitGene";
pub const CHAR_GENE_TYPE: &'static str = "CharGene";
pub const GRAPH_GENE_TYPE: &'static str = "GraphNode";
pub const TREE_GENE_TYPE: &'static str = "TreeNode";
pub const ANY_GENE_TYPE: &'static str = "AnyGene";
pub const PERMUTATION_GENE_TYPE: &'static str = "PermutationGene";

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PyGeneType {
    Empty,
    Int,
    Float,
    Bit,
    Char,
    Graph,
    Tree,
    Any,
    Permutation,
}

#[pymethods]
impl PyGeneType {
    pub fn name(&self) -> String {
        match self {
            PyGeneType::Empty => "NONE".into(),
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
            PyGeneType::Graph => GRAPH_GENE_TYPE.into(),
            PyGeneType::Tree => TREE_GENE_TYPE.into(),
            PyGeneType::Any => ANY_GENE_TYPE.into(),
            PyGeneType::Permutation => PERMUTATION_GENE_TYPE.into(),
        }
    }

    pub fn __repr__(&self) -> String {
        match self {
            PyGeneType::Empty => "NONE".into(),
            PyGeneType::Int => INT_GENE_TYPE.into(),
            PyGeneType::Float => FLOAT_GENE_TYPE.into(),
            PyGeneType::Bit => BIT_GENE_TYPE.into(),
            PyGeneType::Char => CHAR_GENE_TYPE.into(),
            PyGeneType::Graph => GRAPH_GENE_TYPE.into(),
            PyGeneType::Tree => TREE_GENE_TYPE.into(),
            PyGeneType::Any => ANY_GENE_TYPE.into(),
            PyGeneType::Permutation => PERMUTATION_GENE_TYPE.into(),
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __hash__(&self) -> usize {
        match self {
            PyGeneType::Empty => 0,
            PyGeneType::Int => 1,
            PyGeneType::Float => 2,
            PyGeneType::Bit => 3,
            PyGeneType::Char => 4,
            PyGeneType::Graph => 5,
            PyGeneType::Tree => 6,
            PyGeneType::Any => 7,
            PyGeneType::Permutation => 8,
        }
    }

    pub fn __eq__(&self, other: &PyGeneType) -> bool {
        self == other
    }

    pub fn __ne__(&self, other: &PyGeneType) -> bool {
        self != other
    }
}

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyPopulation {
    #[pyo3(get)]
    phenotypes: Vec<PyPhenotype>,
}

#[pymethods]
impl PyPopulation {
    #[new]
    pub fn new(phenotypes: Vec<PyPhenotype>) -> Self {
        PyPopulation { phenotypes }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Population(phenotypes={:?})",
            self.phenotypes
                .iter()
                .map(|p| p.__repr__())
                .collect::<Vec<_>>()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        self.phenotypes.len()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.phenotypes
            .iter()
            .zip(&other.phenotypes)
            .all(|(a, b)| a == b)
    }

    pub fn gene_type(&self) -> PyGeneType {
        if self.phenotypes.is_empty() {
            PyGeneType::Empty
        } else {
            self.phenotypes[0].gene_type()
        }
    }
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPhenotype {
    #[pyo3(get)]
    genotype: PyGenotype,
    #[pyo3(get)]
    score: Vec<f32>,
    #[pyo3(get)]
    id: u64,
}

#[pymethods]
impl PyPhenotype {
    #[new]
    #[pyo3(signature = (genotype, score=None, id=0))]
    pub fn new(genotype: PyGenotype, score: Option<Vec<f32>>, id: u64) -> Self {
        PyPhenotype {
            genotype,
            score: score.unwrap_or_default(),
            id,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Phenotype(id={}, score={:?}, genotype={:?})",
            self.id,
            self.score,
            self.genotype.__repr__()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        self.genotype.chromosomes.len()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.genotype == other.genotype
    }

    pub fn __ne__(&self, other: &Self) -> bool {
        self.genotype != other.genotype
    }

    pub fn gene_type(&self) -> PyGeneType {
        if self.genotype.chromosomes.is_empty() {
            PyGeneType::Empty
        } else {
            self.genotype.gene_type()
        }
    }
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct PyGenotype {
    #[pyo3(get)]
    chromosomes: Vec<PyChromosome>,
}

#[pymethods]
impl PyGenotype {
    #[new]
    pub fn new(chromosomes: Vec<PyChromosome>) -> Self {
        PyGenotype { chromosomes }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "{:?}",
            self.chromosomes
                .iter()
                .map(|c| c.__repr__())
                .collect::<Vec<_>>()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        self.chromosomes.len()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.chromosomes
            .iter()
            .zip(&other.chromosomes)
            .all(|(a, b)| a == b)
    }

    pub fn __getitem__<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyAny>> {
        self.chromosomes
            .get(index)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("index out of range"))
            .and_then(|chromosome| chromosome.clone().into_bound_py_any(py))
    }

    pub fn gene_type(&self) -> PyGeneType {
        if self.chromosomes.is_empty() {
            PyGeneType::Empty
        } else {
            self.chromosomes[0].gene_type()
        }
    }
}

impl IntoIterator for PyGenotype {
    type Item = PyChromosome;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chromosomes.into_iter()
    }
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct PyChromosome {
    #[pyo3(get)]
    genes: Vec<PyGene>,
}

#[pymethods]
impl PyChromosome {
    #[new]
    pub fn new(genes: Vec<PyGene>) -> Self {
        PyChromosome { genes }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Chromosome(genes={:?})",
            self.genes.iter().map(|g| g.__repr__()).collect::<Vec<_>>()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __len__(&self) -> usize {
        self.genes.len()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.genes.iter().zip(&other.genes).all(|(a, b)| a == b)
    }

    pub fn __getitem__<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyAny>> {
        self.genes
            .get(index)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("index out of range"))
            .and_then(|gene| gene.clone().into_bound_py_any(py))
    }

    pub fn gene_type(&self) -> PyGeneType {
        if self.genes.is_empty() {
            PyGeneType::Empty
        } else {
            self.genes[0].gene_type()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum GeneInner {
    Float(FloatGene),
    Int(IntGene<i32>),
    Bit(BitGene),
    Char(CharGene),
    GraphNode(GraphNode<Op<f32>>),
    TreeNode(TreeNode<Op<f32>>),
    Any(AnyGene<'static>),
    Permutation(PermutationGene<usize>),
}

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct PyGene {
    inner: GeneInner,
}

#[pymethods]
impl PyGene {
    pub fn gene_type(&self) -> PyGeneType {
        match &self.inner {
            GeneInner::Float(_) => PyGeneType::Float,
            GeneInner::Int(_) => PyGeneType::Int,
            GeneInner::Bit(_) => PyGeneType::Bit,
            GeneInner::Char(_) => PyGeneType::Char,
            GeneInner::GraphNode(_) => PyGeneType::Graph,
            GeneInner::TreeNode(_) => PyGeneType::Tree,
            GeneInner::Any(_) => PyGeneType::Any,
            GeneInner::Permutation(_) => PyGeneType::Permutation,
        }
    }

    pub fn allele<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            GeneInner::Float(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Int(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Bit(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Char(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::GraphNode(gene) => gene.allele().name().into_bound_py_any(py),
            GeneInner::TreeNode(gene) => gene.allele().name().into_bound_py_any(py),
            GeneInner::Any(gene) => Wrap(gene.allele()).into_bound_py_any(py),
            GeneInner::Permutation(gene) => gene.allele().into_bound_py_any(py),
        }
    }

    pub fn __str__(&self) -> String {
        match &self.inner {
            GeneInner::Float(gene) => format!("{}", gene),
            GeneInner::Int(gene) => format!("{}", gene),
            GeneInner::Bit(gene) => format!("{}", gene),
            GeneInner::Char(gene) => format!("{:?}", gene),
            GeneInner::GraphNode(gene) => format!("{:?}", gene),
            GeneInner::TreeNode(gene) => format!("{:?}", gene),
            GeneInner::Any(gene) => format!("{:?}", gene),
            GeneInner::Permutation(gene) => format!("{:?}", gene),
        }
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
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
            inner: GeneInner::Bit(BitGene::from(allele.unwrap_or(random_provider::bool(0.5)))),
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

        impl From<PyGene> for $gene_type {
            fn from(py_gene: PyGene) -> Self {
                match py_gene.inner {
                    GeneInner::$gene_variant(gene) => gene,
                    _ => panic!("Cannot convert PyGene to {}", stringify!($gene_type)),
                }
            }
        }
    };
}

impl_into_py_gene!(FloatGene, Float);
impl_into_py_gene!(IntGene<i32>, Int);
impl_into_py_gene!(BitGene, Bit);
impl_into_py_gene!(CharGene, Char);
impl_into_py_gene!(GraphNode<Op<f32>>, GraphNode);
impl_into_py_gene!(TreeNode<Op<f32>>, TreeNode);
impl_into_py_gene!(AnyGene<'static>, Any);
impl_into_py_gene!(PermutationGene<usize>, Permutation);

macro_rules! impl_into_py_chromosome {
    ($chromosome_type:ty, $gene_type:ty) => {
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

        impl From<PyChromosome> for $chromosome_type {
            fn from(py_chromosome: PyChromosome) -> Self {
                let genes = py_chromosome
                    .genes
                    .into_iter()
                    .map(|gene| <$gene_type>::from(gene))
                    .collect::<Vec<_>>();
                <$chromosome_type>::from(genes)
            }
        }
    };
}

impl_into_py_chromosome!(FloatChromosome, FloatGene);
impl_into_py_chromosome!(IntChromosome<i32>, IntGene<i32>);
impl_into_py_chromosome!(BitChromosome, BitGene);
impl_into_py_chromosome!(CharChromosome, CharGene);
impl_into_py_chromosome!(GraphChromosome<Op<f32>>, GraphNode<Op<f32>>);
impl_into_py_chromosome!(TreeChromosome<Op<f32>>, TreeNode<Op<f32>>);
impl_into_py_chromosome!(AnyChromosome<'static>, AnyGene<'static>);
impl_into_py_chromosome!(PermutationChromosome<usize>, PermutationGene<usize>);

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

        impl From<PyGenotype> for Genotype<$chromosome>
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(py_genotype: PyGenotype) -> Self {
                let chromosomes = py_genotype
                    .chromosomes
                    .into_iter()
                    .map(|chromosome| <$chromosome>::from(chromosome))
                    .collect::<Vec<_>>();
                Genotype::from(chromosomes)
            }
        }
    };
}

impl_into_py_genotype!(FloatChromosome);
impl_into_py_genotype!(IntChromosome<i32>);
impl_into_py_genotype!(BitChromosome);
impl_into_py_genotype!(CharChromosome);
impl_into_py_genotype!(GraphChromosome<Op<f32>>);
impl_into_py_genotype!(TreeChromosome<Op<f32>>);
impl_into_py_genotype!(AnyChromosome<'static>);
impl_into_py_genotype!(PermutationChromosome<usize>);

macro_rules! impl_from_py_phenotype {
    ($chromosome:ty) => {
        impl From<Phenotype<$chromosome>> for PyPhenotype
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(phenotype: Phenotype<$chromosome>) -> Self {
                PyPhenotype {
                    genotype: PyGenotype::from(phenotype.genotype().clone()),
                    score: phenotype
                        .score()
                        .map(|score| score.as_ref().to_vec())
                        .unwrap_or_default(),
                    id: *phenotype.id(),
                }
            }
        }

        impl From<PyPhenotype> for Phenotype<$chromosome>
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(py_phenotype: PyPhenotype) -> Self {
                let mut result = Phenotype::from((Genotype::from(py_phenotype.genotype), 0));

                if !py_phenotype.score.is_empty() {
                    result.set_score(Some(py_phenotype.score.into()));
                }

                result
            }
        }
    };
}

impl_from_py_phenotype!(FloatChromosome);
impl_from_py_phenotype!(IntChromosome<i32>);
impl_from_py_phenotype!(BitChromosome);
impl_from_py_phenotype!(CharChromosome);
impl_from_py_phenotype!(GraphChromosome<Op<f32>>);
impl_from_py_phenotype!(TreeChromosome<Op<f32>>);
impl_from_py_phenotype!(AnyChromosome<'static>);
impl_from_py_phenotype!(PermutationChromosome<usize>);

macro_rules! impl_into_py_population {
    ($chromosome:ty) => {
        impl From<&Population<$chromosome>> for PyPopulation
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(population: &Population<$chromosome>) -> Self {
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
                            score: phenotype
                                .score()
                                .map(|score| score.as_ref().to_vec())
                                .unwrap_or_default(),
                            id: *phenotype.id(),
                        })
                        .collect(),
                }
            }
        }

        impl From<PyPopulation> for Population<$chromosome>
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(py_population: PyPopulation) -> Self {
                let phenotypes = py_population
                    .phenotypes
                    .into_iter()
                    .map(|py_phenotype| Phenotype::from(py_phenotype))
                    .collect::<Vec<_>>();
                Population::from(phenotypes)
            }
        }
    };
}

impl_into_py_population!(FloatChromosome);
impl_into_py_population!(IntChromosome<i32>);
impl_into_py_population!(BitChromosome);
impl_into_py_population!(CharChromosome);
impl_into_py_population!(GraphChromosome<Op<f32>>);
impl_into_py_population!(TreeChromosome<Op<f32>>);
impl_into_py_population!(AnyChromosome<'static>);
impl_into_py_population!(PermutationChromosome<usize>);
