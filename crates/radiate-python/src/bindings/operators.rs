use pyo3::{PyResult, pyfunction};
use radiate::prelude::*;

use crate::{InputTransform, PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation};

#[pyfunction]
pub fn py_select(
    gene_type: PyGeneType,
    selector: PyEngineInput,
    objective: PyEngineInput,
    count: usize,
    population: PyPopulation,
) -> PyResult<PyPopulation> {
    if !matches!(
        selector.input_type,
        PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector
    ) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Input type {:?} not a selector",
            selector.input_type
        )));
    }

    if !selector.allowed_genes.contains(&gene_type) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Selector {} does not allow gene type {:?}",
            selector.component, gene_type
        )));
    }

    let objectives = objective.get_string("objective").map(|objs| {
        objs.split('|')
            .map(|s| match s.trim().to_lowercase().as_str() {
                "min" => Optimize::Minimize,
                "max" => Optimize::Maximize,
                _ => panic!("Objective {} not recognized", s),
            })
            .collect::<Vec<Optimize>>()
    });

    let opt = match objectives {
        Some(objs) => {
            if objs.len() == 1 {
                Objective::Single(objs[0])
            } else if objs.len() > 1 {
                Objective::Multi(objs)
            } else {
                panic!("No objectives provided");
            }
        }
        None => Objective::Single(Optimize::Maximize),
    };

    let obj = match opt {
        Objective::Single(opt) => match opt {
            Optimize::Minimize => Objective::Single(Optimize::Minimize),
            Optimize::Maximize => Objective::Single(Optimize::Maximize),
        },
        Objective::Multi(opts) => Objective::Multi(opts),
    };

    match gene_type {
        PyGeneType::Float => {
            let selector: Box<dyn Select<FloatChromosome>> = selector.transform();
            let population: Population<FloatChromosome> = population.into();

            Ok(selector.select(&population, &obj, count)).map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Int => {
            let selector: Box<dyn Select<IntChromosome<i32>>> = selector.transform();
            let population: Population<IntChromosome<i32>> = population.into();

            Ok(selector.select(&population, &obj, count)).map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Char => {
            let selector: Box<dyn Select<CharChromosome>> = selector.transform();
            let population: Population<CharChromosome> = population.into();

            Ok(selector.select(&population, &obj, count)).map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Bit => {
            let selector: Box<dyn Select<BitChromosome>> = selector.transform();
            let population: Population<BitChromosome> = population.into();

            Ok(selector.select(&population, &obj, count)).map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Permutation => {
            let selector: Box<dyn Select<PermutationChromosome<usize>>> = selector.transform();
            let population: Population<PermutationChromosome<usize>> = population.into();

            Ok(selector.select(&population, &obj, count)).map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Graph => {
            let selector: Box<dyn Select<GraphChromosome<Op<f32>>>> = selector.transform();
            let population: Population<GraphChromosome<Op<f32>>> = population.into();

            Ok(selector.select(&population, &obj, count)).map(|pop| PyPopulation::from(&pop))
        }
        PyGeneType::Tree => {
            let selector: Box<dyn Select<TreeChromosome<Op<f32>>>> = selector.transform();
            let population: Population<TreeChromosome<Op<f32>>> = population.into();

            Ok(selector.select(&population, &obj, count)).map(|pop| PyPopulation::from(&pop))
        }
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Gene type {:?} not supported for selection",
            gene_type
        ))),
    }
}

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
