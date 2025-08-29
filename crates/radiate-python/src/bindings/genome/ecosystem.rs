use pyo3::{pyclass, pymethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, Ecosystem, FloatChromosome, GraphChromosome,
    IntChromosome, Op, PermutationChromosome, Population, Species, TreeChromosome,
};

use crate::{AnyChromosome, PyPopulation, PySpecies};

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
impl_into_py_ecosystem!(AnyChromosome<'static>);
