use crate::{AnyChromosome, PyGeneType, PyGenotype, PyPhenotype};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, FloatChromosome, GraphChromosome, IntChromosome, Op,
    PermutationChromosome, Phenotype, Population, TreeChromosome,
};

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPopulation {
    #[pyo3(get)]
    pub(crate) phenotypes: Vec<PyPhenotype>,
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
                            genotype: PyGenotype::from(phenotype.genotype().clone()),
                            score: phenotype
                                .score()
                                .map(|score| score.as_ref().to_vec())
                                .unwrap_or_default(),
                            id: phenotype.id().0,
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
impl_into_py_population!(AnyChromosome<'static>);
