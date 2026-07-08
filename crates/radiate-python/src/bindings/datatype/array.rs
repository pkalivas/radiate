use numpy::{PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods, ndarray::s};
use pyo3::{
    Bound, PyAny, PyResult, Python,
    prelude::FromPyObjectOwned,
    types::{PyAnyMethods, PyList},
};
use radiate_error::radiate_py_bail;
use radiate_utils::Float;

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

        let mut result = Vec::with_capacity(rows);
        for i in 0..rows {
            result.push(array.as_array().slice(s![i, ..]).to_vec());
        }

        return Ok(result);
    } else if let Ok(py_list) = obj.cast::<pyo3::types::PyList>() {
        let mut result = Vec::new();
        for item in py_list.try_iter()? {
            match item?.cast::<PyList>() {
                Ok(row_list) => {
                    result.push(row_list.extract::<Vec<F>>()?);
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
