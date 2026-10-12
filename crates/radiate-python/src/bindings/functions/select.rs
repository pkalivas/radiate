use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation};
use pyo3::{PyResult, pyfunction};
use radiate::prelude::*;
use radiate_error::{radiate_py_bail, radiate_py_err};

#[pyfunction]
pub fn py_select(
    selector: PyEngineInput,
    objective: PyEngineInput,
    count: usize,
    population: PyPopulation,
) -> PyResult<PyPopulation> {
    if !matches!(
        selector.input_type,
        PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector
    ) {
        return Err(radiate_py_err!(format!(
            "Input type {:?} not a selector",
            selector.input_type
        )));
    }

    let objectives = objective.extract::<Vec<String>>("objective").map(|objs| {
        objs.iter()
            .map(|val| match val.trim().to_lowercase().as_str() {
                "min" => Optimize::Minimize,
                "max" => Optimize::Maximize,
                _ => panic!("Objective {} not recognized", val),
            })
            .collect::<Vec<Optimize>>()
    })?;

    let obj = if objectives.len() == 1 {
        Objective::Single(objectives[0])
    } else if objectives.len() > 1 {
        Objective::Multi(objectives)
    } else {
        radiate_py_bail!("No objectives provided - I'm not even sure this is possible");
    };

    match population.gene_type() {
        PyGeneType::Float => {
            let selector =
                InputTransform::<RadiateResult<Box<dyn Select<FloatChromosome<f64>>>>>::transform(
                    &selector,
                )?;

            let population: Population<FloatChromosome<f64>> = population.into();

            Ok(selector
                .select(population.as_ref(), &obj, count)
                .iter()
                .map(|pop| population[*pop].clone())
                .collect::<Population<FloatChromosome<f64>>>())
            .map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Int => {
            let selector =
                InputTransform::<RadiateResult<Box<dyn Select<IntChromosome<i64>>>>>::transform(
                    &selector,
                )?;

            let population: Population<IntChromosome<i64>> = population.into();

            Ok(selector
                .select(population.as_ref(), &obj, count)
                .iter()
                .map(|pop| population[*pop].clone())
                .collect::<Population<IntChromosome<i64>>>())
            .map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Char => {
            let selector =
                InputTransform::<RadiateResult<Box<dyn Select<CharChromosome>>>>::transform(
                    &selector,
                )?;

            let population: Population<CharChromosome> = population.into();

            Ok(selector
                .select(population.as_ref(), &obj, count)
                .iter()
                .map(|pop| population[*pop].clone())
                .collect::<Population<CharChromosome>>())
            .map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Bit => {
            let selector =
                InputTransform::<RadiateResult<Box<dyn Select<BitChromosome>>>>::transform(
                    &selector,
                )?;
            let population: Population<BitChromosome> = population.into();

            Ok(selector
                .select(population.as_ref(), &obj, count)
                .iter()
                .map(|pop| population[*pop].clone())
                .collect::<Population<BitChromosome>>())
            .map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Permutation => {
            let selector = InputTransform::<
                RadiateResult<Box<dyn Select<PermutationChromosome<usize>>>>,
            >::transform(&selector)?;
            let population: Population<PermutationChromosome<usize>> = population.into();

            Ok(selector
                .select(population.as_ref(), &obj, count)
                .iter()
                .map(|pop| population[*pop].clone())
                .collect::<Population<PermutationChromosome<usize>>>())
            .map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::GraphNode => {
            let selector = InputTransform::<
                RadiateResult<Box<dyn Select<GraphChromosome<Op<f64>>>>>,
            >::transform(&selector)?;
            let population: Population<GraphChromosome<Op<f64>>> = population.into();

            Ok(selector
                .select(population.as_ref(), &obj, count)
                .iter()
                .map(|pop| population[*pop].clone())
                .collect::<Population<GraphChromosome<Op<f64>>>>())
            .map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::TreeNode => {
            let selector =
                InputTransform::<RadiateResult<Box<dyn Select<TreeChromosome<Op<f64>>>>>>::transform(
                    &selector,
                )?;
            let population: Population<TreeChromosome<Op<f64>>> = population.into();

            Ok(selector
                .select(population.as_ref(), &obj, count)
                .iter()
                .map(|pop| population[*pop].clone())
                .collect::<Population<TreeChromosome<Op<f64>>>>())
            .map(|pop| PyPopulation::from(&pop))
        }

        _ => Err(radiate_py_err!(format!(
            "Gene type {:?} not supported for selection",
            population.gene_type()
        ))),
    }
}
