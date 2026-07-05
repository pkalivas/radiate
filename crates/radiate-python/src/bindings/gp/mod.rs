mod accuracy;
mod graph;
mod ops;
mod tree;

pub use accuracy::{PyAccuracy, py_accuracy};
pub use graph::PyGraph;
use numpy::IntoPyArray;
use numpy::PyArrayDyn;
use numpy::{PyArray1, PyArrayMethods};
use numpy::{
    PyUntypedArrayMethods,
    ndarray::{ArrayD, IxDyn},
};
pub use ops::{_activation_ops, _all_ops, _create_op, _edge_ops, PyOp};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::{Bound, PyAny, PyResult};
use radiate_error::radiate_py_bail;
pub use tree::PyTree;

pub fn generic_eval_runner<'py, F>(
    py: Python<'py>,
    output_length: usize,
    inputs: &Bound<'py, PyAny>,
    mut eval_row: F,
) -> PyResult<Bound<'py, PyArrayDyn<f32>>>
where
    F: FnMut(&[f32]) -> Vec<f32>,
{
    if let Ok(np_array_f32) = inputs.cast::<PyArrayDyn<f32>>() {
        let readonly_view = np_array_f32.readonly();
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
    } else if let Ok(np_array_f64) = inputs.cast::<PyArrayDyn<f64>>() {
        // Intercept native f64 Python NumPy arrays and downcast them directly to f32
        let readonly_view = np_array_f64.readonly();
        let ndim = readonly_view.ndim();

        match ndim {
            1 => {
                let input_slice_f64 = readonly_view.as_slice()?;
                let input_slice_f32 = input_slice_f64
                    .iter()
                    .map(|&x| x as f32)
                    .collect::<Vec<f32>>();

                let output = eval_row(&input_slice_f32);
                return Ok(PyArray1::from_vec(py, output).to_dyn().into());
            }
            2 => {
                let array_view = readonly_view
                    .as_array()
                    .into_dimensionality::<numpy::ndarray::Ix2>()
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

                let mut flat_outputs = Vec::with_capacity(array_view.shape()[0] * output_length);
                let mut cast_buffer = Vec::with_capacity(array_view.shape()[1]);

                for row in array_view.rows() {
                    let input_slice_f64 = row.as_slice().ok_or_else(|| {
                        pyo3::exceptions::PyValueError::new_err(
                            "NumPy array memory must be contiguous",
                        )
                    })?;

                    // Direct buffer recycling assignment via iterator map conversion
                    cast_buffer.clear();
                    cast_buffer.extend(input_slice_f64.iter().map(|&x| x as f32));

                    flat_outputs.extend(eval_row(&cast_buffer));
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

    if let Ok(py_list) = inputs.cast::<PyList>() {
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
                        reuse_buffer.extend(row_list.extract::<Vec<f32>>()?);

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
            let input_vec: Vec<f32> = py_list.extract()?;
            let output = eval_row(&input_vec);
            return Ok(PyArray1::from_vec(py, output).to_dyn().into());
        };
    }

    radiate_py_bail!("Input must be either a 1D/2D NumPy array or a 1D/2D Python list",);
}
