use crate::{PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation, Wrap};
use pyo3::{PyResult, Python, pyfunction, types::PyAnyMethods};
use radiate::prelude::*;
use radiate_error::{radiate_py_bail, radiate_py_err};

#[pyfunction]
pub fn py_select<'py>(
    py: Python<'py>,
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

    let dtype = population.dtype(py)?.extract::<Wrap<DataType>>()?;

    match population.gene_type() {
        PyGeneType::Float => match dtype.0 {
            DataType::Float32 => select_population::<FloatChromosome<f32>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::Float64 => select_population::<FloatChromosome<f64>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for Float gene type: {:?}",
                dtype.0
            ))),
        },
        PyGeneType::Int => match dtype.0 {
            DataType::UInt8 => select_population::<IntChromosome<u8>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::UInt16 => select_population::<IntChromosome<u16>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::UInt32 => select_population::<IntChromosome<u32>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::UInt64 => select_population::<IntChromosome<u64>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::Int8 => select_population::<IntChromosome<i8>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::Int16 => select_population::<IntChromosome<i16>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::Int32 => select_population::<IntChromosome<i32>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::Int64 => select_population::<IntChromosome<i64>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for Int gene type: {:?}",
                dtype.0
            ))),
        },
        PyGeneType::Char => select_population::<CharChromosome>(
            population.into(),
            objective,
            count,
            selector.convert()?,
        ),
        PyGeneType::Bit => select_population::<BitChromosome>(
            population.into(),
            objective,
            count,
            selector.convert()?,
        ),
        PyGeneType::Permutation => select_population::<PermutationChromosome<usize>>(
            population.into(),
            objective,
            count,
            selector.convert()?,
        ),
        PyGeneType::GraphNode => match dtype.0 {
            DataType::Float32 => select_population::<GraphChromosome<Op<f32>>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::Float64 => select_population::<GraphChromosome<Op<f64>>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for GraphNode gene type: {:?}",
                dtype.0
            ))),
        },
        PyGeneType::TreeNode => match dtype.0 {
            DataType::Float32 => select_population::<TreeChromosome<Op<f32>>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            DataType::Float64 => select_population::<TreeChromosome<Op<f64>>>(
                population.into(),
                objective,
                count,
                selector.convert()?,
            ),
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for TreeNode gene type: {:?}",
                dtype.0
            ))),
        },
        _ => Err(radiate_py_err!(format!(
            "Gene type {:?} not supported for selection",
            population.gene_type()
        ))),
    }
}

fn select_population<C>(
    population: Population<C>,
    objective: PyEngineInput,
    count: usize,
    selector: Box<dyn Select<C>>,
) -> PyResult<PyPopulation>
where
    C: Chromosome + Clone,
    PyPopulation: for<'a> From<&'a Population<C>>,
{
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

    Ok(selector
        .select(population.as_ref(), &obj, count)
        .iter()
        .map(|pop| population[*pop].clone())
        .collect::<Population<C>>())
    .map(|pop| PyPopulation::from(&pop))
}
