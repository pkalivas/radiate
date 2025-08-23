use crate::Wrap;
use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, Chromosome, Ecosystem, FloatChromosome,
    FloatGene, Gene, Genotype, GraphChromosome, GraphNode, IntChromosome, IntGene, NodeType, Op,
    PermutationChromosome, PermutationGene, Phenotype, Population, Species, TreeChromosome,
    TreeNode, random_provider,
};
use std::sync::Arc;

pub const FLOAT_GENE_TYPE: &'static str = "FloatGene";
pub const INT_GENE_TYPE: &'static str = "IntGene";
pub const BIT_GENE_TYPE: &'static str = "BitGene";
pub const CHAR_GENE_TYPE: &'static str = "CharGene";
pub const GRAPH_GENE_TYPE: &'static str = "GraphNode";
pub const TREE_GENE_TYPE: &'static str = "TreeNode";
pub const PERMUTATION_GENE_TYPE: &'static str = "PermutationGene";

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PyGeneType {
    Empty,
    Int,
    Float,
    Bit,
    Char,
    GraphNode,
    TreeNode,
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
            PyGeneType::GraphNode => GRAPH_GENE_TYPE.into(),
            PyGeneType::TreeNode => TREE_GENE_TYPE.into(),
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
            PyGeneType::GraphNode => GRAPH_GENE_TYPE.into(),
            PyGeneType::TreeNode => TREE_GENE_TYPE.into(),
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
            PyGeneType::GraphNode => 5,
            PyGeneType::TreeNode => 6,
            PyGeneType::Permutation => 7,
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
pub struct PyEcosystem {
    #[pyo3(get)]
    population: PyPopulation,
    #[pyo3(get)]
    species: Vec<PySpecies>,
}

#[pymethods]
impl PyEcosystem {
    #[new]
    #[pyo3(signature = (population, species=None))]
    pub fn new(population: PyPopulation, species: Option<Vec<PySpecies>>) -> Self {
        PyEcosystem {
            population,
            species: species.unwrap_or_default(),
        }
    }

    pub fn __repr__(&self) -> String {
        let mut result = String::new();
        result.push_str("Ecosystem(\n");
        result.push_str(&format!("  Population: {:?}\n", self.population));

        if self.species.is_empty() {
            result.push_str("  Species: []\n");
        } else {
            result.push_str("  Species: [\n");
            for species in &self.species {
                result.push_str(&format!("    {:?},\n", species));
            }
            result.push_str("  ]\n");
        }
        result.push_str(")\n");
        result
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PySpecies {
    #[pyo3(get)]
    id: u64,
    #[pyo3(get)]
    mascot: PyPhenotype,
    #[pyo3(get)]
    generation: usize,
    #[pyo3(get)]
    stagnation: usize,
    #[pyo3(get)]
    population: PyPopulation,
    #[pyo3(get)]
    score: Option<Vec<f32>>,
}

#[pymethods]
impl PySpecies {
    #[new]
    pub fn new(
        id: u64,
        mascot: PyPhenotype,
        generation: usize,
        stagnation: usize,
        population: PyPopulation,
        score: Option<Vec<f32>>,
    ) -> Self {
        PySpecies {
            id,
            mascot,
            generation,
            stagnation,
            population,
            score,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Species(id={}, generation={}, stagnation={}, score={:?}, mascot={:?}, population_size={})",
            self.id,
            self.generation,
            self.stagnation,
            self.score,
            self.mascot,
            self.population.__len__()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn mascot(&self) -> PyPhenotype {
        self.mascot.clone()
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn stagnation(&self) -> usize {
        self.stagnation
    }

    pub fn population(&self) -> PyPopulation {
        self.population.clone()
    }

    pub fn score(&self) -> Option<Vec<f32>> {
        self.score.clone()
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
        let mut result = String::new();
        result.push_str("Population(\n");
        for phenotype in &self.phenotypes {
            result.push_str(&format!("  {:?},\n", phenotype));
        }
        result.push_str(")");
        result
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

    pub fn __getitem__<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyAny>> {
        self.phenotypes
            .get(index)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("index out of range"))
            .and_then(|phenotype| phenotype.clone().into_bound_py_any(py))
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
            "{:?}",
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
    Permutation(PermutationGene<usize>),
    Empty(PyGeneType),
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
            GeneInner::GraphNode(_) => PyGeneType::GraphNode,
            GeneInner::TreeNode(_) => PyGeneType::TreeNode,
            GeneInner::Permutation(_) => PyGeneType::Permutation,
            GeneInner::Empty(gene_type) => gene_type.clone(),
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
            GeneInner::Permutation(gene) => gene.allele().into_bound_py_any(py),
            GeneInner::Empty(_) => py.None().into_bound_py_any(py),
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
            GeneInner::Permutation(gene) => format!("{:?}", gene),
            GeneInner::Empty(gene_type) => format!("Empty({})", gene_type.name()),
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

    #[staticmethod]
    pub fn permutation(allele: Option<Vec<usize>>, index: usize) -> PyGene {
        let allele = allele.map(|a| a.into_iter().collect::<Arc<[usize]>>());
        if let Some(allele) = allele {
            PyGene {
                inner: GeneInner::Permutation(PermutationGene::new(index, allele)),
            }
        } else {
            PyGene {
                inner: GeneInner::Empty(PyGeneType::Permutation),
            }
        }
    }

    #[staticmethod]
    pub fn empty(gene_type: PyGeneType) -> PyGene {
        PyGene {
            inner: GeneInner::Empty(gene_type),
        }
    }

    #[staticmethod]
    pub fn graph_gene<'py>(
        py: Python<'py>,
        index: usize,
        allele: Py<PyAny>,
        node_type: String,
    ) -> PyGene {
        if let Ok(op) = allele.extract::<Wrap<Op<f32>>>(py) {
            let node_type = match node_type.as_str() {
                "input" => NodeType::Input,
                "output" => NodeType::Output,
                "vertex" => NodeType::Vertex,
                "edge" => NodeType::Edge,
                _ => panic!("Unknown node type: {}", node_type),
            };
            PyGene {
                inner: GeneInner::GraphNode(GraphNode::new(index, node_type, op.0)),
            }
        } else {
            panic!("Invalid allele type for GraphNode gene");
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
impl_into_py_population!(PermutationChromosome<usize>);

macro_rules! impl_into_py_species {
    ($chromosome:ty) => {
        impl From<Species<$chromosome>> for PySpecies
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(species: Species<$chromosome>) -> Self {
                PySpecies {
                    id: *(species.id()),
                    mascot: PyPhenotype::from(species.mascot().clone()),
                    generation: species.generation(),
                    stagnation: species.stagnation(),
                    population: PyPopulation::from(species.population()),
                    score: species.score().map(|s| s.as_ref().to_vec()),
                }
            }
        }

        impl From<PySpecies> for Species<$chromosome>
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(py_species: PySpecies) -> Self {
                let mascot = Phenotype::from(py_species.mascot);
                let population = Population::from(py_species.population);

                let mut species = Species::new(py_species.generation, &mascot);

                for individual in population.iter() {
                    species.push(individual.clone());
                }

                species
            }
        }
    };
}

impl_into_py_species!(FloatChromosome);
impl_into_py_species!(IntChromosome<i32>);
impl_into_py_species!(BitChromosome);
impl_into_py_species!(CharChromosome);
impl_into_py_species!(GraphChromosome<Op<f32>>);
impl_into_py_species!(TreeChromosome<Op<f32>>);
impl_into_py_species!(PermutationChromosome<usize>);

macro_rules! impl_into_py_ecosystem {
    ($chromosome:ty) => {
        impl From<Ecosystem<$chromosome>> for PyEcosystem
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(ecosystem: Ecosystem<$chromosome>) -> Self {
                PyEcosystem {
                    population: PyPopulation::from(ecosystem.population()),
                    species: ecosystem
                        .species()
                        .map(|s| s.iter().cloned().map(PySpecies::from).collect())
                        .unwrap_or_default(),
                }
            }
        }

        impl From<PyEcosystem> for Ecosystem<$chromosome>
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(py_ecosystem: PyEcosystem) -> Self {
                let population = Population::from(py_ecosystem.population);
                let species = if !py_ecosystem.species.is_empty() {
                    Some(
                        py_ecosystem
                            .species
                            .into_iter()
                            .map(|s| Species::<$chromosome>::from(s))
                            .collect::<Vec<Species<$chromosome>>>(),
                    )
                } else {
                    None
                };

                let mut ecosystem = Ecosystem::new(population);
                if let Some(species_list) = species {
                    for spec in species_list {
                        ecosystem.push_species(spec);
                    }
                }

                ecosystem
            }
        }
    };
}

impl_into_py_ecosystem!(FloatChromosome);
impl_into_py_ecosystem!(IntChromosome<i32>);
impl_into_py_ecosystem!(BitChromosome);
impl_into_py_ecosystem!(CharChromosome);
impl_into_py_ecosystem!(GraphChromosome<Op<f32>>);
impl_into_py_ecosystem!(TreeChromosome<Op<f32>>);
impl_into_py_ecosystem!(PermutationChromosome<usize>);
