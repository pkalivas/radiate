use pyo3::{PyResult, pyfunction};
use radiate::prelude::*;

use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation};

#[pyfunction]
pub fn py_alter(
    gene_type: PyGeneType,
    alterer: PyEngineInput,
    population: PyPopulation,
    generation: usize,
) -> PyResult<PyPopulation> {
    if !matches!(alterer.input_type, PyEngineInputType::Alterer) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Input type {:?} not a Alterer",
            alterer.input_type
        )));
    }

    if !alterer.allowed_genes.contains(&gene_type) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Alterer {} does not allow gene type {:?}",
            alterer.component, gene_type
        )));
    }

    match gene_type {
        PyGeneType::Float => {
            let alterer: Vec<Box<dyn Alter<FloatChromosome>>> = alterer.transform();
            let mut population: Population<FloatChromosome> = population.into();

            for alterer in alterer {
                alterer.alter(&mut population, generation);
            }

            Ok(PyPopulation::from(&population))
        }
        PyGeneType::Int => {
            let alterer: Vec<Box<dyn Alter<IntChromosome<i32>>>> = alterer.transform();
            let mut population: Population<IntChromosome<i32>> = population.into();

            for alterer in alterer {
                alterer.alter(&mut population, generation);
            }

            Ok(PyPopulation::from(&population))
        }
        PyGeneType::Char => {
            let alterer: Vec<Box<dyn Alter<CharChromosome>>> = alterer.transform();
            let mut population: Population<CharChromosome> = population.into();

            for alterer in alterer {
                alterer.alter(&mut population, generation);
            }

            Ok(PyPopulation::from(&population))
        }
        PyGeneType::Bit => {
            let alterer: Vec<Box<dyn Alter<BitChromosome>>> = alterer.transform();
            let mut population: Population<BitChromosome> = population.into();

            for alterer in alterer {
                alterer.alter(&mut population, generation);
            }

            Ok(PyPopulation::from(&population))
        }
        PyGeneType::Permutation => {
            let alterer: Vec<Box<dyn Alter<PermutationChromosome<usize>>>> = alterer.transform();
            let mut population: Population<PermutationChromosome<usize>> = population.into();

            for alterer in alterer {
                alterer.alter(&mut population, generation);
            }

            Ok(PyPopulation::from(&population))
        }
        PyGeneType::Graph => {
            let alterer: Vec<Box<dyn Alter<GraphChromosome<Op<f32>>>>> = alterer.transform();
            let mut population: Population<GraphChromosome<Op<f32>>> = population.into();

            for alterer in alterer {
                alterer.alter(&mut population, generation);
            }

            Ok(PyPopulation::from(&population))
        }
        PyGeneType::Tree => {
            let alterer: Vec<Box<dyn Alter<TreeChromosome<Op<f32>>>>> = alterer.transform();
            let mut population: Population<TreeChromosome<Op<f32>>> = population.into();

            for alterer in alterer {
                alterer.alter(&mut population, generation);
            }

            Ok(PyPopulation::from(&population))
        }
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Gene type {:?} not supported for selection",
            gene_type
        ))),
    }
}
