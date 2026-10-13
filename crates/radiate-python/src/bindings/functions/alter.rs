use crate::{PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation};
use pyo3::{PyResult, pyfunction};
use radiate::prelude::*;
use radiate_error::radiate_py_err;

#[pyfunction]
pub fn py_alter(
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

    match population.gene_type() {
        PyGeneType::Float => Ok(alter::<FloatChromosome<f64>>(
            alterer.convert()?,
            population.into(),
            generation,
        )?),
        PyGeneType::Int => Ok(alter::<IntChromosome<i64>>(
            alterer.convert()?,
            population.into(),
            generation,
        )?),
        PyGeneType::Char => Ok(alter::<CharChromosome>(
            alterer.convert()?,
            population.into(),
            generation,
        )?),
        PyGeneType::Bit => Ok(alter::<BitChromosome>(
            alterer.convert()?,
            population.into(),
            generation,
        )?),
        PyGeneType::Permutation => Ok(alter::<PermutationChromosome<usize>>(
            alterer.convert()?,
            population.into(),
            generation,
        )?),
        PyGeneType::GraphNode => Ok(alter::<GraphChromosome<Op<f64>>>(
            alterer.convert()?,
            population.into(),
            generation,
        )?),
        PyGeneType::TreeNode => Ok(alter::<TreeChromosome<Op<f64>>>(
            alterer.convert()?,
            population.into(),
            generation,
        )?),
        _ => Err(radiate_py_err!(format!(
            "Gene type {:?} not supported for alteration",
            population.gene_type()
        ))),
    }
}

fn alter<C>(
    mut alterers: Alterer<C>,
    mut population: Population<C>,
    generation: usize,
) -> RadiateResult<PyPopulation>
where
    C: Chromosome,
    PyPopulation: for<'a> From<&'a Population<C>>,
{
    let mut metrics = MetricSet::default();
    let pop = population.as_mut();

    alterers.alter(pop, &mut metrics, generation)?;

    Ok(PyPopulation::from(&population))
}
