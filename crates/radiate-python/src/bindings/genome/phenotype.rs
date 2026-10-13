use crate::{PyGeneType, PyGenotype, Wrap};
use pyo3::{Bound, IntoPyObject, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, FloatChromosome, Genotype, GraphChromosome,
    IntChromosome, Op, PackedBitChromosome, PermutationChromosome, Phenotype, TreeChromosome,
    phenotype::PhenotypeId,
};
use radiate_utils::DataType;

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPhenotype {
    #[pyo3(get)]
    pub(crate) genotype: PyGenotype,
    #[pyo3(get)]
    pub(crate) score: Vec<f32>,
    #[pyo3(get)]
    pub(crate) generation: usize,
    #[pyo3(get)]
    pub(crate) id: u64,
    #[pyo3(get)]
    pub(crate) species: u64,
}

#[pymethods]
impl PyPhenotype {
    #[new]
    #[pyo3(signature = (genotype, score=None, id=None))]
    pub fn new(genotype: PyGenotype, score: Option<Vec<f32>>, id: Option<u64>) -> Self {
        PyPhenotype {
            genotype,
            score: score.unwrap_or_default(),
            id: id.unwrap_or(PhenotypeId::new().into()),
            generation: 0,
            species: 0,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Phenotype(id={}, score={:?}, genotype={})",
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

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if self.genotype.chromosomes.is_empty() {
            Wrap(DataType::Null).into_pyobject(py)
        } else {
            self.genotype.chromosomes[0].dtype(py)
        }
    }
}

macro_rules! impl_from_py_phenotype {
    ($chromosome:ty) => {
        impl From<Phenotype<$chromosome>> for PyPhenotype
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(mut phenotype: Phenotype<$chromosome>) -> Self {
                let score = phenotype
                    .score()
                    .map(|score| score.as_ref().to_vec())
                    .unwrap_or_default();
                let id = phenotype.id().into();
                let generation = phenotype.generation();
                let species = phenotype.species().into();
                PyPhenotype {
                    genotype: PyGenotype::from(phenotype.take_genotype().unwrap()),
                    score,
                    id,
                    generation,
                    species,
                }
            }
        }

        impl From<PyPhenotype> for Phenotype<$chromosome>
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(py_phenotype: PyPhenotype) -> Self {
                let mut result = Phenotype::from((
                    Genotype::from(py_phenotype.genotype),
                    py_phenotype.generation,
                ));

                if !py_phenotype.score.is_empty() {
                    result.set_score(Some(py_phenotype.score.into()));
                }

                result
            }
        }
    };
}

impl_from_py_phenotype!(IntChromosome<u8>);
impl_from_py_phenotype!(IntChromosome<u16>);
impl_from_py_phenotype!(IntChromosome<u32>);
impl_from_py_phenotype!(IntChromosome<u64>);
impl_from_py_phenotype!(IntChromosome<u128>);

impl_from_py_phenotype!(IntChromosome<i8>);
impl_from_py_phenotype!(IntChromosome<i16>);
impl_from_py_phenotype!(IntChromosome<i32>);
impl_from_py_phenotype!(IntChromosome<i64>);
impl_from_py_phenotype!(IntChromosome<i128>);

impl_from_py_phenotype!(FloatChromosome<f32>);
impl_from_py_phenotype!(FloatChromosome<f64>);

impl_from_py_phenotype!(BitChromosome);
impl_from_py_phenotype!(PackedBitChromosome);
impl_from_py_phenotype!(CharChromosome);
impl_from_py_phenotype!(PermutationChromosome<usize>);

impl_from_py_phenotype!(GraphChromosome<Op<f32>>);
impl_from_py_phenotype!(GraphChromosome<Op<f64>>);

impl_from_py_phenotype!(TreeChromosome<Op<f32>>);
impl_from_py_phenotype!(TreeChromosome<Op<f64>>);
