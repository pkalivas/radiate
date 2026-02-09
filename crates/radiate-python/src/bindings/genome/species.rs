use crate::{AnyChromosome, PyPhenotype, PyPopulation};
use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, FloatChromosome, GraphChromosome, IntChromosome, Op,
    PermutationChromosome, Phenotype, Population, Species, TreeChromosome,
};

#[pyclass(from_py_object)]
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

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.population.dtype(py)
    }
}

macro_rules! impl_into_py_species {
    ($chromosome:ty) => {
        impl From<Species<$chromosome>> for PySpecies
        where
            $chromosome: Chromosome + Clone,
        {
            fn from(species: Species<$chromosome>) -> Self {
                PySpecies {
                    id: species.id().0,
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

impl_into_py_species!(IntChromosome<u8>);
impl_into_py_species!(IntChromosome<u16>);
impl_into_py_species!(IntChromosome<u32>);
impl_into_py_species!(IntChromosome<u64>);
impl_into_py_species!(IntChromosome<u128>);

impl_into_py_species!(IntChromosome<i8>);
impl_into_py_species!(IntChromosome<i16>);
impl_into_py_species!(IntChromosome<i32>);
impl_into_py_species!(IntChromosome<i64>);
impl_into_py_species!(IntChromosome<i128>);

impl_into_py_species!(FloatChromosome<f32>);
impl_into_py_species!(FloatChromosome<f64>);

impl_into_py_species!(BitChromosome);
impl_into_py_species!(CharChromosome);
impl_into_py_species!(GraphChromosome<Op<f32>>);
impl_into_py_species!(TreeChromosome<Op<f32>>);
impl_into_py_species!(PermutationChromosome<usize>);
impl_into_py_species!(AnyChromosome);
