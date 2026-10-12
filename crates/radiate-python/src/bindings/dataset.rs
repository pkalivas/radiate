use crate::{Wrap, bindings::datatype, dtype};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{DataSet, DataType, dtype_names};

enum DataSetInner {
    F32(DataSet<f32>),
    F64(DataSet<f64>),
}

#[pyclass]
pub struct PyDataSet {
    inner: DataSetInner,
}

#[pymethods]
impl PyDataSet {
    #[staticmethod]
    #[pyo3(signature = (dtype=None))]
    pub fn cubic(dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::cubic()),
                _ => DataSetInner::F64(DataSet::cubic()),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (dtype=None))]
    pub fn iris(dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::iris()),
                _ => DataSetInner::F64(DataSet::iris()),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (n=10, dtype=None))]
    pub fn simple_memory(n: usize, dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::simple_memory(n)),
                _ => DataSetInner::F64(DataSet::simple_memory(n)),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (bits=3, dtype=None))]
    pub fn parity(bits: usize, dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::parity(bits)),
                _ => DataSetInner::F64(DataSet::parity(bits)),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (dtype=None))]
    pub fn xor(dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::xor()),
                _ => DataSetInner::F64(DataSet::xor()),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (dtype=None))]
    pub fn koza1(dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::koza1()),
                _ => DataSetInner::F64(DataSet::koza1()),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (dtype=None))]
    pub fn nguyen7(dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::nguyen7()),
                _ => DataSetInner::F64(DataSet::nguyen7()),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (dtype=None))]
    pub fn pagie1(dtype: Option<&str>) -> Self {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::pagie1()),
                _ => DataSetInner::F64(DataSet::pagie1()),
            },
        }
    }

    #[staticmethod]
    #[pyo3(signature = (n=100, noise=0.1, dtype=None))]
    pub fn moons(n: usize, noise: f64, dtype: Option<&str>) -> PyResult<Self> {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        if !(0.0..=1.0).contains(&noise) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Noise must be between 0 and 1",
            ));
        }

        Ok(PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::moons(n, noise)),
                _ => DataSetInner::F64(DataSet::moons(n, noise)),
            },
        })
    }

    #[staticmethod]
    #[pyo3(signature = (n=100, noise=0.1, dtype=None))]
    pub fn circles(n: usize, noise: f64, dtype: Option<&str>) -> PyResult<Self> {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        if !(0.0..=1.0).contains(&noise) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Noise must be between 0 and 1",
            ));
        }

        Ok(PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::circles(n, noise)),
                _ => DataSetInner::F64(DataSet::circles(n, noise)),
            },
        })
    }

    #[staticmethod]
    #[pyo3(signature = (n=100, noise=0.1, dtype=None))]
    pub fn spirals(n: usize, noise: f64, dtype: Option<&str>) -> PyResult<Self> {
        let dtype = dtype::dtype_from_str(dtype.unwrap_or(dtype_names::FLOAT64));

        if !(0.0..=1.0).contains(&noise) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Noise must be between 0 and 1",
            ));
        }

        Ok(PyDataSet {
            inner: match dtype {
                DataType::Float32 => DataSetInner::F32(DataSet::spirals(n, noise)),
                _ => DataSetInner::F64(DataSet::spirals(n, noise)),
            },
        })
    }

    pub fn features<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            DataSetInner::F64(inner) => datatype::matrix_to_numpy(py, inner.feature_matrix()),
            DataSetInner::F32(inner) => datatype::matrix_to_numpy(py, inner.feature_matrix()),
        }
    }

    pub fn labels<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match &self.inner {
            DataSetInner::F64(inner) => datatype::matrix_to_numpy(py, inner.label_matrix()),
            DataSetInner::F32(inner) => datatype::matrix_to_numpy(py, inner.label_matrix()),
        }
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Wrap(match &self.inner {
            DataSetInner::F64(_) => DataType::Float64,
            DataSetInner::F32(_) => DataType::Float32,
        })
        .into_bound_py_any(py)
    }

    pub fn shape(&self) -> PyResult<(usize, usize, usize)> {
        match &self.inner {
            DataSetInner::F64(inner) => Ok(inner.shape()),
            DataSetInner::F32(inner) => Ok(inner.shape()),
        }
    }

    pub fn shuffle(&mut self) -> PyResult<Self> {
        match &mut self.inner {
            DataSetInner::F64(inner) => {
                let shuffled = inner.clone().shuffle();
                Ok(PyDataSet {
                    inner: DataSetInner::F64(shuffled),
                })
            }
            DataSetInner::F32(inner) => {
                let shuffled = inner.clone().shuffle();
                Ok(PyDataSet {
                    inner: DataSetInner::F32(shuffled),
                })
            }
        }
    }

    pub fn split(&self, test_size: f32) -> PyResult<(Self, Self)> {
        if !(0.0..=1.0).contains(&test_size) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Test size must be between 0 and 1",
            ));
        }

        match &self.inner {
            DataSetInner::F64(inner) => {
                let (train, test) = inner.clone().split(test_size);
                Ok((
                    PyDataSet {
                        inner: DataSetInner::F64(train),
                    },
                    PyDataSet {
                        inner: DataSetInner::F64(test),
                    },
                ))
            }
            DataSetInner::F32(inner) => {
                let (train, test) = inner.clone().split(test_size);
                Ok((
                    PyDataSet {
                        inner: DataSetInner::F32(train),
                    },
                    PyDataSet {
                        inner: DataSetInner::F32(test),
                    },
                ))
            }
        }
    }

    pub fn standardize(&mut self) -> PyResult<Self> {
        match &mut self.inner {
            DataSetInner::F64(inner) => {
                let standardized = inner.standardize().clone();
                Ok(PyDataSet {
                    inner: DataSetInner::F64(standardized),
                })
            }
            DataSetInner::F32(inner) => {
                let standardized = inner.standardize().clone();
                Ok(PyDataSet {
                    inner: DataSetInner::F32(standardized),
                })
            }
        }
    }

    pub fn normalize(&mut self) -> PyResult<Self> {
        match &mut self.inner {
            DataSetInner::F64(inner) => {
                let normalized = inner.normalize().clone();
                Ok(PyDataSet {
                    inner: DataSetInner::F64(normalized),
                })
            }
            DataSetInner::F32(inner) => {
                let normalized = inner.normalize().clone();
                Ok(PyDataSet {
                    inner: DataSetInner::F32(normalized),
                })
            }
        }
    }
}
