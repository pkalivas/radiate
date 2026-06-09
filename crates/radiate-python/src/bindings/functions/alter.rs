use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation};
use pyo3::{PyResult, pyfunction};
use radiate::prelude::*;
use radiate_error::radiate_py_err;

#[pyfunction]
pub fn py_alter(
    gene_type: PyGeneType,
    alterer: PyEngineInput,
    population: PyPopulation,
    generation: usize,
) -> PyResult<PyPopulation> {
    if !matches!(alterer.input_type, PyEngineInputType::Alterer) {
        return Err(radiate_py_err!(format!(
            "Input type {:?} not an Alterer",
            alterer.input_type
        )));
    }

    if !alterer.allowed_genes.contains(&gene_type) {
        return Err(radiate_py_err!(format!(
            "Alterer {} does not allow gene type {:?}",
            alterer.component, gene_type
        )));
    }

    match gene_type {
        PyGeneType::Float => {
            let alterer =
                InputTransform::<RadiateResult<Vec<Alterer<FloatChromosome<f64>>>>>::transform(
                    &alterer,
                )?;

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Int => {
            let alterer =
                InputTransform::<RadiateResult<Vec<Alterer<IntChromosome<i64>>>>>::transform(
                    &alterer,
                )?;

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Char => {
            let alterer =
                InputTransform::<RadiateResult<Vec<Alterer<CharChromosome>>>>::transform(&alterer)?;

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Bit => {
            let alterer =
                InputTransform::<RadiateResult<Vec<Alterer<BitChromosome>>>>::transform(&alterer)?;

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::Permutation => {
            let alterer = InputTransform::<
                RadiateResult<Vec<Alterer<PermutationChromosome<usize>>>>,
            >::transform(&alterer)?;

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::GraphNode => {
            let alterer =
                InputTransform::<RadiateResult<Vec<Alterer<GraphChromosome<Op<f32>>>>>>::transform(
                    &alterer,
                )?;

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        PyGeneType::TreeNode => {
            let alterer =
                InputTransform::<RadiateResult<Vec<Alterer<TreeChromosome<Op<f32>>>>>>::transform(
                    &alterer,
                )?;

            Ok(PyPopulation::from(&alter(
                alterer,
                population.into(),
                generation,
            )))
        }
        _ => Err(radiate_py_err!(format!(
            "Gene type {:?} not supported for alteration",
            gene_type
        ))),
    }
}

fn alter<C: Chromosome>(
    mut alterers: Vec<Alterer<C>>,
    mut population: Population<C>,
    generation: usize,
) -> Population<C> {
    let mut metrics = MetricSet::default();
    let pop = population.as_mut();
    for alterer in alterers.iter_mut() {
        alterer.alter(pop, &mut metrics, generation);
    }

    population
}
