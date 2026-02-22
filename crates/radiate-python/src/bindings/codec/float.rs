use crate::{
    DataType, PyChromosome, PyGene, PyGenotype, Wrap,
    bindings::codec::{NumericCodecBuilder, TypedNumericCodec, builder::CodecBuilder},
    dtype, dtype_names,
};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyFloatCodec {
    pub codec: TypedNumericCodec,
}

#[pymethods]
impl PyFloatCodec {
    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        Ok(self.codec.encode())
    }

    pub fn decode_py<'py>(
        &self,
        py: pyo3::Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.codec.decode_with_py(py, genotype)
    }

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Wrap(self.codec.dtype()).into_bound_py_any(py)
    }

    #[staticmethod]
    #[pyo3(signature = (chromosomes, use_numpy=false))]
    pub fn from_chromosomes(chromosomes: Vec<PyChromosome>, use_numpy: bool) -> Self {
        NumericCodecBuilder::default()
            .chromosomes(chromosomes)
            .use_numpy(use_numpy)
            .into()
    }

    #[staticmethod]
    #[pyo3(signature = (genes, use_numpy=false))]
    pub fn from_genes(genes: Vec<PyGene>, use_numpy: bool) -> Self {
        NumericCodecBuilder::default()
            .genes(genes)
            .use_numpy(use_numpy)
            .into()
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None, use_numpy=false, dtype=None))]
    pub fn matrix(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(f64, f64)>,
        bound_range: Option<(f64, f64)>,
        use_numpy: bool,
        dtype: Option<String>,
    ) -> Self {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::FLOAT64.into()));

        NumericCodecBuilder::default()
            .shape(chromosome_lengths.unwrap_or(vec![1]))
            .init_range(value_range)
            .bound_range(bound_range)
            .use_numpy(use_numpy)
            .dtype(dtype)
            .into()
    }

    #[staticmethod]
    #[pyo3(signature = (length=1, value_range=None, bound_range=None, use_numpy=false, dtype=None))]
    pub fn vector(
        length: usize,
        value_range: Option<(f64, f64)>,
        bound_range: Option<(f64, f64)>,
        use_numpy: bool,
        dtype: Option<String>,
    ) -> Self {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::FLOAT64.into()));

        NumericCodecBuilder::default()
            .shape(vec![length])
            .init_range(value_range)
            .bound_range(bound_range)
            .use_numpy(use_numpy)
            .dtype(dtype)
            .into()
    }

    #[staticmethod]
    #[pyo3(signature = (value_range=None, bound_range=None, dtype=None))]
    pub fn scalar(
        value_range: Option<(f64, f64)>,
        bound_range: Option<(f64, f64)>,
        dtype: Option<String>,
    ) -> Self {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::FLOAT64.into()));

        NumericCodecBuilder::default()
            .shape(vec![1])
            .init_range(value_range)
            .bound_range(bound_range)
            .use_numpy(false)
            .dtype(dtype)
            .into()
    }
}

unsafe impl Send for PyFloatCodec {}
unsafe impl Sync for PyFloatCodec {}

impl From<NumericCodecBuilder<f64>> for PyFloatCodec {
    fn from(builder: NumericCodecBuilder<f64>) -> Self {
        match builder.dtype {
            DataType::Float32 => PyFloatCodec {
                codec: TypedNumericCodec::F32(builder.build()),
            },
            DataType::Float64 => PyFloatCodec {
                codec: TypedNumericCodec::F64(builder.build()),
            },
            _ => panic!("Invalid data type for PyFloatCodec"),
        }
    }
}
