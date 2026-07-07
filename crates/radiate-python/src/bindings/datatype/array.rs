use numpy::{
    IntoPyArray, IxDyn, PyArray, PyArray1, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods,
    ndarray::{ArrayD, Dim, IxDynImpl, s},
};
use pyo3::{
    Bound, PyAny, PyResult, Python,
    prelude::FromPyObjectOwned,
    types::{PyAnyMethods, PyList, PyListMethods},
};
use radiate_error::radiate_py_bail;
use radiate_utils::Float;

pub(crate) fn py_object_into_ndarray<'py, P, F>(
    py: Python<'py>,
    obj: &Bound<'py, PyAny>,
    func: F,
) -> PyResult<Bound<'py, PyArrayDyn<P>>>
where
    F: Fn(&Bound<'py, PyArray<P, Dim<IxDynImpl>>>) -> PyResult<Bound<'py, PyArrayDyn<P>>>,
    P: Float + numpy::Element + FromPyObjectOwned<'py>,
{
    if let Ok(np_array) = obj.cast::<PyArrayDyn<P>>() {
        return func(&np_array);
    } else if let Ok(np_array) = obj.cast::<PyArray<P, Dim<IxDynImpl>>>() {
        return func(&np_array);
    } else if let Ok(py_list) = obj.cast::<pyo3::types::PyList>() {
        let first_element = py_list.get_item(0)?;

        if first_element.cast::<PyList>().is_ok() {
            let rows = py_list.len();
            let mut col_count = None;
            let mut flat_outputs = Vec::new();

            for item in py_list.try_iter()? {
                match item?.cast::<PyList>() {
                    Ok(row_list) => {
                        if col_count.is_none() {
                            col_count = Some(row_list.len());
                        } else if col_count.unwrap() != row_list.len() {
                            radiate_py_bail!("All rows must have the same number of columns",);
                        }
                        flat_outputs.extend(row_list.extract::<Vec<P>>()?);
                    }
                    Err(_) => {
                        radiate_py_bail!("All elements of the outer list must be lists",);
                    }
                }
            }

            if let Some(col_count) = col_count {
                let shape = IxDyn(&[rows, col_count]);
                let ndarray_2d = ArrayD::from_shape_vec(shape, flat_outputs)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

                return Ok(ndarray_2d.into_pyarray(py).to_dyn().into());
            } else {
                radiate_py_bail!("Input list cannot be empty",);
            }
        } else {
            let input_vec = py_list.extract::<Vec<P>>()?;

            return Ok(PyArray1::from_vec(py, input_vec).to_dyn().into());
        };
    }

    Err(pyo3::exceptions::PyTypeError::new_err(format!(
        "Input must be either a NumPy array or a list but found: {:?}",
        obj
    )))
}

pub(crate) fn py_object_into_2d_vec<'py, F>(
    _: Python<'py>,
    obj: &Bound<'py, PyAny>,
) -> PyResult<Vec<Vec<F>>>
where
    F: Float + numpy::Element + FromPyObjectOwned<'py>,
{
    if let Ok(np_array) = obj.cast::<PyArrayDyn<F>>() {
        let array = np_array.readonly();
        if array.ndim() != 2 {
            radiate_py_bail!("Expected a 2D NumPy array",);
        }
        let rows = array.shape()[0];
        let cols = array.shape()[1];
        let mut result = Vec::with_capacity(rows);
        for i in 0..rows {
            let row = array.as_array().slice(s![i, ..]).to_vec();
            result.push(row);
        }
        return Ok(result);
    } else if let Ok(py_list) = obj.cast::<pyo3::types::PyList>() {
        let mut result = Vec::new();
        for item in py_list.try_iter()? {
            match item?.cast::<PyList>() {
                Ok(row_list) => {
                    let row: Vec<F> = row_list.extract()?;
                    result.push(row);
                }
                Err(_) => {
                    radiate_py_bail!("All elements of the outer list must be lists",);
                }
            }
        }
        return Ok(result);
    }

    Err(pyo3::exceptions::PyTypeError::new_err(format!(
        "Input must be either a 2D NumPy array or a list of lists but found: {:?}",
        obj
    )))
}

/// A (features, targets) pair, each already extracted into a single, shared float
/// width. `Regression<f32>`/`Regression<f64>` are genuinely different Rust types,
/// so this is the smallest thing that can carry either result out of dispatch.
pub(crate) enum FloatMatrixPair {
    F32 {
        features: Vec<Vec<f32>>,
        targets: Vec<Vec<f32>>,
    },
    F64 {
        features: Vec<Vec<f64>>,
        targets: Vec<Vec<f64>>,
    },
}

/// Decide one shared float width for a `(features, targets)` pair, then extract
/// both into it.
///
/// Policy: f32 only when *both* inputs are already f32 NumPy arrays — that's the
/// only case where the caller unambiguously asked for f32. Everything else (an
/// f64 NumPy array, a plain Python list — which has no dtype of its own, Python
/// floats are always f64 — or a mismatched pair) resolves to f64. This mirrors
/// `problem.rs`'s existing `cast::<PyArrayDyn<f32>>()` / `cast::<PyArrayDyn<f64>>()`
/// probe order rather than inventing a new dtype-sniffing mechanism.
pub(crate) fn extract_regression_pair<'py>(
    py: Python<'py>,
    features: &Bound<'py, PyAny>,
    targets: &Bound<'py, PyAny>,
) -> PyResult<FloatMatrixPair> {
    let both_f32 =
        features.cast::<PyArrayDyn<f32>>().is_ok() && targets.cast::<PyArrayDyn<f32>>().is_ok();

    if both_f32 {
        Ok(FloatMatrixPair::F32 {
            features: py_object_into_2d_vec::<f32>(py, features)?,
            targets: py_object_into_2d_vec::<f32>(py, targets)?,
        })
    } else {
        Ok(FloatMatrixPair::F64 {
            features: py_object_into_2d_vec::<f64>(py, features)?,
            targets: py_object_into_2d_vec::<f64>(py, targets)?,
        })
    }
}
