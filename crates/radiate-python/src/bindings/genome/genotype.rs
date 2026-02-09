use crate::{AnyChromosome, PyChromosome, PyGeneType, Wrap};

use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, DataType, FloatChromosome, Genotype,
    GraphChromosome, IntChromosome, Op, PermutationChromosome, TreeChromosome,
};

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyGenotype {
    #[pyo3(get)]
    pub(crate) chromosomes: Vec<PyChromosome>,
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

    pub fn chromosomes(&self) -> Vec<PyChromosome> {
        self.chromosomes.clone()
    }

    pub fn len(&self) -> usize {
        self.chromosomes.len()
    }

    pub fn gene_type(&self) -> PyGeneType {
        if self.chromosomes.is_empty() {
            PyGeneType::Empty
        } else {
            self.chromosomes[0].gene_type()
        }
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if self.chromosomes.is_empty() {
            Wrap(DataType::Null).into_pyobject(py)
        } else {
            self.chromosomes[0].dtype(py)
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

macro_rules! impl_into_py_genotype {
    ($chromosome:ty) => {
        impl From<Genotype<$chromosome>> for PyGenotype
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(genotype: Genotype<$chromosome>) -> Self {
                PyGenotype {
                    chromosomes: genotype
                        .into_iter()
                        .map(|chromosome| PyChromosome::from(chromosome))
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

impl_into_py_genotype!(IntChromosome<u8>);
impl_into_py_genotype!(IntChromosome<u16>);
impl_into_py_genotype!(IntChromosome<u32>);
impl_into_py_genotype!(IntChromosome<u64>);
impl_into_py_genotype!(IntChromosome<u128>);

impl_into_py_genotype!(IntChromosome<i8>);
impl_into_py_genotype!(IntChromosome<i16>);
impl_into_py_genotype!(IntChromosome<i32>);
impl_into_py_genotype!(IntChromosome<i64>);
impl_into_py_genotype!(IntChromosome<i128>);

impl_into_py_genotype!(FloatChromosome<f32>);
impl_into_py_genotype!(FloatChromosome<f64>);

impl_into_py_genotype!(BitChromosome);
impl_into_py_genotype!(CharChromosome);
impl_into_py_genotype!(GraphChromosome<Op<f32>>);
impl_into_py_genotype!(TreeChromosome<Op<f32>>);
impl_into_py_genotype!(PermutationChromosome<usize>);
impl_into_py_genotype!(AnyChromosome);
