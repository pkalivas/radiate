use crate::{PyEngineInput, PyEngineInputType, PyGeneType, PyPopulation, Wrap};
use pyo3::{PyResult, Python, pyfunction, types::PyAnyMethods};
use radiate::prelude::*;
use radiate_error::radiate_py_err;

#[pyfunction]
pub fn py_alter<'py>(
    py: Python<'py>,
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

    let dtype = population.dtype(py)?.extract::<Wrap<DataType>>()?;

    match population.gene_type() {
        PyGeneType::Float => match dtype.0 {
            DataType::Float32 => {
                alter::<FloatChromosome<f32>>(alterer.convert()?, population.into(), generation)
            }
            DataType::Float64 => {
                alter::<FloatChromosome<f64>>(alterer.convert()?, population.into(), generation)
            }
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for Float gene type: {:?}",
                dtype.0
            ))),
        },
        PyGeneType::Int => match dtype.0 {
            DataType::UInt8 => {
                alter::<IntChromosome<u8>>(alterer.convert()?, population.into(), generation)
            }
            DataType::UInt16 => {
                alter::<IntChromosome<u16>>(alterer.convert()?, population.into(), generation)
            }
            DataType::UInt32 => {
                alter::<IntChromosome<u32>>(alterer.convert()?, population.into(), generation)
            }
            DataType::UInt64 => {
                alter::<IntChromosome<u64>>(alterer.convert()?, population.into(), generation)
            }
            DataType::Int8 => {
                alter::<IntChromosome<i8>>(alterer.convert()?, population.into(), generation)
            }
            DataType::Int16 => {
                alter::<IntChromosome<i16>>(alterer.convert()?, population.into(), generation)
            }
            DataType::Int32 => {
                alter::<IntChromosome<i32>>(alterer.convert()?, population.into(), generation)
            }
            DataType::Int64 => {
                alter::<IntChromosome<i64>>(alterer.convert()?, population.into(), generation)
            }
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for Int gene type: {:?}",
                dtype.0
            ))),
        },
        PyGeneType::Char => {
            alter::<CharChromosome>(alterer.convert()?, population.into(), generation)
        }
        PyGeneType::Bit => {
            alter::<BitChromosome>(alterer.convert()?, population.into(), generation)
        }
        PyGeneType::Permutation => {
            alter::<PermutationChromosome<usize>>(alterer.convert()?, population.into(), generation)
        }
        PyGeneType::GraphNode => match dtype.0 {
            DataType::Float64 => {
                alter::<GraphChromosome<Op<f64>>>(alterer.convert()?, population.into(), generation)
            }
            DataType::Float32 => {
                alter::<GraphChromosome<Op<f32>>>(alterer.convert()?, population.into(), generation)
            }
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for GraphNode gene type: {:?}",
                dtype.0
            ))),
        },
        PyGeneType::TreeNode => match dtype.0 {
            DataType::Float64 => {
                alter::<TreeChromosome<Op<f64>>>(alterer.convert()?, population.into(), generation)
            }
            DataType::Float32 => {
                alter::<TreeChromosome<Op<f32>>>(alterer.convert()?, population.into(), generation)
            }
            _ => Err(radiate_py_err!(format!(
                "Unsupported data type for TreeNode gene type: {:?}",
                dtype.0
            ))),
        },
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
) -> PyResult<PyPopulation>
where
    C: Chromosome,
    PyPopulation: for<'a> From<&'a Population<C>>,
{
    let mut metrics = MetricSet::default();
    let pop = population.as_mut();

    alterers.alter(pop, &mut metrics, generation)?;

    Ok(PyPopulation::from(&population))
}
