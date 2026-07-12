mod accuracy;
mod graph;
mod ops;
mod tree;

pub use accuracy::{PyAccuracy, py_accuracy};
pub use graph::PyGraph;
use numpy::{IntoPyArray, PyArray, ndarray::IxDynImpl};
use numpy::{PyArray1, PyArrayMethods};
use numpy::{PyArrayDyn, ndarray::Dim};
use numpy::{
    PyUntypedArrayMethods,
    ndarray::{ArrayD, IxDyn},
};
pub use ops::{_activation_ops, _all_ops, _create_op, _edge_ops, PyOp};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::{Bound, PyAny, PyResult};
use radiate_error::radiate_py_bail;
use radiate_utils::Float;
pub use tree::PyTree;

pub fn generic_eval_runner<'py, F, E>(
    py: Python<'py>,
    output_length: usize,
    inputs: &Bound<'py, PyAny>,
    eval_row: E,
) -> PyResult<Bound<'py, PyArrayDyn<F>>>
where
    F: Float + numpy::Element + FromPyObjectOwned<'py>,
    E: FnMut(&[F]) -> Vec<F>,
{
    // Typed check for typed numpy arrays first - blocks the second
    // check for lists if the numpy array is of the wrong type becuase np will
    // cast the array to whatever type is requested, so we need to check for
    // the correct type first
    if let Ok(np_array) = inputs.cast::<PyArray<F, Dim<IxDynImpl>>>() {
        return run_gp_eval_array(py, output_length, np_array, eval_row);
    } else if let Ok(py_list) = inputs.cast::<PyList>() {
        return run_gp_eval_list(py, output_length, py_list, eval_row);
    }

    radiate_py_bail!(format!(
        "GP with dtype {:?} Eval recieved unsupported input type",
        F::DTYPE,
    ));
}

fn run_gp_eval_array<'py, F, E>(
    py: Python<'py>,
    output_length: usize,
    np_array: &Bound<'_, PyArray<F, Dim<IxDynImpl>>>,
    mut eval_row: E,
) -> PyResult<Bound<'py, PyArrayDyn<F>>>
where
    F: Float + numpy::Element,
    E: FnMut(&[F]) -> Vec<F>,
{
    let readonly_view = np_array.readonly();
    let ndim = readonly_view.ndim();

    match ndim {
        1 => {
            let input_slice = readonly_view.as_slice()?;
            let output = eval_row(input_slice);
            return Ok(PyArray1::from_vec(py, output).to_dyn().into());
        }
        2 => {
            let array_view = readonly_view
                .as_array()
                .into_dimensionality::<numpy::ndarray::Ix2>()
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            let mut flat_outputs = Vec::with_capacity(array_view.shape()[0] * output_length);
            for row in array_view.rows() {
                flat_outputs.extend(eval_row(match row.as_slice() {
                    Some(slice) => slice,
                    None => {
                        radiate_py_bail!("NumPy array memory must be contiguous",);
                    }
                }));
            }

            let rows = array_view.shape()[0];
            let cols = output_length;

            let shape = IxDyn(&[rows, cols]);
            let ndarray_2d = ArrayD::from_shape_vec(shape, flat_outputs)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

            return Ok(ndarray_2d.into_pyarray(py).to_dyn().into());
        }
        _ => {
            radiate_py_bail!("Expected 1D or 2D NumPy array",);
        }
    };
}

fn run_gp_eval_list<'py, F, E>(
    py: Python<'py>,
    output_length: usize,
    py_list: &Bound<'py, PyList>,
    mut eval_row: E,
) -> PyResult<Bound<'py, PyArrayDyn<F>>>
where
    F: Float + numpy::Element + FromPyObjectOwned<'py>,
    E: FnMut(&[F]) -> Vec<F>,
{
    if py_list.is_empty() {
        radiate_py_bail!("Input list cannot be empty",);
    }

    let first_element = py_list.get_item(0)?;

    if first_element.cast::<PyList>().is_ok() {
        let rows = py_list.len();
        let mut flat_outputs = Vec::new();
        let mut reuse_buffer = Vec::new();

        for item in py_list.try_iter()? {
            match item?.cast::<PyList>() {
                Ok(row_list) => {
                    reuse_buffer.clear();
                    reuse_buffer.extend(row_list.extract::<Vec<F>>()?);

                    flat_outputs.extend(eval_row(&reuse_buffer));
                }
                Err(_) => {
                    radiate_py_bail!("All elements of the outer list must be lists",);
                }
            }
        }

        let shape = IxDyn(&[rows, output_length]);
        let ndarray_2d = ArrayD::from_shape_vec(shape, flat_outputs)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        return Ok(ndarray_2d.into_pyarray(py).to_dyn().into());
    } else {
        let input_vec: Vec<F> = py_list.extract()?;
        let output = eval_row(&input_vec);
        return Ok(PyArray1::from_vec(py, output).to_dyn().into());
    };
}
