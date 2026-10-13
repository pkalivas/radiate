use numpy::{Element, PyArray1, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use pyo3::{
    Bound, PyAny, PyResult, Python,
    prelude::FromPyObjectOwned,
    types::{PyAnyMethods, PyList},
};
use radiate::DataType;
use radiate_error::radiate_py_bail;
use radiate_utils::{Float, Matrix};

pub(crate) enum FloatMatrixPair {
    F32 {
        features: Matrix<f32>,
        targets: Matrix<f32>,
    },
    F64 {
        features: Matrix<f64>,
        targets: Matrix<f64>,
    },
}

pub(crate) fn extract_regression_pair<'py>(
    _: Python<'py>,
    wanted_dtype: DataType,
    features: &Bound<'py, PyAny>,
    targets: &Bound<'py, PyAny>,
) -> PyResult<FloatMatrixPair> {
    if let Ok(cast_features) = features.cast::<PyArrayDyn<f32>>()
        && let Ok(cast_targets) = targets.cast::<PyArrayDyn<f32>>()
    {
        match wanted_dtype {
            DataType::Float32 => Ok(FloatMatrixPair::F32 {
                features: py_object_into_matrix::<f32>(cast_features)?,
                targets: py_object_into_matrix::<f32>(cast_targets)?,
            }),
            DataType::Float64 => Ok(FloatMatrixPair::F64 {
                features: py_object_into_matrix::<f64>(cast_features)?,
                targets: py_object_into_matrix::<f64>(cast_targets)?,
            }),
            _ => radiate_py_bail!(
                "Unsupported data type for regression pair extraction: {wanted_dtype:?}"
            ),
        }
    } else if let Ok(cast_features) = features.cast::<PyArrayDyn<f64>>()
        && let Ok(cast_targets) = targets.cast::<PyArrayDyn<f64>>()
    {
        match wanted_dtype {
            DataType::Float32 => Ok(FloatMatrixPair::F32 {
                features: py_object_into_matrix::<f32>(cast_features)?,
                targets: py_object_into_matrix::<f32>(cast_targets)?,
            }),
            DataType::Float64 => Ok(FloatMatrixPair::F64 {
                features: py_object_into_matrix::<f64>(cast_features)?,
                targets: py_object_into_matrix::<f64>(cast_targets)?,
            }),
            _ => radiate_py_bail!(
                "Unsupported data type for regression pair extraction: {wanted_dtype:?}"
            ),
        }
    } else {
        radiate_py_bail!("Features and targets must be either 2D NumPy arrays of f32 or f64");
    }
}

pub(crate) fn py_object_into_matrix<'py, F>(obj: &Bound<'py, PyAny>) -> PyResult<Matrix<F>>
where
    F: Float + numpy::Element + FromPyObjectOwned<'py>,
{
    if let Ok(np_array) = obj.cast::<PyArrayDyn<F>>() {
        let array = np_array.readonly();
        if array.ndim() != 2 {
            radiate_py_bail!("Expected a 2D NumPy array",);
        }

        let shape = array.shape();
        let rows = shape[0];
        let cols = shape[1];

        return Ok(Matrix::from(array.as_slice()?.to_vec()).reshape(rows, cols));
    } else if let Ok(py_list) = obj.cast::<pyo3::types::PyList>() {
        let mut matrix = Matrix::empty();
        for item in py_list.try_iter()? {
            match item?.cast::<PyList>() {
                Ok(row_list) => {
                    matrix.append_row(row_list.extract::<Vec<F>>()?);
                }
                Err(_) => {
                    radiate_py_bail!("All elements of the outer list must be lists",);
                }
            }
        }

        return Ok(matrix);
    }

    Err(pyo3::exceptions::PyTypeError::new_err(format!(
        "Input must be either a 2D NumPy array or a list of lists but found: {:?}\n
         Please ensure your input is correctly formatted and your DataType is correct.",
        obj
    )))
}

pub(crate) fn matrix_to_numpy<'py, T: Element>(
    py: Python<'py>,
    matrix: &Matrix<T>,
) -> PyResult<Bound<'py, PyAny>> {
    Ok(PyArray1::from_slice(py, matrix.data())
        .reshape([matrix.rows(), matrix.cols()])?
        .into_any())
}
