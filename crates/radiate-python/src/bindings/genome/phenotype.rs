use crate::{AnyChromosome, PyGeneType, PyGenotype};
use pyo3::{pyclass, pymethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, FloatChromosome, Genotype, GraphChromosome,
    IntChromosome, Op, PermutationChromosome, Phenotype, TreeChromosome,
};

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPhenotype {
    #[pyo3(get)]
    pub(crate) genotype: PyGenotype,
    #[pyo3(get)]
    pub(crate) score: Vec<f32>,
    #[pyo3(get)]
    pub(crate) id: u64,
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
                    id: phenotype.id().0,
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
impl_from_py_phenotype!(AnyChromosome<'static>);
