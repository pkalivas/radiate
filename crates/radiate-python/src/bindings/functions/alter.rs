use crate::{
    AnyChromosome, InputTransform, PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation,
};
use pyo3::{PyResult, exceptions::PyValueError, pyfunction};
use radiate::prelude::*;

#[pyfunction]
pub fn py_alter(
    gene_type: PyGeneType,
    alterer: PyEngineInput,
    population: PyPopulation,
    generation: usize,
) -> PyResult<PyPopulation> {
    if !matches!(alterer.input_type, PyEngineInputType::Alterer) {
        return Err(PyValueError::new_err(format!(
            "Input type {:?} not a Alterer",
            alterer.input_type
        )));
    }

    if !alterer.allowed_genes.contains(&gene_type) {
        return Err(PyValueError::new_err(format!(
            "Alterer {} does not allow gene type {:?}",
            alterer.component, gene_type
        )));
    }

    match gene_type {
        PyGeneType::Float => {
            let alterer: Vec<Alterer<FloatChromosome>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Int => {
            let alterer: Vec<Alterer<IntChromosome<i32>>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Char => {
            let alterer: Vec<Alterer<CharChromosome>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Bit => {
            let alterer: Vec<Alterer<BitChromosome>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Permutation => {
            let alterer: Vec<Alterer<PermutationChromosome<usize>>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::AnyGene => {
            let alterer: Vec<Alterer<AnyChromosome<'static>>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::GraphNode => {
            let alterer: Vec<Alterer<GraphChromosome<Op<f32>>>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::TreeNode => {
            let alterer: Vec<Alterer<TreeChromosome<Op<f32>>>> = alterer.transform();

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        _ => Err(PyValueError::new_err(format!(
            "Gene type {:?} not supported for selection",
            gene_type
        ))),
    }
}

fn alter<C: Chromosome>(
    alterers: Vec<Alterer<C>>,
    mut population: Population<C>,
    generation: usize,
) -> Population<C> {
    for alterer in alterers {
        alterer.alter(&mut population, generation);
    }

    population
}
