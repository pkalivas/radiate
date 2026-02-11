use crate::{DataType, dtype_names};
use crate::{
    PyChromosome, PyGene, PyGenotype,
    bindings::{
        codec::{NumericCodecBuilder, TypedNumericCodec, builder::CodecBuilder},
        dtype,
    },
};
use pyo3::{Bound, PyAny, PyResult, pyclass, pymethods};

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyIntCodec {
    pub codec: TypedNumericCodec,
}

#[pymethods]
impl PyIntCodec {
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
        value_range: Option<(i64, i64)>,
        bound_range: Option<(i64, i64)>,
        use_numpy: bool,
        dtype: Option<String>,
    ) -> Self {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::INT64.into()));

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
        value_range: Option<(i64, i64)>,
        bound_range: Option<(i64, i64)>,
        use_numpy: bool,
        dtype: Option<String>,
    ) -> Self {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::INT64.into()));

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
        value_range: Option<(i64, i64)>,
        bound_range: Option<(i64, i64)>,
        dtype: Option<String>,
    ) -> Self {
        let dtype = dtype::dtype_from_str(&dtype.unwrap_or_else(|| dtype_names::INT64.into()));

        NumericCodecBuilder::default()
            .shape(vec![1])
            .init_range(value_range)
            .bound_range(bound_range)
            .dtype(dtype)
            .into()
    }
}

unsafe impl Send for PyIntCodec {}
unsafe impl Sync for PyIntCodec {}

impl From<NumericCodecBuilder<i64>> for PyIntCodec {
    fn from(codec: NumericCodecBuilder<i64>) -> Self {
        match codec.dtype {
            DataType::UInt8 => PyIntCodec {
                codec: TypedNumericCodec::U8(codec.build()),
            },
            DataType::UInt16 => PyIntCodec {
                codec: TypedNumericCodec::U16(codec.build()),
            },
            DataType::UInt32 => PyIntCodec {
                codec: TypedNumericCodec::U32(codec.build()),
            },
            DataType::UInt64 => PyIntCodec {
                codec: TypedNumericCodec::U64(codec.build()),
            },
            DataType::Int8 => PyIntCodec {
                codec: TypedNumericCodec::I8(codec.build()),
            },
            DataType::Int16 => PyIntCodec {
                codec: TypedNumericCodec::I16(codec.build()),
            },
            DataType::Int32 => PyIntCodec {
                codec: TypedNumericCodec::I32(codec.build()),
            },
            DataType::Int64 => PyIntCodec {
                codec: TypedNumericCodec::I64(codec.build()),
            },

            _ => panic!("Unsupported dtype for IntCodec"),
        }
    }
}
